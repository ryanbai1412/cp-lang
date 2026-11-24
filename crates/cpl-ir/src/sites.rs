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
    let mut add = |block: &'p Block| written(block, &mut *f);
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
    within(block, &[], f)
}

/// [`written`] for a block inside statements spanned `outer`. A statement
/// spanned inside the last one written before it in its block or an
/// enclosing one is one a lowering wrote for that statement's expression
/// (an inline `all`'s loop over `xs`), not one of its own.
fn within<'p>(block: &'p Block, outer: &[Span], f: &mut dyn FnMut(&'p Stmt)) {
    let mut prev = None;
    for stmt in &block.stmts {
        let span = stmt.span();
        let made = |s: Span| {
            prev.iter()
                .chain(outer)
                .any(|p| p.start <= s.start && s.end <= p.end)
        };
        let own = span.is_some_and(|s| !made(s));
        if own {
            f(stmt);
            prev = span;
        }
        let mut inner = outer.to_vec();
        inner.extend(prev);
        arms(stmt, &inner, f);
    }
}

fn arms<'p>(stmt: &'p Stmt, outer: &[Span], f: &mut dyn FnMut(&'p Stmt)) {
    match stmt {
        Stmt::If(_, then, otherwise) => {
            within(then, outer, f);
            match &otherwise.stmts[..] {
                [next @ Stmt::If(..)] => arms(next, outer, f),
                _ => within(otherwise, outer, f),
            }
        }
        // A body statement spanned before the loop's iterable binds its
        // pattern (`for (a, b) in ...`): the loop's, not one a step starts.
        Stmt::For(..) => {
            let at = stmt.span().map_or(0, |span| span.start);
            walk::stmt_blocks(stmt, &mut |inner| {
                within(inner, outer, &mut |s: &'p Stmt| {
                    if s.span().is_none_or(|span| span.start >= at) {
                        f(s)
                    }
                })
            })
        }
        _ => walk::stmt_blocks(stmt, &mut |inner| within(inner, outer, f)),
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
    let written = user_made(program, &found);
    while let Some(block) = work.pop() {
        walk::deep_exprs(block, &mut |e| {
            if let ExprKind::Closure(id) = e.kind {
                if !written.contains(&id) && found.insert(id) {
                    work.push(&program.closures[id].body);
                }
            }
        });
    }
    found
}

/// The closures the user's own code makes, other than `made` (the compiler's):
/// a library body may still name one, once a pass calls a known fn param's
/// closure directly, and it stays the user's.
fn user_made(program: &Program, made: &HashSet<usize>) -> HashSet<usize> {
    let mut found = HashSet::new();
    let mut work: Vec<&Block> = vec![&program.setup, &program.main];
    work.extend(
        (program.fns.iter().enumerate())
            .filter(|(id, _)| !program.library_fns.get(*id).copied().unwrap_or(false))
            .map(|(_, f)| &f.body),
    );
    while let Some(block) = work.pop() {
        walk::deep_exprs(block, &mut |e| {
            if let ExprKind::Closure(id) = e.kind {
                if !made.contains(&id) && found.insert(id) {
                    work.push(&program.closures[id].body);
                }
            }
        });
    }
    found
}
