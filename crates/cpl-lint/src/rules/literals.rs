//! Repeated literals: a big bound or modulus written on several lines is
//! facts that must agree but are kept apart (L0506).

use crate::walk::Visitor;
use crate::{File, Finding, REPEATED_LITERAL};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::*;
use std::collections::HashMap;

/// Big literals: a bound or modulus written more than once is two facts
/// that must agree.
pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Ints {
        base: u32,
        seen: HashMap<i128, Vec<Span>>,
        /// The types of `let x: T = read()` and `read<T>()`, and the field
        /// types of records read as input: each is the statement's bound for
        /// its own input, so equal bounds there are separate facts.
        input_types: Vec<Span>,
    }
    impl Visitor for Ints {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::Struct(decl) => {
                    for field in &decl.fields {
                        self.input_types.push(field.ty.span.shift(self.base));
                    }
                }
                StmtKind::Enum(decl) => {
                    for (_, ty) in decl.variants.iter().flat_map(|v| &v.fields) {
                        self.input_types.push(ty.span.shift(self.base));
                    }
                }
                _ => {}
            }
            if let StmtKind::Let {
                ty: Some(ty),
                init: Some(init),
                ..
            } = &stmt.kind
            {
                if let ExprKind::Call { callee, .. } = &init.kind {
                    if matches!(&callee.kind, ExprKind::Name(n) if n == "read") {
                        self.input_types.push(ty.span.shift(self.base));
                    }
                }
            }
        }
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Generic { base, args } = &expr.kind {
                if matches!(&base.kind, ExprKind::Name(n) if n == "read") {
                    let types = args.iter().map(|t| t.span.shift(self.base));
                    self.input_types.extend(types);
                }
            }
            if let ExprKind::Int(v) = expr.kind {
                if v >= 1000 {
                    self.seen
                        .entry(v)
                        .or_default()
                        .push(expr.span.shift(self.base));
                }
            }
        }
    }
    let mut ints = Ints {
        base: 0,
        seen: HashMap::new(),
        input_types: Vec::new(),
    };
    file.walk(&mut ints, |i, base| i.base = base);
    let input_types = ints.input_types;
    for spans in ints.seen.values_mut() {
        spans.retain(|s| {
            !input_types
                .iter()
                .any(|t| t.start <= s.start && s.end <= t.end)
        });
    }
    let lines = cpl_diag::LineIndex::new(file.src);
    let mut out = Vec::new();
    for (value, mut spans) in ints.seen {
        // `-1e9..=1e9` is one fact: count lines, not occurrences.
        spans.sort_by_key(|s| s.start);
        spans.dedup_by_key(|s| lines.line_col(s.start).line);
        let need = if value >= 1_000_000 { 2 } else { 3 };
        if spans.len() < need {
            continue;
        }
        let line = lines.line_col(spans[0].start).line + 1;
        let span = spans[1];
        let text = span.text(file.src);
        let diag = Diagnostic::warning(
            REPEATED_LITERAL.code,
            span,
            format!(
                "`{text}` is written on {} lines; it is first on line {line}",
                spans.len()
            ),
        )
        .with_help(format!(
            "name it once (`const P = {text}`, a domain, or a type alias) and use the name"
        ));
        out.push(Finding {
            lint: &REPEATED_LITERAL,
            diag,
        });
    }
    out
}
