//! Where a value is released (R1). Ownership places every release and the runtimes only run
//! them: a frame releases `Func::drops` when the fn leaves, and `Stmt::Drop` releases a
//! discarded value and the value an overwrite replaces.

use cpl_next_ir::ir::{addressed, pure, visit_places, Arg, Expr, Place, Stmt, Target, EK};
use cpl_next_ir::prog::{stores, CTy, InstKey, Program};
use std::collections::HashMap;

pub(crate) fn place(p: &mut Program) {
    let caps: HashMap<InstKey, Vec<CTy>> = (p.index.iter())
        .filter(|(_, &i)| i < p.insts.len())
        .map(|(k, _)| (k.clone(), p.captures(k)))
        .collect();
    let params: Vec<Vec<(bool, CTy)>> = (p.insts.iter())
        .map(|f| {
            (f.params.iter().zip(&f.views))
                .map(|(&(l, _), &v)| (v, f.locals[l].clone()))
                .collect()
        })
        .collect();
    let mut insts = std::mem::take(&mut p.insts);
    for f in &mut insts {
        let stores = |t: &CTy| stores(&|s| p.parts(s), &|k| caps.get(k).cloned(), t);
        // A lent or viewed param is the caller's value: the caller releases it.
        let borrowed = |l: usize| {
            (f.params.iter().zip(&f.views)).any(|(&(x, lent), &view)| x == l && (lent || view))
        };
        let owned: Vec<bool> = (f.locals.iter().enumerate())
            .map(|(l, t)| !borrowed(l) && stores(t))
            .collect();
        // A call may read or write a local whose address is taken, so the local's old value
        // is released only once the new one is computed.
        let calm: Vec<bool> = (0..owned.len())
            .map(|l| owned[l] && !addressed(&f.body, l))
            .collect();
        let mut body = std::mem::take(&mut f.body);
        let mut placer = Placer {
            locals: &mut f.locals,
            stores: &stores,
            params: &params,
            owned,
            calm,
        };
        placer.block(&mut body);
        f.drops = (0..placer.owned.len())
            .filter(|&l| placer.owned[l])
            .collect();
        f.body = body;
    }
    p.insts = insts;
}

/// An instance's index and its captures: whether its body views one (`Func::views`), and its
/// type.
type Member = Option<(usize, Vec<(bool, CTy)>)>;

/// Expands each call through a fn value in a canonical body (`cpl_next_link::canonicalize`);
/// returns how many name a member the program lacks.
pub fn expand_calls(p: &Program, b: &mut Vec<Stmt<CTy>>) -> usize {
    let member = |k: &InstKey| -> Member {
        let &i = p.index.get(k)?;
        let views = &p.insts.get(i)?.views;
        let caps = (p.captures(k).into_iter().enumerate())
            .map(|(j, t)| (views.get(j).copied().unwrap_or(false), t))
            .collect();
        Some((i, caps))
    };
    expand(&member, b)
}

fn expand(m: &dyn Fn(&InstKey) -> Member, b: &mut Vec<Stmt<CTy>>) -> usize {
    let mut left = 0;
    for mut st in std::mem::take(b) {
        match &mut st {
            Stmt::If(_, x, y) | Stmt::Loop(x, y, _) => left += expand(m, x) + expand(m, y),
            Stmt::Switch(_, arms) => left += arms.iter_mut().map(|a| expand(m, a)).sum::<usize>(),
            Stmt::Region { body, .. } => left += expand(m, body),
            _ => {}
        }
        let (dst, e) = match st {
            Stmt::Let(l, e) if matches!(e.kind, EK::CallValue(..)) => (Some(Place::Local(l)), e),
            Stmt::Assign(at, e) if matches!(e.kind, EK::CallValue(..)) => (Some(at), e),
            Stmt::Expr(e) if matches!(e.kind, EK::CallValue(..)) => (None, e),
            st => {
                b.push(st);
                continue;
            }
        };
        let set = |e: Expr<CTy>| match &dst {
            Some(at) => Stmt::Assign(at.clone(), e),
            None => Stmt::Expr(e),
        };
        let call = |k: &InstKey, env: &Place, e: &Expr<CTy>| {
            let (EK::CallValue(_, args, at), Some((i, caps))) = (&e.kind, m(k)) else {
                return None;
            };
            let mut all: Vec<Arg<CTy>> = (caps.iter().enumerate())
                .map(|(j, (view, t))| {
                    let at = Place::Field(Box::new(env.clone()), j);
                    match view {
                        true => Arg::View(at),
                        false => Arg::Value(Expr {
                            kind: EK::Copy(at),
                            ty: t.clone(),
                            site: 0,
                        }),
                    }
                })
                .collect();
            all.extend(args.iter().cloned());
            Some(Expr {
                kind: EK::Call(Target::Inst(i), all, *at),
                ty: e.ty.clone(),
                site: e.site,
            })
        };
        let EK::CallValue(g, ..) = &e.kind else {
            continue;
        };
        let made = match (&g.kind, &g.ty) {
            (EK::Copy(env), CTy::Clo(k)) => call(k, env, &e).map(&set),
            (EK::Copy(env), CTy::Fn(_, _, ms)) => (ms.iter().enumerate())
                .map(|(i, k)| {
                    call(k, &Place::Variant(Box::new(env.clone()), i), &e).map(|c| vec![set(c)])
                })
                .collect::<Option<Vec<_>>>()
                .map(|arms| {
                    let tag = Expr {
                        kind: EK::Tag(env.clone()),
                        ty: CTy::Int,
                        site: 0,
                    };
                    Stmt::Switch(tag, arms)
                }),
            _ => None,
        };
        b.push(made.unwrap_or_else(|| {
            left += 1;
            match &dst {
                Some(at) => Stmt::Assign(at.clone(), e),
                None => Stmt::Expr(e),
            }
        }));
    }
    left
}

struct Placer<'a> {
    locals: &'a mut Vec<CTy>,
    stores: &'a dyn Fn(&CTy) -> bool,
    /// Each instance's params: whether it is viewed (`Func::views`), and its type.
    params: &'a [Vec<(bool, CTy)>],
    /// The locals the frame releases; a temporary made here is released where it is made,
    /// except one holding an argument (`held`).
    owned: Vec<bool>,
    /// The owned locals nothing but the fn itself reads or writes.
    calm: Vec<bool>,
}

impl Placer<'_> {
    fn block(&mut self, b: &mut Vec<Stmt<CTy>>) {
        for st in std::mem::take(b) {
            self.stmt(st, b);
        }
    }
    fn stmt(&mut self, mut st: Stmt<CTy>, out: &mut Vec<Stmt<CTy>>) {
        match &mut st {
            Stmt::If(_, a, b) | Stmt::Loop(a, b, _) => {
                self.block(a);
                self.block(b);
            }
            Stmt::Switch(_, arms) => arms.iter_mut().for_each(|a| self.block(a)),
            Stmt::Region { body, .. } => self.block(body),
            _ => {}
        }
        self.hoist(&mut st, out);
        match st {
            Stmt::Expr(e) if (self.stores)(&e.ty) => {
                let t = self.temp(&e.ty);
                out.push(Stmt::Let(t, e));
                out.push(Stmt::Drop(Place::Local(t)));
            }
            Stmt::Let(l, e) if self.owned.get(l) == Some(&true) => {
                self.overwrite(Place::Local(l), e, true, out)
            }
            Stmt::Assign(p, e) if (self.stores)(&e.ty) => self.overwrite(p, e, false, out),
            st => out.push(st),
        }
    }
    /// `p = e` releases what `p` held. It does so first when computing `e` cannot read `p`;
    /// else `e` goes to a temporary first, which also keeps the order in which `e` and an
    /// index of `p` trap.
    fn overwrite(&mut self, p: Place, e: Expr<CTy>, local: bool, out: &mut Vec<Stmt<CTy>>) {
        let set = |p: Place, e| match p {
            Place::Local(l) if local => Stmt::Let(l, e),
            p => Stmt::Assign(p, e),
        };
        if let Place::Local(l) = p {
            if self.calm.get(l) == Some(&true) && !reads(&e, l) {
                out.push(Stmt::Drop(p.clone()));
                out.push(set(p, e));
                return;
            }
        }
        let t = self.temp(&e.ty);
        let copy = Expr {
            kind: EK::Copy(Place::Local(t)),
            ty: e.ty.clone(),
            site: 0,
        };
        out.push(Stmt::Let(t, e));
        out.push(Stmt::Drop(p.clone()));
        out.push(set(p, copy));
        out.push(Stmt::Drop(Place::Local(t)));
    }
    fn temp(&mut self, t: &CTy) -> usize {
        self.locals.push(t.clone());
        self.owned.push(false);
        self.calm.push(false);
        self.locals.len() - 1
    }
    /// A local the frame releases, set before the statement that reads it; a set in a loop
    /// releases the value the last pass left.
    fn held(&mut self, t: &CTy) -> usize {
        let l = self.temp(t);
        let own = (self.stores)(t);
        (self.owned[l], self.calm[l]) = (own, own);
        l
    }
    /// A fresh value given to a viewed param is set in a local the caller releases, and the
    /// param views it. An operand with an effect evaluated before such a value goes to a local
    /// first, so the statement's effects keep their order.
    fn hoist(&mut self, st: &mut Stmt<CTy>, out: &mut Vec<Stmt<CTy>>) {
        match st {
            Stmt::Let(_, e)
            | Stmt::Assign(_, e)
            | Stmt::Expr(e)
            | Stmt::Yield(e)
            | Stmt::Fact(e)
            | Stmt::Return(Some(e))
            | Stmt::If(e, ..)
            | Stmt::Switch(e, _)
            | Stmt::Check { cond: e, .. } => self.expr(e, out),
            Stmt::Event(_, es) => self.fields(es, out),
            Stmt::Region { target, args, .. } => {
                let views = self.callee(target);
                self.args(&views, args, out)
            }
            _ => {}
        }
    }
    fn callee(&self, t: &Target<CTy>) -> Vec<(bool, CTy)> {
        match t {
            Target::Inst(i) => self.params.get(*i).cloned().unwrap_or_default(),
            _ => Vec::new(),
        }
    }
    fn expr(&mut self, e: &mut Expr<CTy>, out: &mut Vec<Stmt<CTy>>) {
        match &mut e.kind {
            EK::Call(t, xs, _) => {
                let views = self.callee(t);
                self.args(&views, xs, out)
            }
            EK::Intr(_, xs, _) => self.args(&[], xs, out),
            EK::CallValue(g, xs, _) => {
                let spill = spills(g, xs);
                self.item(g, spill, false, out);
                self.args(&[], xs, out)
            }
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => self.fields(xs, out),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => self.expr(x, out),
            _ => {}
        }
    }
    fn args(&mut self, views: &[(bool, CTy)], xs: &mut [Arg<CTy>], out: &mut Vec<Stmt<CTy>>) {
        for k in 0..xs.len() {
            let later = (k + 1..xs.len()).any(|j| self.arg_needs(views, j, &xs[j]));
            let view = views.get(k).is_some_and(|v| v.0);
            // A place viewed from the call's start but given to a param the callee owns is a copy
            // taken in its turn: the call writes it nowhere, so the two read the same words.
            if let (Arg::View(p), Some((false, t))) = (&xs[k], views.get(k)) {
                xs[k] = Arg::Value(Expr {
                    kind: EK::Copy(p.clone()),
                    ty: t.clone(),
                    site: 0,
                })
            }
            // A copy of a place that no later operand can write is viewed in place.
            if let (
                true,
                Arg::Value(Expr {
                    kind: EK::Copy(p), ..
                }),
            ) = (view && !later, &xs[k])
            {
                xs[k] = Arg::View(p.clone());
                continue;
            }
            if let Arg::Value(x) = &mut xs[k] {
                let fresh = view && (self.stores)(&x.ty);
                if let Some(t) = self.item(x, fresh, later, out) {
                    if view {
                        xs[k] = Arg::View(Place::Local(t));
                    }
                }
            }
        }
    }
    fn fields(&mut self, xs: &mut [Expr<CTy>], out: &mut Vec<Stmt<CTy>>) {
        for k in 0..xs.len() {
            let later = xs[k + 1..].iter().any(|x| self.needs(x));
            self.item(&mut xs[k], false, later, out);
        }
    }
    /// Places `x`'s own hoists, then sets `x` in a held local when it is `fresh` or has an
    /// effect a `later` hoist must follow; `x` then reads that local.
    fn item(
        &mut self,
        x: &mut Expr<CTy>,
        fresh: bool,
        later: bool,
        out: &mut Vec<Stmt<CTy>>,
    ) -> Option<usize> {
        self.expr(x, out);
        if !fresh && !(later && !pure(x)) {
            return None;
        }
        let t = self.held(&x.ty);
        let read = Expr {
            kind: EK::Copy(Place::Local(t)),
            ty: x.ty.clone(),
            site: 0,
        };
        let v = std::mem::replace(x, read);
        self.stmt(Stmt::Let(t, v), out);
        Some(t)
    }
    fn arg_needs(&self, views: &[(bool, CTy)], k: usize, a: &Arg<CTy>) -> bool {
        match a {
            Arg::Value(x) => {
                views.get(k).is_some_and(|v| v.0) && (self.stores)(&x.ty) || self.needs(x)
            }
            _ => false,
        }
    }
    /// Whether placing `e` sets a value in a held local.
    fn needs(&self, e: &Expr<CTy>) -> bool {
        match &e.kind {
            EK::Call(t, xs, _) => {
                let views = self.callee(t);
                (xs.iter().enumerate()).any(|(k, a)| self.arg_needs(&views, k, a))
            }
            EK::CallValue(g, xs, _) if spills(g, xs) => true,
            EK::Intr(_, xs, _) | EK::CallValue(_, xs, _) => {
                (xs.iter().enumerate()).any(|(k, a)| self.arg_needs(&[], k, a))
            }
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().any(|x| self.needs(x)),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => self.needs(x),
            _ => false,
        }
    }
}

/// A call through a fn value reads the value from a place its operands cannot write, so it is
/// set in a held local unless it is a read of a place and every operand is pure.
fn spills(g: &Expr<CTy>, xs: &[Arg<CTy>]) -> bool {
    !matches!(g.kind, EK::Copy(_)) || (xs.iter()).any(|a| matches!(a, Arg::Value(x) if !pure(x)))
}

/// Whether `e` names local `l` in any place it reads, writes or lends.
fn reads(e: &Expr<CTy>, l: usize) -> bool {
    let mut named = false;
    visit_places(&mut [Stmt::Expr(e.clone())], &mut |p| {
        named |= matches!(p.root_mut(), Place::Local(x) if *x == l);
    });
    named
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copy(l: usize) -> Expr<CTy> {
        Expr {
            kind: EK::Copy(Place::Local(l)),
            ty: CTy::Raw(Box::new(CTy::Int)),
            site: 0,
        }
    }
    fn shape(s: &Stmt<CTy>) -> String {
        let at = |p: &Place| match p {
            Place::Local(l) => l.to_string(),
            _ => "field".into(),
        };
        let of = |e: &Expr<CTy>| match &e.kind {
            EK::Copy(p) => at(p),
            _ => "?".into(),
        };
        match s {
            Stmt::Let(l, e) => format!("{l} = {}", of(e)),
            Stmt::Assign(p, e) => format!("{} = {}", at(p), of(e)),
            Stmt::Drop(p) => format!("drop {}", at(p)),
            Stmt::Expr(e) => of(e),
            _ => "?".into(),
        }
    }

    /// A discarded value is dropped in a temporary; an overwrite drops the old value first,
    /// unless the new value reads it or the place is more than a local.
    #[test]
    fn every_release_is_a_drop_where_it_happens() {
        let heap = CTy::Raw(Box::new(CTy::Int));
        let mut locals = vec![heap.clone(), heap];
        let mut body = vec![
            Stmt::Expr(copy(1)),
            Stmt::Let(0, copy(1)),
            Stmt::Assign(Place::Local(0), copy(0)),
            Stmt::Assign(Place::Field(Box::new(Place::Local(1)), 0), copy(0)),
        ];
        Placer {
            locals: &mut locals,
            stores: &|t| !matches!(t, CTy::Int),
            params: &[],
            owned: vec![true, true],
            calm: vec![true, true],
        }
        .block(&mut body);
        let got: Vec<String> = body.iter().map(shape).collect();
        let want = [
            "2 = 1",
            "drop 2", //
            "drop 0",
            "0 = 1", //
            "3 = 0",
            "drop 0",
            "0 = 3",
            "drop 3", //
            "4 = 0",
            "drop field",
            "field = 4",
            "drop 4",
        ];
        assert_eq!(got, want);
        assert_eq!(locals.len(), 5);
    }

    /// A call through a fn value made by an expression reads it from a held local the frame
    /// releases; one read from a place, with pure operands, is called through that place.
    #[test]
    fn a_fn_value_is_called_from_a_place() {
        let heap = CTy::Raw(Box::new(CTy::Int));
        let call = |g: Expr<CTy>| {
            Stmt::Expr(Expr {
                kind: EK::CallValue(Box::new(g), vec![Arg::Value(copy(0))], 0),
                ty: CTy::Int,
                site: 0,
            })
        };
        let made = Expr {
            kind: EK::MakeClosure(vec![copy(0)]),
            ty: heap.clone(),
            site: 0,
        };
        let mut locals = vec![heap.clone(), heap];
        let mut body = vec![call(copy(1)), call(made)];
        let mut placer = Placer {
            locals: &mut locals,
            stores: &|t| !matches!(t, CTy::Int),
            params: &[],
            owned: vec![true, true],
            calm: vec![true, true],
        };
        placer.block(&mut body);
        assert_eq!(placer.owned, [true, true, true]);
        let callee = |s: &Stmt<CTy>| match s {
            Stmt::Expr(Expr {
                kind: EK::CallValue(g, ..),
                ..
            }) => shape(&Stmt::Expr((**g).clone())),
            s => shape(s),
        };
        let got: Vec<String> = body.iter().map(callee).collect();
        assert_eq!(got, ["1", "drop 2", "2 = ?", "2"]);
    }

    /// A call through a closure calls its member with a view of each capture the member views
    /// and a copy of each other; one through a fn value of two members switches on its tag.
    #[test]
    fn a_fn_value_call_is_its_member_call() {
        let heap = CTy::Raw(Box::new(CTy::Int));
        let key = |n: u32| InstKey {
            func: cpl_next_ir::ir::FuncKey::Ctx(cpl_next_ir::ty::FnId(0), n),
            targs: vec![],
            sets: vec![],
        };
        let members: HashMap<InstKey, (usize, Vec<(bool, CTy)>)> = [
            (
                key(0),
                (7, vec![(true, heap.clone()), (false, heap.clone())]),
            ),
            (key(1), (8, vec![])),
        ]
        .into_iter()
        .collect();
        let through = |ty: CTy| {
            Stmt::Let(
                1,
                Expr {
                    kind: EK::CallValue(
                        Box::new(Expr {
                            kind: EK::Copy(Place::Local(0)),
                            ty,
                            site: 0,
                        }),
                        vec![Arg::Value(copy(2))],
                        0,
                    ),
                    ty: CTy::Int,
                    site: 0,
                },
            )
        };
        let clo = CTy::Clo(Box::new(key(0)));
        let both = CTy::Fn(vec![], Box::new(CTy::Int), vec![key(0), key(1)]);
        let mut body = vec![through(clo), through(both)];
        assert_eq!(expand(&|k| members.get(k).cloned(), &mut body), 0);
        let args = |s: &Stmt<CTy>| match s {
            Stmt::Assign(Place::Local(1), e) => match &e.kind {
                EK::Call(Target::Inst(i), xs, _) => {
                    let xs: Vec<String> = (xs.iter())
                        .map(|a| match a {
                            Arg::View(_) => "view".into(),
                            Arg::Value(e) => shape(&Stmt::Expr(e.clone())),
                            _ => "?".into(),
                        })
                        .collect();
                    format!("{i}({})", xs.join(" "))
                }
                _ => "?".into(),
            },
            _ => "?".into(),
        };
        assert_eq!(args(&body[0]), "7(view field 2)");
        let Stmt::Switch(_, arms) = &body[1] else {
            panic!("a fn value of two members switches on its tag");
        };
        let got: Vec<String> = arms.iter().map(|a| args(&a[0])).collect();
        assert_eq!(got, ["7(view field 2)", "8(2)"]);
    }
}
