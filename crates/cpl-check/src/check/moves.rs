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
        // A later move of a moved name is where the next use's fix goes.
        for &src in moved {
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

    /// `var ys = xs` with `xs` a heap parameter: a `var` owns what it holds,
    /// and `xs` is lent by the caller, so it can't be taken (R05.7.1).
    pub(crate) fn no_var_of_lent_param(&mut self, init: &Expr, init_ty: Option<&Ty>) {
        let ExprKind::Name(src) = &unparen(init).kind else {
            return;
        };
        let lent = self.local(src).is_some_and(|l| l.keyword == "param");
        if !lent || !init_ty.is_some_and(|t| self.is_heap_type(t)) {
            return;
        }
        let message = format!(
            "`{src}` is lent by the caller, so a `var` can't take it: a `var` owns what it holds"
        );
        if let Some(diag) = self.error(codes::USE_AFTER_MOVE, init.span, message) {
            diag.help = Some(format!("copy it: `{src}.clone()`"));
            diag.fix(
                format!("clone `{src}`"),
                Span::point(init.span.end),
                ".clone()",
            );
        }
    }
}

/// Whether `body` can leave its loop early (`break`, `return`): a move just
/// before that does not reach the next round.
fn may_leave(body: &ast::Block) -> bool {
    #[derive(Default)]
    struct Leaves(bool);
    impl cpl_syntax::visit::Visitor for Leaves {
        fn stmt(&mut self, stmt: &Stmt) {
            if matches!(stmt.kind, StmtKind::Break(_) | StmtKind::Return(_)) {
                self.0 = true;
            }
        }
    }
    let mut leaves = Leaves::default();
    cpl_syntax::visit::walk_block(body, &mut leaves);
    leaves.0
}

/// The names `body` binds with `let`, `var` or a pattern.
fn declared_in(body: &ast::Block) -> Vec<String> {
    #[derive(Default)]
    struct Binds(Vec<String>);
    impl cpl_syntax::visit::Visitor for Binds {
        fn pattern(&mut self, pat: &ast::Pattern) {
            if let ast::PatternKind::Name(n) = &pat.kind {
                self.0.push(n.clone());
            }
        }
    }
    let mut binds = Binds::default();
    cpl_syntax::visit::walk_block(body, &mut binds);
    binds.0
}

impl Checker<'_> {
    /// A loop body that moves a name from outside the loop moves it again in
    /// the next round, which finds it moved (R05.2.2). `before` is what was
    /// moved when the loop started; call it after the body's scope is gone.
    pub(crate) fn no_move_across_rounds(
        &mut self,
        before: &HashMap<String, Span>,
        body: &ast::Block,
    ) {
        // A name the body declares is its own value, gone with the body.
        for name in declared_in(body) {
            match before.get(&name) {
                Some(at) => self.moved.insert(name, *at),
                None => self.moved.remove(&name),
            };
        }
        if may_leave(body) {
            return;
        }
        let mut again: Vec<(String, Span)> = self
            .moved
            .iter()
            .filter(|(k, v)| before.get(*k) != Some(*v))
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        again.sort_by_key(|(_, at)| at.start);
        for (src, at) in again {
            let reported =
                self.sink.diagnostics().iter().any(|d| {
                    d.code == codes::USE_AFTER_MOVE && d.labels.iter().any(|l| l.span == at)
                });
            if reported || !self.declared_before(&src, body.span.start) {
                continue;
            }
            let message = format!(
                "`{src}` moves here in one round of the loop, so the next round finds it already moved"
            );
            if let Some(diag) = self.error(codes::USE_AFTER_MOVE, at, message) {
                diag.help = Some(format!(
                    "move a copy each round: `{src}.clone()`, or declare `{src}` inside the loop"
                ));
            }
        }
    }

    /// `name` is a value declared before `at`: a local of an enclosing
    /// scope, or a top-level value (not a fn or type of that name).
    pub(crate) fn declared_before(&mut self, name: &str, at: u32) -> bool {
        match self.local(name) {
            Some(local) => local.span.start < at,
            None => matches!(self.global(name).as_deref(), Some(Def::Value { .. })),
        }
    }
}
