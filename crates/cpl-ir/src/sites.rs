//! The statements of the user's own file, the ones a profile counts and a
//! coverage report lists. An imported module's code is lowered with the
//! span of the user's expression that asked for it, so its statements are
//! left out: their time belongs to that expression's line.

use crate::{walk, Block, ExprKind, Program, Stmt};
use cpl_diag::Span;
use std::collections::HashSet;

/// Every statement with a span outside the imported modules' fns and the
/// closures they make, each once. Keyed by address, so the set is only
/// good for this `program`.
pub fn user_stmts(program: &Program) -> HashSet<*const Stmt> {
    let mut stmts = HashSet::new();
    each_user_stmt(program, &mut |stmt| {
        stmts.insert(stmt as *const Stmt);
    });
    stmts
}

/// The first written statement of each user fn and closure body: where a
/// profiled build opens a frame, so the time after the call returns is the
/// caller's again (library code calling back into user code).
pub fn user_entries(program: &Program) -> HashSet<*const Stmt> {
    let library = library_closures(program);
    let fns = (program.fns.iter().enumerate())
        .filter(|(id, _)| !program.library_fns.get(*id).copied().unwrap_or(false))
        .map(|(_, f)| &f.body);
    let closures = (program.closures.iter().enumerate())
        .filter(|(id, _)| !library.contains(id))
        .map(|(_, c)| &c.body);
    (fns.chain(closures))
        .filter_map(|body| body.stmts.iter().find(|stmt| stmt.span().is_some()))
        .map(|stmt| stmt as *const Stmt)
        .collect()
}

/// The spans of [`user_stmts`].
pub fn user_spans(program: &Program) -> Vec<Span> {
    let mut spans = Vec::new();
    each_user_stmt(program, &mut |stmt| spans.extend(stmt.span()));
    spans
}

fn each_user_stmt<'p>(program: &'p Program, f: &mut dyn FnMut(&'p Stmt)) {
    let library = library_closures(program);
    let mut add = |block: &'p Block| written(block, f);
    add(&program.setup);
    add(&program.main);
    for (id, f) in program.fns.iter().enumerate() {
        if !program.library_fns.get(id).copied().unwrap_or(false) {
            add(&f.body);
        }
    }
    for (id, c) in program.closures.iter().enumerate() {
        if !library.contains(&id) {
            add(&c.body);
        }
    }
}

/// The statements of `block` and of the blocks of its statements, but
/// not those inside an expression (`ExprKind::Do`): those are how an
/// expression is lowered, which the C++ may write another way, and their
/// time is the enclosing statement's.
///
/// An `if` that is the whole `else` of another is the next arm of one
/// chain (`elif`, an arm of a `match`), which the C++ may test as one
/// `switch`: it is not a statement of its own.
fn written<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Stmt)) {
    for stmt in &block.stmts {
        if stmt.span().is_some() {
            f(stmt);
        }
        arms(stmt, f);
    }
}

fn arms<'p>(stmt: &'p Stmt, f: &mut dyn FnMut(&'p Stmt)) {
    match stmt {
        Stmt::If(_, then, otherwise) => {
            written(then, f);
            match &otherwise.stmts[..] {
                [next @ Stmt::If(..)] => arms(next, f),
                _ => written(otherwise, f),
            }
        }
        _ => walk::stmt_blocks(stmt, &mut |inner| written(inner, f)),
    }
}

/// The closures an imported module's fn makes, the ones the compiler
/// writes for the user (a fn passed as a value, a library parameter's
/// default: every statement spanned as the expression that makes it), and
/// the ones those make.
fn library_closures(program: &Program) -> HashSet<usize> {
    let mut found = HashSet::new();
    let mut work: Vec<&Block> = (program.fns.iter().enumerate())
        .filter(|(id, _)| program.library_fns.get(*id).copied().unwrap_or(false))
        .map(|(_, f)| &f.body)
        .collect();
    let mut user: Vec<&Block> = vec![&program.setup, &program.main];
    user.extend(program.fns.iter().map(|f| &f.body));
    user.extend(program.closures.iter().map(|c| &c.body));
    for block in user {
        walk::deep_exprs(block, &mut |e| {
            if let ExprKind::Closure(id) = e.kind {
                let body = &program.closures[id].body;
                let made = |stmt: &Stmt| stmt.span().is_none_or(|span| span == e.span);
                if body.stmts.iter().all(made) && found.insert(id) {
                    work.push(body);
                }
            }
        });
    }
    while let Some(block) = work.pop() {
        walk::deep_exprs(block, &mut |e| {
            if let ExprKind::Closure(id) = e.kind {
                if found.insert(id) {
                    work.push(&program.closures[id].body);
                }
            }
        });
    }
    found
}
