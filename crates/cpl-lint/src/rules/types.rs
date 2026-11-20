//! `domain_keyed_map` and `position_arithmetic`: both are about using a
//! domain as if it were a number.

use crate::walk::{unparen, Visitor};
use crate::{File, Finding, DOMAIN_KEYED_MAP, POSITION_ARITHMETIC};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{BinOp, Expr, ExprKind, Stmt, StmtKind, Type, TypeKind};
use std::collections::HashSet;

/// The names of the file's domains, wherever they are declared.
pub(crate) fn domains(file: &File) -> HashSet<String> {
    struct Domains(HashSet<String>);
    impl Visitor for Domains {
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::Domain { name, .. } = &stmt.kind {
                self.0.insert(name.text.clone());
            }
        }
    }
    let mut found = Domains(HashSet::new());
    file.walk(&mut found, |_, _| {});
    found.0
}

struct Types<'a> {
    src: &'a str,
    base: u32,
    domains: HashSet<String>,
    lines: cpl_diag::LineIndex,
    /// The line of the last `domain_keyed_map` finding.
    last_keyed_line: Option<u32>,
    findings: Vec<Finding>,
}

impl Types<'_> {
    /// Reports `Map<K, V>` or `Set<K>` when `K` is a domain.
    fn keyed(&mut self, name: &str, args: &[Type], span: Span) {
        let texts: Vec<&str> = args
            .iter()
            .map(|arg| arg.span.shift(self.base).text(self.src))
            .collect();
        let key = match args.first().map(|arg| &arg.kind) {
            Some(TypeKind::Named { name, args }) if args.is_empty() => name.text.as_str(),
            _ => return,
        };
        if !self.domains.contains(key) {
            return;
        }
        let (written, instead) = match (name, texts.as_slice()) {
            ("Map", [k, v]) => (format!("Map<{k}, {v}>"), format!("[{k}]?{v}")),
            ("Set", [k]) => (format!("Set<{k}>"), format!("[{k}]Bool")),
            _ => return,
        };
        // `var m: Map<V, Int> = Map<V, Int>()` is one finding, not two.
        let line = self.lines.line_of(span.shift(self.base).start);
        if self.last_keyed_line.replace(line) == Some(line) {
            return;
        }
        let diag = Diagnostic::warning(
            DOMAIN_KEYED_MAP.code,
            span.shift(self.base),
            format!("`{written}` is keyed by the domain `{key}`"),
        )
        .with_help(format!(
            "`{instead}` uses one slot per `{key}` and has no lookup cost"
        ));
        self.findings.push(Finding {
            lint: &DOMAIN_KEYED_MAP,
            diag,
        });
    }
}

/// `v.ord + 1` or `1 + v.ord`: the value whose position is taken, and the
/// signed offset as written (`1`, `-2`, `-k`).
fn position_offset<'a>(expr: &'a Expr, src: &str, base: u32) -> Option<(&'a Expr, String)> {
    let ExprKind::Binary { op, lhs, rhs, .. } = &unparen(expr).kind else {
        return None;
    };
    let position = |side: &'a Expr| match &unparen(side).kind {
        ExprKind::Field { base, name } if matches!(name.text.as_str(), "ord" | "idx") => {
            Some(&**base)
        }
        _ => None,
    };
    let text = |side: &Expr| side.span.shift(base).text(src).to_string();
    match op {
        BinOp::Add => position(lhs)
            .map(|value| (value, text(rhs)))
            .or_else(|| position(rhs).map(|value| (value, text(lhs)))),
        BinOp::Sub => {
            let negated = match &unparen(rhs).kind {
                ExprKind::Int(_) | ExprKind::Name(_) => format!("-{}", text(rhs)),
                _ => format!("-({})", text(rhs)),
            };
            position(lhs).map(|value| (value, negated))
        }
        _ => None,
    }
}

impl Visitor for Types<'_> {
    fn ty(&mut self, ty: &Type) {
        if let TypeKind::Named { name, args } = &ty.kind {
            self.keyed(&name.text, args, ty.span);
        }
    }

    fn expr(&mut self, expr: &Expr) {
        match &expr.kind {
            // `Map<V, Int>()` in an expression.
            ExprKind::Generic { base, args } => {
                if let ExprKind::Name(name) = &base.kind {
                    self.keyed(name, args, expr.span);
                }
            }
            // `V(v.ord + 1)`.
            ExprKind::Call { callee, args } => {
                let ExprKind::Name(domain) = &callee.kind else {
                    return;
                };
                let [arg] = args.as_slice() else {
                    return;
                };
                if !self.domains.contains(domain) {
                    return;
                }
                let Some((value, offset)) = position_offset(&arg.value, self.src, self.base) else {
                    return;
                };
                let value = value.span.shift(self.base).text(self.src);
                let written = expr.span.shift(self.base).text(self.src);
                let step = match offset.as_str() {
                    "1" => format!("`{value}.next` is the `{domain}` after `{value}`"),
                    "-1" => format!("`{value}.prev` is the `{domain}` before `{value}`"),
                    _ => format!("`{value}.offset({offset})` moves `{value}` by `{offset}`"),
                };
                let diag = Diagnostic::warning(
                    POSITION_ARITHMETIC.code,
                    expr.span.shift(self.base),
                    format!("`{written}` builds a `{domain}` from a position"),
                )
                .with_note(format!(
                    "`.ord` counts from 0, and `{domain}(..)` takes a label: they differ unless `{domain}` starts at 0"
                ))
                .with_help(format!(
                    "{step} (a `?{domain}`: `none` past the end); `.cyc(..)` wraps around"
                ));
                self.findings.push(Finding {
                    lint: &POSITION_ARITHMETIC,
                    diag,
                });
            }
            _ => {}
        }
    }
}

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut types = Types {
        src: file.src,
        base: 0,
        domains: domains(file),
        lines: cpl_diag::LineIndex::new(file.src),
        last_keyed_line: None,
        findings: Vec::new(),
    };
    file.walk(&mut types, |types, base| types.base = base);
    types.findings
}
