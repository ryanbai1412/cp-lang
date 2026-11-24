//! E0805: a value of a domain declared below `@!multitest` stored into a
//! name declared above it, which would carry the position into the next
//! test case (R02.9.2).

use crate::check::HoverEntry;
use crate::db::{Analysis, Database};
use cpl_diag::{codes, Diagnostic, Label, Span};
use cpl_syntax::ast::{Expr, ExprKind, Stmt, StmtKind};
use cpl_syntax::visit::{walk_stmt, Visitor};

/// Stores into a name from `above`: the name and the stored value's span.
struct Stores<'a> {
    above: &'a [String],
    found: Vec<(String, Span)>,
}

fn root(e: &Expr) -> Option<&str> {
    match &e.kind {
        ExprKind::Name(n) => Some(n),
        ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => root(base),
        _ => None,
    }
}

impl Stores<'_> {
    fn outer(&self, e: &Expr) -> Option<String> {
        root(e)
            .filter(|n| self.above.iter().any(|a| a == n))
            .map(str::to_string)
    }
}

impl Visitor for Stores<'_> {
    fn stmt(&mut self, s: &Stmt) {
        if let StmtKind::Assign { target, value, .. } = &s.kind {
            if let Some(name) = self.outer(target) {
                self.found.push((name, value.span));
            }
        }
    }

    fn expr(&mut self, e: &Expr) {
        let ExprKind::Call { callee, args } = &e.kind else {
            return;
        };
        let ExprKind::Field { base, .. } = &callee.kind else {
            return;
        };
        if let Some(name) = self.outer(base) {
            let values = args.iter().map(|a| &a.value);
            let values = values.filter(|v| !matches!(v.kind, ExprKind::Lambda { .. }));
            self.found.extend(values.map(|v| (name.clone(), v.span)));
        }
    }
}

impl Database {
    pub(crate) fn per_test_leaks(&mut self, analysis: &mut Analysis) {
        let divider = analysis.items.iter().find_map(|item| {
            let stmt = item
                .syntax
                .parsed
                .stmts
                .iter()
                .find(|s| s.decorators.iter().any(|d| d.name.text == "multitest"))?;
            Some(stmt.span.start + item.offset)
        });
        let Some(divider) = divider else {
            return;
        };
        let mut above: Vec<String> = analysis.scope.names.keys().cloned().collect();
        above.retain(|n| analysis.decl_span(n).is_some_and(|s| s.start < divider));
        above.sort();
        let mut leaks = Vec::new();
        for item in &analysis.items {
            let mut stores = Stores {
                above: &above,
                found: Vec::new(),
            };
            let below = item.syntax.parsed.stmts.iter();
            for stmt in below.filter(|s| s.span.start + item.offset >= divider) {
                walk_stmt(stmt, &mut stores);
            }
            for (name, at) in stores.found {
                let hovers = item
                    .check
                    .hovers
                    .iter()
                    .filter(|h: &&HoverEntry| h.span == at);
                let tys = item
                    .check
                    .shapes
                    .iter()
                    .filter(|(s, _)| *s == at)
                    .map(|(_, t)| t);
                let tys = tys.chain(hovers.map(|h| &h.ty));
                let per_test = tys
                    .flat_map(|t| t.mentioned_domains())
                    .find(|d| analysis.decl_span(d).is_some_and(|s| s.start >= divider));
                if let Some(domain) = per_test {
                    leaks.push((name, domain, at.shift(item.offset)));
                }
            }
        }
        let mut seen = Vec::new();
        for (name, domain, at) in leaks {
            if seen.contains(&name) {
                continue;
            }
            let Some(decl) = analysis.decl_span(&name) else {
                continue;
            };
            let message = format!(
                "`{name}` is declared above `@!multitest`, so a `{domain}` stored in it would outlive its test case"
            );
            let mut diag = Diagnostic::error(codes::FRESH_TYPE_LEAKS_TEST, at, message).with_help(
                format!("declare `{name}` below `@!multitest`, so each test case has its own"),
            );
            diag.labels.push(Label {
                span: decl,
                message: format!("`{name}` is declared here, once for every case"),
            });
            let pos = analysis
                .diagnostics
                .partition_point(|d| d.span.start <= at.start);
            analysis.diagnostics.insert(pos, diag);
            seen.push(name);
        }
    }
}
