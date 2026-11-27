//! Canonical IR to optimized IR (contract 06), which the fast engine and the C++ run: the
//! checks the facts prove left out (`facts`), each scalar global only the entry names made
//! one of its locals, then each scalar temporary that only carries a value to the next
//! statement forwarded into it. The reference interpreter runs canonical
//! IR, so the suite compares the two on every case.
use crate::facts::{self, Proved};
use crate::ir::*;
use crate::prog::CTy;

pub fn optimize(
    funcs: &[Func<CTy>],
    globals: &[CTy],
    main: usize,
) -> (Vec<Func<CTy>>, Vec<Proved>) {
    let (mut out, proved) = facts::prove(funcs, main);
    localize(&mut out, globals, main);
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
        let scalar: Vec<bool> = f.locals.iter().map(scalar).collect();
        forward(&mut f.body, &uses, &scalar);
    }
    (out, proved)
}

/// A global no fn but the entry names is a local of the entry: the entry runs once, so its
/// frame holds it as long as the program, and it starts zero as a global does.
fn localize(funcs: &mut [Func<CTy>], globals: &[CTy], main: usize) {
    let mut local: Vec<Option<usize>> = globals.iter().map(|_| None).collect();
    let mut named = vec![false; globals.len()];
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

/// `Let(t, e)` and a next statement that only reads `t` once become that statement on `e`,
/// evaluated where `t` was: `t` is a scalar named nowhere else, and the place assigned is
/// reached by fields alone, so finding it reads nothing `e` could change.
fn forward(b: &mut Vec<Stmt<CTy>>, uses: &[usize], scalar: &[bool]) {
    let once = |e: &Expr<CTy>| match &e.kind {
        EK::Copy(Place::Local(t)) if uses[*t] == 2 && scalar[*t] => Some(*t),
        _ => None,
    };
    let mut i = 0;
    while i < b.len() {
        match &mut b[i] {
            Stmt::If(_, x, y) | Stmt::Loop(x, y, _) => {
                forward(x, uses, scalar);
                forward(y, uses, scalar);
            }
            Stmt::Switch(_, arms) => arms.iter_mut().for_each(|a| forward(a, uses, scalar)),
            Stmt::Region { body, .. } => forward(body, uses, scalar),
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
