//! Ownership: what each instance link made reads, writes and holds, and which passed places
//! are copies (spec chapter 5, R7, R9, R10, R12). Before link, `lent` decides which values are
//! lent rather than moved, since link keys an instance by those marks; lowering makes none.
//!
//! A fn's summary names the places it touches: top-level bindings and its own parameters,
//! with the field path kept, so `p.a` and `p.b` do not overlap (R05.8.1). A call through a
//! function value is the join of the closures its set can hold; a set that is a parameter
//! of the fn stays symbolic in the summary and is filled in at each call.
use cpl_next_decls::views::Signatures;
use cpl_next_ir::ir::{sets, Arg, Expr, Func, FuncKey, Place, Site, Stmt, EK};
use cpl_next_ir::prog::{CTy, Program};
use cpl_next_ir::ty::*;
use std::collections::{BTreeSet, HashMap};
mod drops;
mod escape;
pub use drops::expand_calls;
mod inst;
mod lent;

/// The variable a place starts at: a top-level one, or a local of the instance.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Root {
    Global(usize),
    Local(usize),
}
/// A variable and a path under it. An element at a computed key is the whole container; one at
/// a literal key is apart from the container's other literal keys (R05.8.2).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Pl {
    pub root: Root,
    pub path: Vec<Step>,
}
impl Pl {
    fn whole(root: Root) -> Pl {
        Pl { root, path: vec![] }
    }
}
/// One step under a variable: a field, or an element at a literal key.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Step {
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
/// (R11) or a read. Then marks the params each instance reads in place (`Func::views`), and
/// places every release (`Func::drops`, `Stmt::Drop`).
pub(crate) fn place_copies(p: &mut Program, copies: &BTreeSet<(usize, Site)>) {
    for (i, f) in p.insts.iter_mut().enumerate() {
        Settle {
            locals: &mut f.locals,
            copy: |s: Site| copies.contains(&(i, s)),
            saved: HashMap::new(),
        }
        .block(&mut f.body);
    }
    let views: Vec<Vec<bool>> = p.insts.iter().map(|f| views(p, f)).collect();
    for (f, v) in p.insts.iter_mut().zip(views) {
        f.views = v;
    }
    drops::place(p);
}
/// Which params the callee reads where the caller has them: nothing writes one and it has a
/// heap part, so passing it takes no copy (R1).
fn views(p: &Program, f: &Func<CTy>) -> Vec<bool> {
    (f.params.iter())
        .map(|(l, lent)| !lent && !sets(&f.body, *l) && !p.plain(&f.locals[*l]))
        .collect()
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
            Stmt::Event(cpl_next_ir::ir::EventPoint { vals: xs, .. }) => {
                xs.iter_mut().for_each(|x| self.expr(x, pre))
            }
            Stmt::Region { args, body, .. } => {
                self.args(args, pre);
                self.block(body);
            }
            Stmt::Return(None)
            | Stmt::LendOut(_)
            | Stmt::Drop(_)
            | Stmt::Break
            | Stmt::Continue => {}
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
                self.args(xs, pre);
            }
            _ => {}
        }
    }
}

pub use inst::{after, Bound, Owned};

/// What ownership found before link that its check after link reports: the generic fns'
/// verify errors and the `@cache` rule's findings on the bodies as written.
pub struct Before {
    pub(crate) generic: Vec<cpl_next_ir::msg::Msg>,
    pub(crate) cache: Vec<cpl_next_ir::msg::Msg>,
}

/// Ownership before link (contract 4): marks what the generic fns lend (`lent`), since link
/// keys an instance by those marks, and checks the `@cache` rule on the bodies as written.
/// `globals` names each top-level binding and whether it is a `var`.
pub fn before(
    d: &dyn Signatures,
    funcs: &mut HashMap<FuncKey, Func<Ty>>,
    tys: &mut [Ty],
    globals: &[(String, bool)],
) -> Before {
    use cpl_next_ir::verify::{all, Stage};
    let mutable: Vec<bool> = globals.iter().map(|(_, m)| *m).collect();
    let generic = all(funcs.values(), Stage::Generic, &[], tys.len());
    lent::mark(d, funcs, tys, &mutable);
    let cache = inst::cache_calls(d, funcs);
    Before { generic, cache }
}

/// The parameters a call to `f` lends to its result, receiver first, from its signature: the
/// heap ones it declares `lends` (R05.6.1), else each whose type is a type parameter the result
/// type holds, and, for a result whose type is a type parameter, each that holds it: the result
/// may be a cell of it (R05.5.3).
fn lends(d: &dyn Signatures, f: FnId) -> BTreeSet<usize> {
    use cpl_next_decls::decls::SelfMode;
    use cpl_next_ir::ty::params_in;
    let fd = d.f(f);
    let mut held = BTreeSet::new();
    params_in(&fd.ret, &mut held);
    let recv = (fd.self_mode != SelfMode::None).then_some(fd.self_ty.as_ref());
    let tys: Vec<&Ty> = (recv.flatten().into_iter())
        .chain(fd.params.iter().map(|p| &p.ty))
        .collect();
    (tys.iter().enumerate())
        .filter(|(i, t)| match &fd.lends {
            Some(ls) => ls.contains(i) && !t.copied(&cpl_next_decls::fields::fields(d)),
            None => match (t, &fd.ret) {
                (Ty::Param(k), _) => held.contains(k),
                (_, Ty::Param(k)) => {
                    let mut ps = BTreeSet::new();
                    params_in(t, &mut ps);
                    ps.contains(k)
                }
                _ => false,
            },
        })
        .map(|(i, _)| i)
        .collect()
}

/// Where each local is read for the last time (R05.5.3): the statement that names it once,
/// after every other place it is named, in the loop it is declared in, or a `return`.
#[derive(Default)]
pub(crate) struct Last {
    total: HashMap<usize, usize>,
    seen: HashMap<usize, usize>,
    pub depth: usize,
    at: HashMap<usize, usize>,
    pub here: BTreeSet<usize>,
}
impl Last {
    pub fn new<T: Clone>(b: &[Stmt<T>]) -> Last {
        let mut last = Last::default();
        uses(b, &mut last.total);
        last
    }
    pub fn stmt<T: Clone>(&mut self, s: &Stmt<T>) {
        self.here.clear();
        if let Stmt::Let(l, _) = s {
            self.at.insert(*l, self.depth);
        }
        // A compound statement's own reads, not its blocks', which are walked one by one.
        let head = match s {
            Stmt::If(c, ..) | Stmt::Switch(c, _) => Some(Stmt::Expr(c.clone())),
            Stmt::Region { .. } => {
                let mut h = s.clone();
                if let Stmt::Region { body, .. } = &mut h {
                    body.clear();
                }
                Some(h)
            }
            Stmt::Loop(..) => return,
            _ => None,
        };
        let mut here = HashMap::new();
        uses(std::slice::from_ref(head.as_ref().unwrap_or(s)), &mut here);
        if head.is_some() {
            for (l, n) in here {
                *self.seen.entry(l).or_default() += n;
            }
            return;
        }
        let ret = matches!(s, Stmt::Return(_));
        for (l, n) in here {
            let seen = self.seen.entry(l).or_default();
            *seen += n;
            let after = Some(&*seen) == self.total.get(&l) && self.at.get(&l) == Some(&self.depth);
            if n == 1 && (ret || after) {
                self.here.insert(l);
            }
        }
    }
}
/// How many places in `b` each local is part of.
fn uses<T: Clone>(b: &[Stmt<T>], n: &mut HashMap<usize, usize>) {
    let mut b = b.to_vec();
    cpl_next_ir::ir::visit_places(&mut b, &mut |p| {
        if let Some(l) = p.local() {
            *n.entry(l).or_default() += 1;
        }
    });
}
