//! Canonical IR to optimized IR (contract 06), which the fast engine and the C++ run: the
//! checks the facts prove left out (`facts`), each scalar global only the entry names made
//! one of its locals, each flag a branch sets only for the next branch to test folded into the
//! first, then each plain temporary that only carries a value to the next statement forwarded
//! into it. The reference interpreter runs canonical
//! IR, so the suite compares the two on every case.
use crate::facts::{self, Proved};
use crate::ir::*;
use crate::prog::{CTy, Program};

pub fn optimize(funcs: &[Func<CTy>], p: &Program) -> (Vec<Func<CTy>>, Vec<Proved>) {
    let (mut out, proved) = facts::prove(funcs, p.main);
    localize(&mut out, &p.globals, p.main);
    for f in &mut out {
        let mut uses = vec![0usize; f.locals.len()];
        for (l, _) in &f.params {
            uses[*l] = usize::MAX;
        }
        visit_places(&mut f.body, &mut |p| {
            if let Place::Local(l) = p.root_mut() {
                uses[*l] = uses[*l].saturating_add(1);
            }
        });
        let flags: Vec<bool> = (f.locals.iter().zip(&uses))
            .map(|(t, &n)| *t == CTy::Bool && n == 3)
            .collect();
        fold(&mut f.body, &flags);
        let plain: Vec<bool> = f.locals.iter().map(|t| p.plain(t)).collect();
        forward(&mut f.body, &uses, &plain);
    }
    (out, proved)
}

/// A scalar global no fn but the entry names is a local of the entry: the entry runs once, so
/// its frame holds it as long as the program, and it starts zero as a global does. A global
/// with storage stays one, as ownership places no release for it in the entry's frame.
fn localize(funcs: &mut [Func<CTy>], globals: &[CTy], main: usize) {
    let mut local: Vec<Option<usize>> = globals.iter().map(|_| None).collect();
    let mut named: Vec<bool> = globals.iter().map(|t| !scalar(t)).collect();
    for (i, f) in funcs.iter_mut().enumerate() {
        if i != main {
            visit_places(&mut f.body, &mut |p| {
                if let Place::Global(g) = p.root_mut() {
                    named[*g] = true;
                }
            });
        }
    }
    let m = &mut funcs[main];
    visit_places(&mut m.body, &mut |p| {
        if let Place::Global(g) = *p.root_mut() {
            if named[g] {
                return;
            }
            let l = *local[g].get_or_insert_with(|| {
                m.locals.push(globals[g].clone());
                m.locals.len() - 1
            });
            *p.root_mut() = Place::Local(l);
        }
    });
}

fn scalar(t: &CTy) -> bool {
    matches!(
        t,
        CTy::Int | CTy::Bool | CTy::Char | CTy::U64 | CTy::I128 | CTy::F64 | CTy::Dom(_)
    )
}

/// `if c: A; f = a else: B; f = !a` then `if f: X else: Y` becomes `if c: A; X else: B; Y`
/// (or `Y`, `X` when `a` is false): `f` is a `Bool` named only by the two sets and the test.
fn fold(b: &mut Vec<Stmt<CTy>>, flags: &[bool]) {
    let set = |s: Option<&Stmt<CTy>>| match s {
        Some(Stmt::Assign(
            Place::Local(f),
            Expr {
                kind: EK::Bool(v), ..
            },
        )) if flags[*f] => Some((*f, *v)),
        _ => None,
    };
    let mut i = 0;
    while i < b.len() {
        let (head, tail) = b.split_at_mut(i + 1);
        if let (Stmt::If(_, x, y), Some(Stmt::If(t, xx, yy))) = (&mut head[i], tail.first_mut()) {
            if let (Some((f, a)), Some((g, z)), EK::Copy(Place::Local(h))) =
                (set(x.last()), set(y.last()), &t.kind)
            {
                if f == g && f == *h && a != z {
                    let (xx, yy) = (std::mem::take(xx), std::mem::take(yy));
                    let (to_x, to_y) = if a { (xx, yy) } else { (yy, xx) };
                    x.pop();
                    y.pop();
                    x.extend(to_x);
                    y.extend(to_y);
                    b.remove(i + 1);
                    continue;
                }
            }
        }
        match &mut b[i] {
            Stmt::If(_, x, y) | Stmt::Loop(x, y, _) => {
                fold(x, flags);
                fold(y, flags);
            }
            Stmt::Switch(_, arms) => arms.iter_mut().for_each(|a| fold(a, flags)),
            Stmt::Region { body, .. } => fold(body, flags),
            _ => {}
        }
        i += 1;
    }
}

/// `Let(t, e)` and a next statement that only reads `t` once become that statement on `e`,
/// evaluated where `t` was: `t` is plain (it holds neither storage nor an address, so no count
/// or lend is lost) and named nowhere else, and the place assigned is
/// reached by fields alone, so finding it reads nothing `e` could change.
fn forward(b: &mut Vec<Stmt<CTy>>, uses: &[usize], plain: &[bool]) {
    let once = |e: &Expr<CTy>| match &e.kind {
        EK::Copy(Place::Local(t)) if uses[*t] == 2 && plain[*t] => Some(*t),
        _ => None,
    };
    let mut i = 0;
    while i < b.len() {
        match &mut b[i] {
            Stmt::If(_, x, y) | Stmt::Loop(x, y, _) => {
                forward(x, uses, plain);
                forward(y, uses, plain);
            }
            Stmt::Switch(_, arms) => arms.iter_mut().for_each(|a| forward(a, uses, plain)),
            Stmt::Region { body, .. } => forward(body, uses, plain),
            _ => {}
        }
        let fixed = match b.get(i + 1) {
            Some(Stmt::Assign(p, _)) => by_fields(p),
            _ => true,
        };
        let (head, tail) = b.split_at_mut(i + 1);
        if let (
            Stmt::Let(t, v),
            Some(Stmt::Let(_, e) | Stmt::Assign(_, e) | Stmt::Return(Some(e))),
        ) = (&mut head[i], tail.first_mut())
        {
            if fixed && once(e) == Some(*t) {
                std::mem::swap(v, e);
                b.remove(i);
                continue;
            }
        }
        i += 1;
    }
}

/// Whether `p` is a local or global or a field of one, through lent slots at most.
fn by_fields(p: &Place) -> bool {
    match p {
        Place::Local(_) | Place::Global(_) => true,
        Place::Field(b, _) | Place::Through(b) => by_fields(b),
        Place::Variant(..) | Place::Deref(_) | Place::Slot(..) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(kind: EK<CTy>) -> Expr<CTy> {
        Expr {
            kind,
            ty: CTy::Bool,
            site: 0,
        }
    }
    fn set(f: usize, v: bool) -> Stmt<CTy> {
        Stmt::Assign(Place::Local(f), e(EK::Bool(v)))
    }

    #[test]
    fn a_flag_set_for_the_next_branch_folds_into_the_first() {
        let mut b = vec![
            Stmt::If(
                e(EK::Copy(Place::Local(0))),
                vec![set(1, false)],
                vec![set(1, true)],
            ),
            Stmt::If(
                e(EK::Copy(Place::Local(1))),
                vec![Stmt::Break],
                vec![Stmt::Continue],
            ),
        ];
        fold(&mut b, &[false, true]);
        let [Stmt::If(_, x, y)] = &b[..] else {
            panic!("not folded: {b:?}")
        };
        assert!(matches!(&x[..], [Stmt::Continue]) && matches!(&y[..], [Stmt::Break]));
    }

    #[test]
    fn a_flag_read_elsewhere_stays() {
        let mut b = vec![
            Stmt::If(
                e(EK::Copy(Place::Local(0))),
                vec![set(1, true)],
                vec![set(1, false)],
            ),
            Stmt::If(e(EK::Copy(Place::Local(1))), vec![Stmt::Break], vec![]),
        ];
        fold(&mut b, &[false, false]);
        assert_eq!(b.len(), 2);
    }
}
