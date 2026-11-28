//! Where running a fn's body can reach its end without giving a value (E0409): a fn that
//! gives a value gives one, or leaves, on every path through its body.

use crate::ast::{Block, Cond, Expr, ExprKind, FnBody, StmtKind};
use cpl_diag::Span;

/// Where the body can end without a value, and how it does, for the message.
pub type End = Option<(Span, &'static str)>;

impl FnBody {
    /// Where running the body can reach its end without giving a value.
    pub fn falls_through(&self) -> End {
        match self {
            FnBody::Block(b) => b.falls_through(),
            FnBody::Expr(e) => e.falls_through(),
            FnBody::None => None,
        }
    }
}

impl Block {
    /// Where running the block can reach its end without giving a value.
    pub fn falls_through(&self) -> End {
        let last = self.stmts.last()?;
        let head = |n: u32| Span::new(last.span.start, last.span.start + n);
        match &last.kind {
            StmtKind::Expr(e) => e.falls_through(),
            StmtKind::Unchecked(b) => b.falls_through(),
            StmtKind::While {
                cond,
                body,
                else_body,
            } => loop_falls(head(5), forever(cond), body, else_body.as_ref()),
            StmtKind::For {
                body, else_body, ..
            } => loop_falls(head(3), false, body, else_body.as_ref()),
            _ => None,
        }
    }
}

impl Block {
    /// The block can't reach its end: it leaves ([`Block::leaves`]), or its last statement is
    /// a `while true` with no `break` out of it.
    pub fn never_ends(&self) -> bool {
        self.leaves()
            || matches!(self.stmts.last().map(|s| &s.kind), Some(StmtKind::While { cond, body, .. })
                if forever(cond) && !breaks_out(body, false, &mut Vec::new()))
    }
}

impl Expr {
    /// Running the expression can't reach its end; see [`Block::never_ends`].
    pub fn never_ends(&self) -> bool {
        match &self.kind {
            ExprKind::Block(b) => b.never_ends(),
            _ => self.leaves(),
        }
    }

    /// Where running the expression can end without giving a value; see
    /// [`Block::falls_through`].
    pub fn falls_through(&self) -> End {
        match &self.kind {
            ExprKind::Block(b) => b.falls_through(),
            ExprKind::If {
                branches,
                otherwise,
            } => {
                if let Some(end) = branches.iter().find_map(|(_, b)| b.falls_through()) {
                    return Some(end);
                }
                match otherwise {
                    Some(b) => b.falls_through(),
                    None => Some((
                        Span::new(self.span.start, self.span.start + 2),
                        "this `if` has no `else`",
                    )),
                }
            }
            ExprKind::Match { arms, .. } => arms.iter().find_map(|a| a.body.falls_through()),
            _ => None,
        }
    }
}

pub use cpl_next_syntax::fall_through::help;

/// `while true`; `while True:` already has its error, so it reads as the `true` it means.
fn forever(cond: &Cond) -> bool {
    match cond {
        Cond::Expr(Expr { kind, .. }) => match kind {
            ExprKind::Bool(b) => *b,
            ExprKind::Name(n) => cpl_diag::habits::find(cpl_diag::habits::Kind::Value, n)
                .is_some_and(|h| h.fix == "true"),
            _ => false,
        },
        _ => false,
    }
}

/// A loop ends unless it is `while true` with no `break` out of it; after
/// an `else:` the loop's end is the `else` block's.
fn loop_falls(head: Span, forever: bool, body: &Block, else_body: Option<&Block>) -> End {
    let breaks = breaks_out(body, false, &mut Vec::new());
    match else_body {
        _ if forever && !breaks => None,
        Some(b) if !breaks => b.falls_through(),
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
