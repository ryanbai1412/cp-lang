//! Loop forms (`plan/domains/lint-fmt/design/loop-forms.md`): what a loop
//! does, found once so every loop-shaped lint reads the same answer.

use super::{breaks_after_setting, pushed_onto, var_decl};
use crate::Calls;
use cpl_syntax::ast::*;

/// `var name = init` followed by a `for` loop: a loop whose job may be to
/// compute `name`.
#[derive(Clone, Copy)]
pub(super) struct AccLoop<'a> {
    pub name: &'a str,
    pub init: &'a Expr,
    pub pat: &'a Pattern,
    pub iter: &'a Expr,
    pub body: &'a Block,
    /// Asks the checker about the loop's calls.
    pub calls: Calls<'a>,
}

/// What an [`AccLoop`] does with its variable.
pub(super) enum Form<'a> {
    /// Some branch sets it and breaks: a search (L0501).
    Search,
    /// The body is one push onto it, maybe under one `if`: a build (L0507).
    Build {
        cond: Option<&'a Expr>,
        value: &'a Expr,
    },
}

impl<'a> AccLoop<'a> {
    pub(super) fn of(decl: &'a Stmt, next: &'a Stmt, calls: Calls<'a>) -> Option<AccLoop<'a>> {
        let (name, init) = var_decl(decl)?;
        let StmtKind::For {
            label: None,
            pat,
            iter,
            body,
            else_body: None,
        } = &next.kind
        else {
            return None;
        };
        // `repeat n:` is a `for _ in 0..n` whose pattern and range nobody wrote.
        if matches!(pat.kind, PatternKind::Wildcard) && pat.span.end - pat.span.start > 1 {
            return None;
        }
        // `for x in fields(a)` is unrolled at compile time: no sequence to fold.
        if let ExprKind::Call { callee, .. } = &iter.kind {
            if matches!(&callee.kind, ExprKind::Name(n) if n == "fields") {
                return None;
            }
        }
        Some(AccLoop {
            name,
            init,
            pat,
            iter,
            body,
            calls,
        })
    }

    /// Every form the loop has, a search first.
    pub(super) fn forms(&self) -> Vec<Form<'a>> {
        let mut out = Vec::new();
        let flips = |v: &Expr| match (&v.kind, &self.init.kind) {
            (ExprKind::Bool(to), ExprKind::Bool(from)) => to != from,
            _ => !matches!(v.kind, ExprKind::Bool(_)),
        };
        if breaks_after_setting(&self.body.stmts, self.name)
            || write_once(&self.body.stmts, self.name).is_some_and(|(_, v)| flips(v))
        {
            out.push(Form::Search);
        }
        out.extend(self.single_write());
        out
    }

    /// A body that is one push onto the variable, maybe under one `if`.
    fn single_write(&self) -> Option<Form<'a>> {
        let [only] = &self.body.stmts[..] else {
            return None;
        };
        let (cond, only) = match &only.kind {
            StmtKind::Expr(Expr {
                kind:
                    ExprKind::If {
                        branches,
                        otherwise: None,
                    },
                ..
            }) => match &branches[..] {
                [(Cond::Expr(cond), then)] => match &then.stmts[..] {
                    [only] => (Some(cond), only),
                    _ => return None,
                },
                _ => return None,
            },
            _ => (None, only),
        };
        match &only.kind {
            StmtKind::Expr(e) => {
                pushed_onto(e, self.name, self.calls).map(|value| Form::Build { cond, value })
            }
            _ => None,
        }
    }
}

/// `if c: x = v; break` alone: `(c, v)`.
pub(super) fn set_then_break<'e>(body: &'e Block, name: &str) -> Option<(&'e Expr, &'e Expr)> {
    let [Stmt {
        kind: StmtKind::Expr(e),
        ..
    }] = &body.stmts[..]
    else {
        return None;
    };
    let ExprKind::If {
        branches,
        otherwise: None,
    } = &e.kind
    else {
        return None;
    };
    let [(Cond::Expr(cond), then)] = &branches[..] else {
        return None;
    };
    let [Stmt {
        kind:
            StmtKind::Assign {
                target,
                op: AssignOp::Set,
                value,
            },
        ..
    }, Stmt {
        kind: StmtKind::Break(None),
        ..
    }] = &then.stmts[..]
    else {
        return None;
    };
    if !matches!(&target.kind, ExprKind::Name(n) if n == name) {
        return None;
    }
    Some((cond, value))
}

/// `if c: x = v` alone, no `break`, where only the first write counts: `v` is
/// `true`/`false`, or `c` ends in `and x == none`. `(c without the guard, v)`.
pub(super) fn write_once<'e>(stmts: &'e [Stmt], name: &str) -> Option<(&'e Expr, &'e Expr)> {
    let [Stmt {
        kind:
            StmtKind::Expr(Expr {
                kind:
                    ExprKind::If {
                        branches,
                        otherwise: None,
                    },
                ..
            }),
        ..
    }] = stmts
    else {
        return None;
    };
    let [(Cond::Expr(cond), then)] = &branches[..] else {
        return None;
    };
    let [Stmt {
        kind:
            StmtKind::Assign {
                target,
                op: AssignOp::Set,
                value,
            },
        ..
    }] = &then.stmts[..]
    else {
        return None;
    };
    if !matches!(&target.kind, ExprKind::Name(n) if n == name) {
        return None;
    }
    if matches!(value.kind, ExprKind::Bool(_)) {
        return Some((cond, value));
    }
    let ExprKind::Binary {
        op: BinOp::And,
        lhs,
        rhs,
        ..
    } = &cond.kind
    else {
        return None;
    };
    let unset = matches!(&rhs.kind, ExprKind::Compare { first, rest }
        if matches!(&first.kind, ExprKind::Name(n) if n == name)
            && matches!(&rest[..], [(CmpOp::Eq, v)] if matches!(v.kind, ExprKind::None)));
    unset.then_some((&**lhs, value))
}

/// `xs.rev.collect()` when a loop over `0..N` pushes `xs[N - 1 - i]`.
pub(super) fn reversed(i: &str, it: &str, v: &str) -> Option<String> {
    let n = it.strip_prefix("0..")?;
    let (xs, rest) = v.split_once('[')?;
    let idx = rest.strip_suffix(']')?;
    let len = format!("{xs}.len");
    let ok = idx == format!("{n} - 1 - {i}") && (n == len || !n.contains('.'));
    (ok && !xs.is_empty() && xs.chars().all(|c| c.is_alphanumeric() || c == '_'))
        .then(|| format!("{xs}.rev.collect()"))
}
