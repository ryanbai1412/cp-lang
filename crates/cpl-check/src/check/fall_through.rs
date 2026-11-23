//! E0409: a fn that gives a value gives one, or leaves, on every path
//! through its body.

use super::*;
use cpl_syntax::ast::{Block, Cond, Expr, ExprKind, FnBody, FnDecl, StmtKind};

impl Checker<'_> {
    /// What is checked once a fn's body is: E0409 when it gives a value
    /// (not a generator's), and L0610 for a `plain` fn.
    pub(crate) fn after_fn_body(&mut self, f: &FnDecl, ret: &Ty, generator: bool, plain: bool) {
        let gives =
            f.ret.is_some() && !generator && !matches!(ret, Ty::Unit | Ty::Error | Ty::Unknown);
        let end = match &f.body {
            FnBody::Block(block) if gives => falls_through(block),
            FnBody::Expr(expr) if gives => expr_falls(expr),
            _ => None,
        };
        if let Some((at, how)) = end {
            let name = &f.name.text;
            let message = format!("{how}, and then `{name}` ends without a value");
            let value = ret.with_article();
            let help = if how.contains("`if`") {
                format!("give `{name}` {value} on every path: an `else:` for the other case, or a value after the `if`")
            } else {
                format!("give `{name}` {value} for when the loop ends, after it")
            };
            if let Some(diag) = self.error(codes::FN_FALLS_THROUGH, at, message) {
                diag.help = Some(format!("{help}; `todo` stands for a value not written yet"));
            }
        }
        if plain {
            self.unchanged_mut_params(f);
        }
    }
}

type End = Option<(Span, &'static str)>;

/// Where running `block` can reach its end without giving a value.
fn falls_through(block: &Block) -> End {
    let last = block.stmts.last()?;
    let head = |n: u32| Span::new(last.span.start, last.span.start + n);
    match &last.kind {
        StmtKind::Expr(e) => expr_falls(e),
        StmtKind::Unchecked(b) => falls_through(b),
        StmtKind::While {
            cond,
            body,
            else_body,
        } => {
            let forever = matches!(
                cond,
                Cond::Expr(Expr {
                    kind: ExprKind::Bool(true),
                    ..
                })
            );
            loop_falls(head(5), forever, body, else_body.as_ref())
        }
        StmtKind::For {
            body, else_body, ..
        } => loop_falls(head(3), false, body, else_body.as_ref()),
        _ => None,
    }
}

fn expr_falls(e: &Expr) -> End {
    match &e.kind {
        ExprKind::Block(b) => falls_through(b),
        ExprKind::If {
            branches,
            otherwise,
        } => {
            if let Some(end) = branches.iter().find_map(|(_, b)| falls_through(b)) {
                return Some(end);
            }
            match otherwise {
                Some(b) => falls_through(b),
                None => Some((
                    Span::new(e.span.start, e.span.start + 2),
                    "this `if` has no `else`",
                )),
            }
        }
        ExprKind::Match { arms, .. } => arms.iter().find_map(|a| falls_through(&a.body)),
        _ => None,
    }
}

/// A loop ends unless it is `while true` with no `break` out of it; after
/// an `else:` the loop's end is the `else` block's.
fn loop_falls(head: Span, forever: bool, body: &Block, else_body: Option<&Block>) -> End {
    let breaks = breaks_out(body, false, &mut Vec::new());
    match else_body {
        _ if forever && !breaks => None,
        Some(b) if !breaks => falls_through(b),
        _ => Some((head, "this loop can end")),
    }
}

/// Whether a `break` in `block` leaves the loop it is the body of:
/// `nested` inside an inner loop only a labelled `break` past the
/// labels in `inner` does.
fn breaks_out<'a>(block: &'a Block, nested: bool, inner: &mut Vec<&'a str>) -> bool {
    block.stmts.iter().any(|s| match &s.kind {
        StmtKind::Break(None) => !nested,
        StmtKind::Break(Some(l)) => !inner.contains(&l.text.as_str()),
        StmtKind::While {
            body, else_body, ..
        } => {
            breaks_out(body, true, inner)
                || else_body
                    .as_ref()
                    .is_some_and(|b| breaks_out(b, nested, inner))
        }
        StmtKind::For {
            label,
            body,
            else_body,
            ..
        } => {
            let n = inner.len();
            inner.extend(label.as_ref().map(|l| l.text.as_str()));
            let out = breaks_out(body, true, inner);
            inner.truncate(n);
            out || else_body
                .as_ref()
                .is_some_and(|b| breaks_out(b, nested, inner))
        }
        StmtKind::Unchecked(b) => breaks_out(b, nested, inner),
        StmtKind::Expr(e) => expr_breaks(e, nested, inner),
        _ => false,
    })
}

fn expr_breaks<'a>(e: &'a Expr, nested: bool, inner: &mut Vec<&'a str>) -> bool {
    match &e.kind {
        ExprKind::Block(b) => breaks_out(b, nested, inner),
        ExprKind::If {
            branches,
            otherwise,
        } => {
            branches.iter().any(|(_, b)| breaks_out(b, nested, inner))
                || otherwise
                    .as_ref()
                    .is_some_and(|b| breaks_out(b, nested, inner))
        }
        ExprKind::Match { arms, .. } => arms.iter().any(|a| breaks_out(&a.body, nested, inner)),
        _ => false,
    }
}
