//! Ownership: what each instance link made reads, writes and holds, and which passed places
//! are copies (spec chapter 5, R7, R9, R10, R12).
//!
//! A fn's summary names the places it touches: top-level bindings and its own parameters,
//! with the field path kept, so `p.a` and `p.b` do not overlap (R05.8.1). A call through a
//! function value is the join of the closures its set can hold; a set that is a parameter
//! of the fn stays symbolic in the summary and is filled in at each call.
use cpl_next_ir::ir::{Arg, Expr, Place, PrintPiece, Site, Stmt, EK};
use cpl_next_ir::prog::{CTy, Program};
use cpl_next_ir::ty::*;
use std::collections::{BTreeSet, HashMap};
mod inst;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Root {
    Global(usize),
    Local(usize),
}
/// A variable and a path under it. An element at a computed key is the whole container; one at
/// a literal key is apart from the container's other literal keys (R05.8.2).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Pl {
    root: Root,
    path: Vec<Step>,
}
impl Pl {
    fn whole(root: Root) -> Pl {
        Pl { root, path: vec![] }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Step {
    Field(usize),
    Key(i64),
}
/// A summary through a call that recurses on a field (`Desc<Desc<T>>`'s `lt` calling `T`'s) would
/// name ever deeper places; past this depth it names the place holding them, which overlaps all.
fn overlap(a: &Pl, b: &Pl) -> bool {
    a.root == b.root && a.path.iter().zip(&b.path).all(|(x, y)| x == y)
}

#[derive(Default, Clone, PartialEq, Debug)]
struct Fx {
    reads: BTreeSet<Pl>,
    writes: BTreeSet<Pl>,
    /// Lent to something that is still running when this fn yields or lends.
    held: BTreeSet<Pl>,
    /// Function values it calls whose set is a parameter of the fn.
    calls: BTreeSet<SetTy>,
    /// It yields a value that borrows its own locals, so a walk's item lends from the walk.
    yields_own: bool,
}
impl Fx {
    fn add(&mut self, o: &Fx) {
        self.reads.extend(o.reads.iter().cloned());
        self.writes.extend(o.writes.iter().cloned());
        self.held.extend(o.held.iter().cloned());
        self.calls.extend(o.calls.iter().cloned());
        self.yields_own |= o.yields_own;
    }
}

/// Settles each place passed to a call (`EK::Pass`) in the linked program: a copy taken in its
/// turn where ownership found the call also writes it (R4), else a view from the call's start
/// (R11) or a read.
pub fn place_copies(p: &mut Program, copies: &BTreeSet<(usize, Site)>) {
    for (i, f) in p.insts.iter_mut().enumerate() {
        Settle {
            locals: &mut f.locals,
            copy: |s: Site| copies.contains(&(i, s)),
            saved: HashMap::new(),
        }
        .block(&mut f.body);
    }
}
struct Settle<'a, C> {
    locals: &'a mut Vec<CTy>,
    copy: C,
    /// A passed place saved before a later operand ran, by site: the temporary its copy is in.
    saved: HashMap<Site, usize>,
}
impl<C: Fn(Site) -> bool> Settle<'_, C> {
    fn block(&mut self, b: &mut Vec<Stmt<CTy>>) {
        for mut st in std::mem::take(b) {
            if let Stmt::Let(t, e) = &mut st {
                if let EK::Pass(p, view, s) = &e.kind {
                    if *view && !(self.copy)(*s) {
                        continue;
                    }
                    if *view {
                        self.saved.insert(*s, *t);
                    }
                    e.kind = EK::Copy(p.clone());
                    b.push(st);
                    continue;
                }
            }
            let mut pre = Vec::new();
            self.stmt(&mut st, &mut pre);
            b.extend(pre);
            b.push(st);
        }
    }
    fn stmt(&mut self, st: &mut Stmt<CTy>, pre: &mut Vec<Stmt<CTy>>) {
        match st {
            Stmt::Let(_, e)
            | Stmt::Assign(_, e)
            | Stmt::Expr(e)
            | Stmt::Return(Some(e))
            | Stmt::Yield(e)
            | Stmt::Fact(e)
            | Stmt::Check { cond: e, .. } => self.expr(e, pre),
            Stmt::If(c, a, b) => {
                self.expr(c, pre);
                self.block(a);
                self.block(b);
            }
            Stmt::Loop(a, b, _) => {
                self.block(a);
                self.block(b);
            }
            Stmt::Switch(e, bs) => {
                self.expr(e, pre);
                bs.iter_mut().for_each(|b| self.block(b));
            }
            Stmt::Print(ps) => {
                for x in ps {
                    if let PrintPiece::Value(e, _) = x {
                        self.expr(e, pre);
                    }
                }
            }
            Stmt::Event(_, xs) => xs.iter_mut().for_each(|x| self.expr(x, pre)),
            Stmt::Region { args, body, .. } => {
                self.args(args, pre);
                self.block(body);
            }
            Stmt::Return(None) | Stmt::LendOut(_) | Stmt::Break | Stmt::Continue => {}
        }
    }
    fn args(&mut self, xs: &mut [Arg<CTy>], pre: &mut Vec<Stmt<CTy>>) {
        for a in xs {
            let Arg::Value(v) = a else { continue };
            match &v.kind {
                EK::Pass(p, true, s) if !(self.copy)(*s) => *a = Arg::View(p.clone()),
                EK::Pass(_, true, s) if self.saved.contains_key(s) => {
                    v.kind = EK::Copy(Place::Local(self.saved[s]));
                }
                _ => self.expr(v, pre),
            }
        }
    }
    fn expr(&mut self, e: &mut Expr<CTy>, pre: &mut Vec<Stmt<CTy>>) {
        if let EK::Pass(p, _, s) = &e.kind {
            let p = p.clone();
            e.kind = if (self.copy)(*s) {
                let t = self.locals.len();
                self.locals.push(e.ty.clone());
                let ty = e.ty.clone();
                pre.push(Stmt::Let(
                    t,
                    Expr {
                        site: 0,
                        kind: EK::Copy(p),
                        ty,
                    },
                ));
                EK::Copy(Place::Local(t))
            } else {
                EK::Copy(p)
            };
            return;
        }
        match &mut e.kind {
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => {
                xs.iter_mut().for_each(|x| self.expr(x, pre))
            }
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => self.expr(x, pre),
            EK::Intr(_, xs, _) | EK::Call(_, xs, _) => self.args(xs, pre),
            EK::CallValue(c, xs, _) => {
                self.expr(c, pre);
                xs.iter_mut().for_each(|x| self.expr(x, pre));
            }
            _ => {}
        }
    }
}

pub use inst::{check, Owned};
