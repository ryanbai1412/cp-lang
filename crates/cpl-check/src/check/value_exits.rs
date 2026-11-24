//! E0410: a `return`, `break` or `continue` leaves from a statement, or
//! from an arm of the `if`, `match` or block that is a statement's whole
//! value, whose arms run as statements. Deeper, inside an argument, an
//! operand or a condition, the expression around it is half worked out
//! when the arm would leave.

use super::*;
use cpl_diag::msgs;
use cpl_syntax::ast::{BinOp, Expr, ExprKind, FnBody, Stmt, StmtKind};
use cpl_syntax::visit::{self, Visitor};
use std::collections::HashSet;

impl Checker<'_> {
    pub(crate) fn value_exits(&mut self, stmts: &[Stmt]) {
        let mut values = Values::default();
        for stmt in stmts {
            visit::walk_stmt(stmt, &mut values);
        }
        for at in values.found {
            let word: String = self.src[at.start as usize..]
                .chars()
                .take_while(|c| c.is_ascii_alphabetic())
                .collect();
            if let Some(diag) = self.report(at, msgs::check::LEAVES_A_PART_OF_A_VALUE, &[&word]) {
                diag.help = Some(format!(
                    "give the `if` or `match` its own `let` first, then use the name: `let v = if ..: .. else: {word}`"
                ));
            }
        }
    }
}

/// The values whose arms may leave, and the leaves found anywhere else.
#[derive(Default)]
struct Values {
    whole: HashSet<Span>,
    found: Vec<Span>,
    reported: HashSet<Span>,
}

fn unwrapped(mut e: &Expr) -> &Expr {
    while let ExprKind::Paren(inner) = &e.kind {
        e = inner;
    }
    e
}

/// `x ?? return`: the way out of a statement's whole value is a statement.
fn way_out(e: &Expr) -> Option<&Expr> {
    let ExprKind::Binary {
        op: BinOp::Coalesce,
        rhs,
        ..
    } = &e.kind
    else {
        return None;
    };
    let ExprKind::Block(block) = &rhs.kind else {
        return None;
    };
    let [stmt] = block.stmts.as_slice() else {
        return None;
    };
    matches!(
        stmt.kind,
        StmtKind::Return(_) | StmtKind::Break(_) | StmtKind::Continue(_)
    )
    .then_some(&**rhs)
}

impl Visitor for Values {
    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let {
                init: Some(e),
                otherwise: None,
                ..
            }
            | StmtKind::Assign { value: e, .. }
            | StmtKind::Expr(e)
            | StmtKind::Return(Some(e)) => {
                let root = unwrapped(e);
                self.whole.insert(root.span);
                if let Some(rhs) = way_out(root) {
                    self.whole.insert(rhs.span);
                }
            }
            StmtKind::Fn(f) => {
                if let FnBody::Expr(body) = &f.body {
                    self.whole.insert(unwrapped(body).span);
                }
            }
            _ => {}
        }
    }

    fn expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Lambda { body, .. } => {
                self.whole.insert(unwrapped(body).span);
            }
            ExprKind::If { .. } | ExprKind::Match { .. } | ExprKind::Block(_)
                if !self.whole.contains(&expr.span) =>
            {
                for at in visit::leaves_out_of(expr) {
                    if self.reported.insert(at) {
                        self.found.push(at);
                    }
                }
            }
            _ => {}
        }
    }
}
