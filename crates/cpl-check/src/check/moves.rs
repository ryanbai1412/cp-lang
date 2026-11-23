//! Moves a top-level `if`, `match` or loop makes, for the items after it
//! (R05.2.2): each top-level item is checked on its own (see [`Flow`]).

use super::lends::{flow_of, unparen, Flow};
use super::*;
use cpl_syntax::ast::{self, Expr, ExprKind, Stmt, StmtKind};

/// An `if`, `match` or loop standing as a statement.
pub(crate) fn is_compound(stmt: &Stmt) -> bool {
    matches!(
        &stmt.kind,
        StmtKind::Expr(Expr {
            kind: ExprKind::If { .. } | ExprKind::Match { .. },
            ..
        }) | StmtKind::For { .. }
            | StmtKind::While { .. }
    )
}

/// The statements of `stmt` that move a name by themselves ([`flow_of`]),
/// `stmt` included: in `if c: let ys = xs` or a loop body, `xs` is moved for
/// what follows (R05.2.2). A name the statement declares or assigns is left
/// out: that is another value.
pub(crate) fn nested_moves(stmt: &Stmt) -> Vec<(Stmt, Vec<String>)> {
    #[derive(Default)]
    struct Movers {
        stmts: Vec<Stmt>,
        renewed: Vec<String>,
    }
    impl cpl_syntax::visit::Visitor for Movers {
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::Assign { target, .. } = &stmt.kind {
                if let ExprKind::Name(t) = &target.kind {
                    self.renewed.push(t.clone());
                }
            }
            if !is_compound(stmt) && flow_of(std::slice::from_ref(stmt)) != Flow::None {
                self.stmts.push(stmt.clone());
            }
        }
        fn pattern(&mut self, pat: &ast::Pattern) {
            if let ast::PatternKind::Name(n) = &pat.kind {
                self.renewed.push(n.clone());
            }
        }
    }
    if !is_compound(stmt) {
        return match flow_of(std::slice::from_ref(stmt)) {
            Flow::Move(srcs) => vec![(stmt.clone(), srcs)],
            _ => Vec::new(),
        };
    }
    let mut movers = Movers::default();
    cpl_syntax::visit::walk_stmt(stmt, &mut movers);
    movers
        .stmts
        .into_iter()
        .filter_map(|s| match flow_of(std::slice::from_ref(&s)) {
            Flow::Move(srcs) => {
                let kept: Vec<String> = srcs
                    .into_iter()
                    .filter(|src| !movers.renewed.contains(src))
                    .collect();
                (!kept.is_empty()).then_some((s, kept))
            }
            _ => None,
        })
        .collect()
}

impl Checker<'_> {
    /// An earlier top-level `let`/`var`/`=` that moves `moved` (a [`flow_of`] move).
    pub(super) fn replay_move(&mut self, stmt: &Stmt, moved: &[&String], back: u32) {
        // If the statement is a call (`Bag(a)`), only treat it as a move when
        // the callee is a generated struct constructor (no custom `new`).
        let init_expr = match &stmt.kind {
            StmtKind::Let { init: Some(e), .. } => Some(unparen(e)),
            StmtKind::Assign { value, .. } => Some(unparen(value)),
            _ => None,
        };
        let is_generated_ctor = match init_expr.map(|e| &e.kind) {
            Some(ExprKind::Call { callee, .. }) => {
                let type_name = match &unparen(callee).kind {
                    ExprKind::Name(n) => Some(n.as_str()),
                    ExprKind::Generic { base, .. } => match &unparen(base).kind {
                        ExprKind::Name(n) => Some(n.as_str()),
                        _ => None,
                    },
                    _ => None,
                };
                type_name.is_some_and(|n| {
                    matches!(
                        self.type_def(n).as_deref(),
                        Some(Def::Struct { ctor: None, .. })
                    )
                })
            }
            _ => true,
        };
        if !is_generated_ctor {
            return;
        }
        for &src in moved {
            if self.moved.contains_key(src) {
                continue;
            }
            let ty = match self.env.global(src).as_deref() {
                Some(Def::Value { ty, .. }) => Some(ty.clone()),
                _ => None,
            };
            if ty.is_some_and(|ty| self.is_heap_type(&ty)) {
                let at = super::move_site(init_expr, src, back);
                self.moved.insert(src.clone(), at);
            }
        }
    }
}
