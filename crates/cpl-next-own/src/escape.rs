//! Every path through a lender lends (R05.6.3), checked on each generic fn before link, and
//! no closure that shares a `var` outlives its frame (R05.10), checked over the linked
//! instances once own has walked them.

use cpl_diag::codes;
use cpl_next_decls::diag::Msg;
use cpl_next_decls::views::Signatures;
use cpl_next_ir::ir::{Arg, Expr, Func, FuncKey, Stmt, Target, EK};
use cpl_next_ir::prog::{CTy, InstKey, Program};

/// A lender lends one place on every path through it: its one expression, or a `lend mut`
/// (R05.6.3). A path that leaves without one would lose the caller's write.
pub(crate) fn lenders_lend<'a, T: 'a>(funcs: impl IntoIterator<Item = &'a Func<T>>) -> Vec<Msg> {
    /// Whether a path through `b` reaches its end, and whether one leaves the fn, unlent.
    fn gap<T>(b: &[Stmt<T>]) -> (bool, bool) {
        let (mut on, mut left) = (true, false);
        for s in b {
            if !on {
                break;
            }
            match s {
                Stmt::LendOut(_) | Stmt::Break | Stmt::Continue => on = false,
                Stmt::Return(_) => (on, left) = (false, true),
                Stmt::If(_, a, b) => {
                    let ((fa, la), (fb, lb)) = (gap(a), gap(b));
                    (on, left) = (fa || fb, left || la || lb);
                }
                Stmt::Switch(_, arms) if !arms.is_empty() => {
                    let gs: Vec<_> = arms.iter().map(|a| gap(a)).collect();
                    on = gs.iter().any(|g| g.0);
                    left |= gs.iter().any(|g| g.1);
                }
                // A lend's body runs once; a loop's or a walk's may not run, and lends nothing.
                Stmt::Region {
                    lend: true, body, ..
                } => {
                    let (f, l) = gap(body);
                    (on, left) = (f, left || l);
                }
                Stmt::Loop(body, step, _) => left |= gap(body).1 || gap(step).1,
                Stmt::Region { body, .. } => left |= gap(body).1,
                _ => {}
            }
        }
        (on, left)
    }
    let mut bad = Vec::new();
    for f in funcs {
        if !matches!(f.region, Some((true, _))) || gap(&f.body) == (false, false) {
            continue;
        }
        let last = f.body.last().map(std::slice::from_ref).unwrap_or_default();
        let site = site_of(last, &|_| true).or_else(|| site_of(&f.body, &|_| true));
        let m = Msg::verdict(
            codes::BAD_PROJECTION,
            format!(
                "`{}` is declared `lends mut`, so every path through it ends in `lend mut place`; \
                 a body that leaves without one would lose the caller's write (R05.6.3)",
                f.shown
            ),
        );
        let m = match site.and_then(|i| f.sites.get(i)) {
            Some(s) => Msg::at(s.clone(), m),
            None => m,
        };
        if !bad.contains(&m) {
            bad.push(m);
        }
    }
    bad
}

/// A closure that shares a `var` of the fn around it is lent: it cannot be returned (spec R05.10).
/// Nor can it be stored through a `mut` parameter.
pub(crate) fn escape(d: &dyn Signatures, p: &Program) -> Vec<Msg> {
    // `made` keeps the closures the returning fn itself makes: one passed in through a
    // container's `new` or `at` captures what its maker's frame holds, which outlives them.
    fn holds_ref(
        p: &Program,
        t: &CTy,
        seen: &mut Vec<CTy>,
        made: &dyn Fn(&InstKey) -> bool,
    ) -> bool {
        if seen.contains(t) {
            return false;
        }
        seen.push(t.clone());
        match t {
            CTy::Ref(_) => true,
            CTy::Raw(e) => holds_ref(p, e, seen, made),
            CTy::Struct(..) => p.parts(t).iter().any(|f| holds_ref(p, f, seen, made)),
            CTy::Fn(_, _, ms) => ms
                .iter()
                .any(|k| holds_ref(p, &CTy::Clo(Box::new(k.clone())), seen, made)),
            CTy::Clo(k) if made(k) => p.captures(k).iter().any(|c| holds_ref(p, c, seen, made)),
            _ => false,
        }
    }
    let mut bad = Vec::new();
    for (me, f) in p.insts.iter().enumerate() {
        let owner = match f.key {
            FuncKey::Decl(g) | FuncKey::Ctx(g, _) => Some(g),
            _ => None,
        };
        // A generator's frame ends with its walk, so a closure made in one outlives it anywhere.
        let made = |k: &InstKey| match (owner, &k.func) {
            (Some(g), FuncKey::Lambda(_)) => {
                let o = (p.index.get(k)).and_then(|&i| p.insts.get(i)?.maker.as_ref()?.decl());
                o == Some(g) || o.is_some_and(|o| matches!(d.f(o).region, Some((false, _))))
            }
            (Some(_), _) => false,
            (None, _) => true,
        };
        // Said where the fn makes the closure, or else where it is called.
        let at = |bad: &mut Vec<Msg>, m: String| {
            let made_here = |e: &Expr<CTy>| {
                matches!((&e.kind, &e.ty), (EK::MakeClosure(_), CTy::Clo(k))
                    if made(k) && p.captures(k).iter().any(|c| matches!(c, CTy::Ref(_))))
            };
            let calls_me =
                |e: &Expr<CTy>| matches!(&e.kind, EK::Call(Target::Inst(j), ..) if *j == me);
            let site = (site_of(&f.body, &made_here).and_then(|i| f.sites.get(i))).or_else(|| {
                (p.insts.iter())
                    .find_map(|g| site_of(&g.body, &calls_me).and_then(|i| g.sites.get(i)))
            });
            let m = Msg::verdict(codes::LENT_ESCAPES, m);
            bad.push(match site {
                Some(s) => Msg::at(s.clone(), m),
                None => m,
            });
        };
        if holds_ref(p, &f.ret, &mut Vec::new(), &made) {
            at(&mut bad, format!("`{}` returns a function that uses a `var` of the fn that made it; the variable is gone by then", f.name));
        }
        // What a `mut` parameter holds is the caller's, so a closure stored there outlives this fn too.
        let stored = (f.params.iter().filter(|(_, m)| *m)).any(|(l, _)| {
            let t = match &f.locals[*l] {
                CTy::Ref(t) => t,
                t => t,
            };
            owner.is_some() && holds_ref(p, t, &mut Vec::new(), &made)
        });
        if stored {
            at(&mut bad, format!("`{}` stores a function that uses one of its `var`s through a `mut` parameter; the variable is gone when it returns", f.name));
        }
    }
    bad
}

/// The site of the first expression of `b` that `hit` takes, outermost first.
fn site_of<T>(b: &[Stmt<T>], hit: &dyn Fn(&Expr<T>) -> bool) -> Option<usize> {
    fn ex<T>(e: &Expr<T>, hit: &dyn Fn(&Expr<T>) -> bool) -> Option<usize> {
        if hit(e) {
            return Some(e.site as usize);
        }
        let args = |xs: &[Arg<T>]| {
            xs.iter().find_map(|a| match a {
                Arg::Value(v) => ex(v, hit),
                Arg::View(_) | Arg::Lend(_) => None,
            })
        };
        match &e.kind {
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().find_map(|x| ex(x, hit)),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => ex(x, hit),
            EK::Intr(_, xs, _) | EK::Call(_, xs, _) => args(xs),
            EK::CallValue(g, xs, _) => ex(g, hit).or_else(|| args(xs)),
            EK::Copy(_)
            | EK::Pass(..)
            | EK::AddrOf(_)
            | EK::Lent(_)
            | EK::Tag(_)
            | EK::Int(_)
            | EK::F64(_)
            | EK::Bool(_)
            | EK::Unit
            | EK::Text(_)
            | EK::DomLen(_)
            | EK::DomLo(_)
            | EK::DomHeld(_)
            | EK::DomCounted(_)
            | EK::HasBound(..) => None,
        }
    }
    b.iter().find_map(|s| match s {
        Stmt::Let(_, e) | Stmt::Assign(_, e) => ex(e, hit),
        Stmt::Expr(e) | Stmt::Yield(e) | Stmt::Fact(e) | Stmt::Check { cond: e, .. } => ex(e, hit),
        Stmt::Return(e) => e.as_ref().and_then(|e| ex(e, hit)),
        Stmt::If(c, x, y) => ex(c, hit)
            .or_else(|| site_of(x, hit))
            .or_else(|| site_of(y, hit)),
        Stmt::Loop(x, y, _) => site_of(x, hit).or_else(|| site_of(y, hit)),
        Stmt::Switch(c, arms) => ex(c, hit).or_else(|| arms.iter().find_map(|a| site_of(a, hit))),
        Stmt::Region { args: xs, body, .. } => (xs.iter())
            .find_map(|a| match a {
                Arg::Value(v) => ex(v, hit),
                Arg::View(_) | Arg::Lend(_) => None,
            })
            .or_else(|| site_of(body, hit)),
        Stmt::Event(cpl_next_ir::ir::EventPoint { vals: xs, .. }) => {
            xs.iter().find_map(|x| ex(x, hit))
        }
        Stmt::LendOut(_) | Stmt::Drop(_) | Stmt::Break | Stmt::Continue => None,
    })
}
