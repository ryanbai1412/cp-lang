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
        /// `@test` functions: their literals are test data, not facts the
        /// program keeps.
        tests: Vec<Span>,
        /// Values written as the right of a `%`: a modulus, named `P`.
        moduli: Vec<i128>,
    }
    impl Visitor for Ints {
        fn stmt(&mut self, stmt: &Stmt) {
            if stmt.decorators.iter().any(|d| d.name.text == "test") {
                self.tests.push(stmt.span.shift(self.base));
            }
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
            if let ExprKind::Binary {
                op: BinOp::Rem,
                rhs,
                ..
            } = &expr.kind
            {
                if let ExprKind::Int(v) = crate::walk::unparen(rhs).kind {
                    self.moduli.push(v);
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
        tests: Vec::new(),
        moduli: Vec::new(),
    };
    file.walk(&mut ints, |i, base| i.base = base);
    let skipped: Vec<Span> = ints.input_types.into_iter().chain(ints.tests).collect();
    for spans in ints.seen.values_mut() {
        spans.retain(|s| !skipped.iter().any(|t| t.start <= s.start && s.end <= t.end));
    }
    let lines = cpl_diag::LineIndex::new(file.src);
    let mut out = Vec::new();
    for (value, mut spans) in ints.seen {
        // A big value is a fact each time it is written, but `-1e9..=1e9`
        // is one: a range's two ends count once. A small one counts by line:
        // `m + 1000, q - 1000` on one line is arithmetic, not a fact.
        spans.sort_by_key(|s| s.start);
        let big = value >= 1_000_000;
        spans.dedup_by(|hi, lo| {
            let between = file.src.get(lo.end as usize..hi.start as usize);
            let range_ends = between
                .is_some_and(|t| matches!(t.trim().trim_end_matches('-').trim(), ".." | "..="));
            range_ends || (!big && lines.line_col(hi.start).line == lines.line_col(lo.start).line)
        });
        let need = if big { 2 } else { 3 };
        if spans.len() < need {
            continue;
        }
        let line = lines.line_col(spans[0].start).line + 1;
        let span = spans[1];
        let text = span.text(file.src);
        let named = if ints.moduli.contains(&value) {
            "P"
        } else {
            "LIM"
        };
        let diag = Diagnostic::warning(
            REPEATED_LITERAL.code,
            span,
            format!(
                "`{text}` is written {} {}; it is first on line {line}",
                spans.len(),
                if big { "times" } else { "lines" }
            ),
        )
        .with_help(format!(
            "name it once (`const {named} = {text}`, a domain, or a type alias) and use the name: copies of one fact can drift apart"
        ));
        out.push(Finding {
            lint: &REPEATED_LITERAL,
            diag,
        });
    }
    out
}
