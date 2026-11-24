//! Which lists of `main` a statement walks for the last time.

use std::collections::BTreeSet;

use cpl_ir::{Arg, Block, Expr, ExprKind, Iter, Program, Stmt, Ty};

use crate::walk;

/// The globals statement `k` of `block` reads only as the source of one loop
/// that runs once (a comprehension's, say), and that nothing after it
/// reads, while later statements still loop or call: so the memory they
/// hold can go to the rest of the run.
pub(crate) fn walked_last(block: &Block, k: usize) -> BTreeSet<usize> {
    let mut last = BTreeSet::new();
    let rest = Block {
        stmts: block.stmts[k + 1..].to_vec(),
    };
    if !works(&rest) {
        return last;
    }
    let here = Block {
        stmts: vec![block.stmts[k].clone()],
    };
    let walks = |s: &'_ Stmt| match s {
        Stmt::For(
            _,
            _,
            Iter::Seq(
                seq @ Expr {
                    kind: ExprKind::Global(g),
                    ..
                },
                _,
            ),
            _,
        ) => Some((*g, seq as *const Expr)),
        _ => None,
    };
    let mut walked = Vec::new();
    once(&here, &mut |s| walked.extend(walks(s)));
    for &(g, seq) in &walked {
        let mut named = BTreeSet::new();
        mentions(&here, g, &mut named);
        let mut later = BTreeSet::new();
        mentions(&rest, g, &mut later);
        if named == BTreeSet::from([seq])
            && later.is_empty()
            && walked.iter().filter(|w| w.0 == g).count() == 1
        {
            last.insert(g);
        }
    }
    last
}

/// The globals statement `k` of `block` reads or writes, in any position,
/// and nothing after it names, while later statements still loop or call.
pub(crate) fn used_last(block: &Block, k: usize) -> BTreeSet<usize> {
    let rest = Block {
        stmts: block.stmts[k + 1..].to_vec(),
    };
    if !works(&rest) {
        return BTreeSet::new();
    }
    let here = Block {
        stmts: vec![block.stmts[k].clone()],
    };
    let mut named = BTreeSet::new();
    walk::deep_exprs(&here, &mut |e| {
        if let ExprKind::Global(g) = e.kind {
            named.insert(g);
        }
    });
    walk::deep_stmts(&here, &mut |s| {
        if let Stmt::Assign(place, _, _) = s {
            if let (true, g) = walk::root(place) {
                named.insert(g);
            }
        }
    });
    named
        .into_iter()
        .filter(|&g| {
            let mut later = BTreeSet::new();
            mentions(&rest, g, &mut later);
            later.is_empty()
        })
        .collect()
}

/// Whether a statement of `block` after `k` names a variable that was
/// live before it, a global or a local declared up to `k`, whose type
/// `pred` accepts. A local the later statements declare starts after `k`.
pub(crate) fn names_after(block: &Block, k: usize, pred: &dyn Fn(&Ty) -> bool) -> bool {
    let rest = Block {
        stmts: block.stmts[k + 1..].to_vec(),
    };
    let mut fresh = BTreeSet::new();
    walk::deep_stmts(&rest, &mut |s| {
        if let Stmt::Let(slot, _) | Stmt::For(_, slot, _, _) = s {
            fresh.insert(*slot);
        }
    });
    let mut hit = false;
    walk::deep_exprs(&rest, &mut |e| {
        let live = match e.kind {
            ExprKind::Global(_) => true,
            ExprKind::Local(slot) => !fresh.contains(&slot),
            _ => false,
        };
        hit |= live && pred(&e.ty)
    });
    hit
}

/// Whether some closure's body names global `g`.
pub(crate) fn closure_names(program: &Program, g: usize) -> bool {
    program.closures.iter().any(|c| {
        let mut hit = false;
        walk::deep_exprs(&c.body, &mut |e| {
            hit |= matches!(e.kind, ExprKind::Global(id) if id == g)
        });
        hit
    })
}

/// Whether `rest` still loops or calls: work whose memory a freed list helps.
fn works(rest: &Block) -> bool {
    let mut works = false;
    walk::deep_stmts(rest, &mut |s| {
        works |= matches!(s, Stmt::For(..) | Stmt::While(..))
    });
    walk::deep_exprs(rest, &mut |e| works |= matches!(e.kind, ExprKind::Call(..)));
    works
}

/// Calls `f` on each statement of `block` that runs at most once each time
/// `block` runs: not inside a loop's body, a `while` test or a closure.
fn once<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Stmt)) {
    for stmt in &block.stmts {
        f(stmt);
        if matches!(stmt, Stmt::While(..)) {
            continue;
        }
        walk::stmt_exprs(stmt, &mut |e| once_in(e, f));
        if !matches!(stmt, Stmt::For(..)) {
            walk::stmt_blocks(stmt, &mut |inner| once(inner, f));
        }
    }
}

/// [`once`] on each `Do` block `e` runs, reached only through `e`'s own
/// children: a loop inside a block (an array builder's, say) runs its body
/// more than once, so only `once` may look inside it.
fn once_in<'p>(e: &'p Expr, f: &mut dyn FnMut(&'p Stmt)) {
    match &e.kind {
        ExprKind::Do(inner) => once(inner, f),
        _ => walk::children(e, &mut |child| once_in(child, f)),
    }
}

/// Every read or write of global `g` in `block`, each once.
fn mentions(block: &Block, g: usize, out: &mut BTreeSet<*const Expr>) {
    let at = |e: &Expr| e as *const Expr;
    walk::deep_exprs(block, &mut |e| {
        let hit = match &e.kind {
            ExprKind::Global(id) => *id == g,
            ExprKind::Mutate(_, place, _) | ExprKind::Sort(place, _, _) => {
                walk::root(place) == (true, g)
            }
            ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => args
                .iter()
                .any(|a| matches!(a, Arg::Mut(place) if walk::root(place) == (true, g))),
            _ => false,
        };
        if hit {
            out.insert(at(e));
        }
    });
    walk::deep_stmts(block, &mut |s| {
        if let Stmt::Assign(place, _, value) = s {
            if walk::root(place) == (true, g) {
                out.insert(at(value));
            }
        }
    });
}
