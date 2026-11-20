//! Where an optional is known to hold a value.
//!
//! After `if x == none: return`, and inside `if x != none:`, the name `x`
//! stands for the value it holds, so the code reads `x + 1` and not
//! `x! + 1`. The rule is read off the source alone, so the checker and the
//! lowering ask the same questions here and can't disagree: a test names
//! `x` and `none`, and the code the test covers never assigns `x`.

use crate::ast::*;
use crate::visit::{walk_block, walk_expr, walk_stmt, Visitor};

/// The names that hold a value when `cond` comes out as `when`: each
/// `x != none` of an `and` chain that is true, each `x == none` of an
/// `or` chain that is false.
fn present<'a>(cond: &'a Expr, when: bool, out: &mut Vec<&'a str>) {
    match &cond.kind {
        ExprKind::Paren(inner) => present(inner, when, out),
        ExprKind::Unary {
            op: UnOp::Not,
            expr,
        } => present(expr, !when, out),
        ExprKind::Binary { op, lhs, rhs, .. }
            if *op == if when { BinOp::And } else { BinOp::Or } =>
        {
            present(lhs, when, out);
            present(rhs, when, out);
        }
        ExprKind::Compare { first, rest } => {
            let [(op, other)] = rest.as_slice() else {
                return;
            };
            let wanted = if when { CmpOp::Ne } else { CmpOp::Eq };
            let name = match (&unparen(first).kind, &unparen(other).kind) {
                (ExprKind::Name(name), ExprKind::None) | (ExprKind::None, ExprKind::Name(name)) => {
                    name
                }
                _ => return,
            };
            if *op == wanted && !out.contains(&name.as_str()) {
                out.push(name);
            }
        }
        _ => {}
    }
}

/// Whether `e` is a name where an optional is asked for by what is
/// written around it: `x!`, `x ?? 0`, `x?`, `x == none`, `if let v = x`,
/// `match x`. There a name keeps the type it was declared with, so code
/// that spells the unwrap out still means what it says.
pub fn asks_optional(e: &Expr) -> bool {
    matches!(unparen(e).kind, ExprKind::Name(_))
}

/// Whether one side of this comparison is `none`.
pub fn tests_none(first: &Expr, rest: &[(CmpOp, Expr)]) -> bool {
    std::iter::once(first)
        .chain(rest.iter().map(|(_, e)| e))
        .any(|e| matches!(unparen(e).kind, ExprKind::None))
}

fn unparen(mut e: &Expr) -> &Expr {
    while let ExprKind::Paren(inner) = &e.kind {
        e = inner;
    }
    e
}

/// The name an assignment target or a `mut` argument changes.
fn root(mut e: &Expr) -> Option<&str> {
    loop {
        e = match &e.kind {
            ExprKind::Index { base, .. } | ExprKind::Field { base, .. } => base,
            ExprKind::Paren(inner) | ExprKind::Unwrap(inner) => inner,
            ExprKind::Name(name) => return Some(name),
            _ => return None,
        };
    }
}

/// Whether some code may give `name` a new value, or declares another
/// `name`.
struct Writes<'a, 'c> {
    name: &'a str,
    found: bool,
    calls: &'c mut Calls<'c>,
}

/// Asked of a call in the code a test covers: does calling the fn or
/// method `callee` assign `name`? A fn that does may leave it `none`.
pub type Calls<'c> = dyn FnMut(&str, &str) -> bool + 'c;

impl Writes<'_, '_> {
    fn target(&mut self, target: &Expr) {
        match &target.kind {
            ExprKind::Tuple(items) => items.iter().for_each(|item| self.target(item)),
            _ => self.found |= root(target) == Some(self.name),
        }
    }
}

impl Visitor for Writes<'_, '_> {
    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Assign { target, .. } => self.target(target),
            StmtKind::Fn(decl) => {
                self.found |= decl.params.iter().any(|p| p.name.text == self.name);
            }
            _ => {}
        }
    }

    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Call { callee, args } => {
                for arg in args.iter().filter(|a| a.mutable) {
                    self.found |= root(&arg.value) == Some(self.name);
                }
                let callee = match &callee.kind {
                    ExprKind::Name(name) => Some(name.as_str()),
                    ExprKind::Field { name, .. } => Some(name.text.as_str()),
                    _ => None,
                };
                if let Some(callee) = callee {
                    self.found |= (self.calls)(callee, self.name);
                }
            }
            ExprKind::Lambda { params, .. } => {
                params.iter().for_each(|param| self.pattern(param));
            }
            _ => {}
        }
    }

    fn pattern(&mut self, pat: &Pattern) {
        match &pat.kind {
            PatternKind::Name(name) => self.found |= name == self.name,
            PatternKind::Tuple(items) | PatternKind::Variant { args: items, .. } => {
                items.iter().for_each(|item| self.pattern(item))
            }
            _ => {}
        }
    }
}

fn kept<'a>(names: Vec<&'a str>, calls: &mut Calls, walk: impl Fn(&mut Writes)) -> Vec<&'a str> {
    names
        .into_iter()
        .filter(|name| {
            let mut writes = Writes {
                name,
                found: false,
                calls: &mut *calls,
            };
            walk(&mut writes);
            !writes.found
        })
        .collect()
}

/// What each fn declared in some code assigns outside itself: the names
/// its body assigns and does not declare, and those of the fns it calls. Kept by fn name, so a method
/// counts for every call of its name: a call that may assign a name ends
/// what was known about it.
#[derive(Default)]
pub struct Mutators(std::collections::HashMap<String, Vec<String>>);

impl Mutators {
    pub fn of<'a>(stmts: impl IntoIterator<Item = &'a Stmt>) -> Mutators {
        struct Bodies(Vec<(String, Vec<String>, Vec<String>)>);
        /// Assigned, called, declared.
        #[derive(Default)]
        struct Body(Vec<String>, Vec<String>, Vec<String>);
        impl Visitor for Body {
            fn pattern(&mut self, pat: &Pattern) {
                match &pat.kind {
                    PatternKind::Name(name) => self.2.push(name.clone()),
                    PatternKind::Tuple(items) | PatternKind::Variant { args: items, .. } => {
                        items.iter().for_each(|item| self.pattern(item))
                    }
                    _ => {}
                }
            }
            fn stmt(&mut self, stmt: &Stmt) {
                if let StmtKind::Assign { target, .. } = &stmt.kind {
                    let mut targets = vec![target];
                    while let Some(target) = targets.pop() {
                        match &target.kind {
                            ExprKind::Tuple(items) => targets.extend(items),
                            _ => self.0.extend(root(target).map(String::from)),
                        }
                    }
                }
            }
            fn expr(&mut self, e: &Expr) {
                if let ExprKind::Call { callee, args } = &e.kind {
                    for arg in args.iter().filter(|a| a.mutable) {
                        self.0.extend(root(&arg.value).map(String::from));
                    }
                    match &callee.kind {
                        ExprKind::Name(name) => self.1.push(name.clone()),
                        ExprKind::Field { name, .. } => self.1.push(name.text.clone()),
                        _ => {}
                    }
                }
            }
        }
        impl Visitor for Bodies {
            fn stmt(&mut self, stmt: &Stmt) {
                let mut body = Body::default();
                let name = match &stmt.kind {
                    StmtKind::Fn(decl) => {
                        body.2.extend(decl.params.iter().map(|p| p.name.text.clone()));
                        match &decl.body {
                            FnBody::Block(block) => walk_block(block, &mut body),
                            FnBody::Expr(expr) => walk_expr(expr, &mut body),
                            FnBody::None => {}
                        }
                        decl.name.text.clone()
                    }
                    // `let clear = () => x = none`.
                    StmtKind::Let {
                        pat:
                            Pattern {
                                kind: PatternKind::Name(name),
                                ..
                            },
                        init: Some(init @ Expr {
                            kind: ExprKind::Lambda { .. },
                            ..
                        }),
                        ..
                    } => {
                        if let ExprKind::Lambda { params, .. } = &init.kind {
                            params.iter().for_each(|param| body.pattern(param));
                        }
                        walk_expr(init, &mut body);
                        name.clone()
                    }
                    _ => return,
                };
                let Body(mut assigned, called, declared) = body;
                assigned.retain(|name| !declared.contains(name));
                self.0.push((name, assigned, called));
            }
        }
        let mut bodies = Bodies(Vec::new());
        for stmt in stmts {
            walk_stmt(stmt, &mut bodies);
        }
        let mut writes: std::collections::HashMap<String, Vec<String>> = Default::default();
        for (name, assigned, _) in &bodies.0 {
            writes.entry(name.clone()).or_default().extend(assigned.iter().cloned());
        }
        // A fn also assigns what the fns it calls assign.
        loop {
            let mut grew = false;
            for (name, _, calls) in &bodies.0 {
                for callee in calls {
                    let more: Vec<String> = writes.get(callee).cloned().unwrap_or_default();
                    let mine = writes.entry(name.clone()).or_default();
                    for written in more {
                        if !mine.contains(&written) {
                            mine.push(written);
                            grew = true;
                        }
                    }
                }
            }
            if !grew {
                break;
            }
        }
        Mutators(writes)
    }

    /// Whether calling `callee` may assign `name`.
    pub fn writes(&self, callee: &str, name: &str) -> bool {
        self.0.get(callee).is_some_and(|names| names.iter().any(|n| n == name))
    }
}

/// The names that hold a value all through `body`, the branch of an
/// `if` that `cond` picks.
pub fn in_branch<'a>(cond: &'a Cond, body: &Block, calls: &mut Calls) -> Vec<&'a str> {
    let mut names = Vec::new();
    if let Cond::Expr(cond) = cond {
        present(cond, true, &mut names);
    }
    kept(names, calls, |writes| walk_block(body, writes))
}

/// The names that hold a value while `rhs` runs, in `lhs and rhs` or
/// `lhs or rhs`: `rhs` only runs once `lhs` has come out one way, as in
/// `best == none or x < best`.
pub fn in_rhs<'a>(op: BinOp, lhs: &'a Expr, rhs: &Expr, calls: &mut Calls) -> Vec<&'a str> {
    let mut names = Vec::new();
    match op {
        BinOp::And => present(lhs, true, &mut names),
        BinOp::Or => present(lhs, false, &mut names),
        _ => {}
    }
    kept(names, calls, |writes| walk_expr(rhs, writes))
}

/// The names that hold a value in everything an `if` tries once `cond`
/// has failed: the branches in `later` and the `else`.
pub fn past<'a>(
    cond: &'a Cond,
    later: &[(Cond, Block)],
    otherwise: Option<&Block>,
    calls: &mut Calls,
) -> Vec<&'a str> {
    let mut names = Vec::new();
    if let Cond::Expr(cond) = cond {
        present(cond, false, &mut names);
    }
    kept(names, calls, |writes| {
        for (_, body) in later {
            walk_block(body, writes);
        }
        if let Some(body) = otherwise {
            walk_block(body, writes);
        }
    })
}

/// The names that hold a value in `rest`, the statements after `stmt` in
/// its block: `stmt` is an `if` without an `else` whose every branch
/// leaves, so getting past it disproves each test.
pub fn after<'a>(stmt: &'a Stmt, rest: &[Stmt], calls: &mut Calls) -> Vec<&'a str> {
    let StmtKind::Expr(Expr {
        kind: ExprKind::If {
            branches,
            otherwise: None,
        },
        ..
    }) = &stmt.kind
    else {
        return Vec::new();
    };
    let leaves = |body: &Block| {
        matches!(
            body.stmts.last().map(|s| &s.kind),
            Some(StmtKind::Return(_) | StmtKind::Break(_) | StmtKind::Continue(_))
        )
    };
    if rest.is_empty() || !branches.iter().all(|(_, body)| leaves(body)) {
        return Vec::new();
    }
    let mut names = Vec::new();
    for (cond, _) in branches {
        if let Cond::Expr(cond) = cond {
            present(cond, false, &mut names);
        }
    }
    kept(names, calls, |writes| {
        rest.iter().for_each(|stmt| walk_stmt(stmt, writes))
    })
}
