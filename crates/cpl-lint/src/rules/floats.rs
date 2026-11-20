//! `float_equality`: `==` and `!=` on floats compare every bit, and two
//! ways of computing the same number rarely agree to the last one.
//!
//! There are no types here, so "is a float" is read off the source: a
//! float literal, a name declared `Float` or set to a float, a fn declared
//! to return `Float`, a prelude fn that always does, and arithmetic on any
//! of those.

use crate::walk::{Bind, Visitor};
use crate::{File, Finding, FLOAT_EQUALITY};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{CmpOp, Expr, ExprKind, PatternKind, Stmt, StmtKind, Type, TypeKind, UnOp};
use std::collections::HashSet;

/// Prelude fns whose result is a float whatever they are given.
const FLOAT_FNS: &[&str] = &[
    "sqrt", "cbrt", "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "exp", "ln", "log",
    "log2", "log10", "hypot", "float", "Float",
];

fn is_float_type(ty: &Type) -> bool {
    matches!(&ty.kind, TypeKind::Named { name, args } if name.text == "Float" && args.is_empty())
}

/// The names and fns of a file that are floats.
#[derive(Default)]
struct Floats {
    names: HashSet<String>,
    fns: HashSet<String>,
    fields: HashSet<String>,
}

impl Floats {
    fn is_float(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Float(_) => true,
            ExprKind::Name(name) => self.names.contains(name),
            ExprKind::Paren(inner) | ExprKind::Unwrap(inner) => self.is_float(inner),
            ExprKind::Unary {
                op: UnOp::Neg,
                expr,
            } => self.is_float(expr),
            ExprKind::Binary { lhs, rhs, .. } => self.is_float(lhs) || self.is_float(rhs),
            ExprKind::Field { name, .. } => self.fields.contains(&name.text),
            ExprKind::Call { callee, .. } => match &callee.kind {
                ExprKind::Name(name) => {
                    FLOAT_FNS.contains(&name.as_str()) || self.fns.contains(name)
                }
                _ => false,
            },
            _ => false,
        }
    }
}

/// Finds what is a float. Run until it finds nothing new, so that
/// `let half = total / 2.0` then `let third = half * x` are both seen.
struct Collect<'a> {
    floats: &'a mut Floats,
    grew: bool,
    /// The statement being bound is a float.
    binding_float: bool,
}

impl Visitor for Collect<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        self.binding_float = false;
        match &stmt.kind {
            StmtKind::Let { pat, ty, init, .. } => {
                let annotated = ty.as_ref().is_some_and(is_float_type);
                let inferred =
                    ty.is_none() && init.as_ref().is_some_and(|init| self.floats.is_float(init));
                // `let (a, b) = ..` says nothing about either alone.
                self.binding_float =
                    (annotated || inferred) && matches!(pat.kind, PatternKind::Name(_));
            }
            StmtKind::Const { name, ty, init, .. } => {
                let float = match ty {
                    Some(ty) => is_float_type(ty),
                    None => init.as_ref().is_some_and(|init| self.floats.is_float(init)),
                };
                if float {
                    self.grew |= self.floats.names.insert(name.text.clone());
                }
            }
            StmtKind::Fn(decl) => {
                if decl.ret.as_ref().is_some_and(is_float_type) {
                    self.grew |= self.floats.fns.insert(decl.name.text.clone());
                }
                for param in &decl.params {
                    if param.ty.as_ref().is_some_and(is_float_type) {
                        self.grew |= self.floats.names.insert(param.name.text.clone());
                    }
                }
            }
            StmtKind::Struct(decl) => {
                for field in &decl.fields {
                    if is_float_type(&field.ty) {
                        self.grew |= self.floats.fields.insert(field.name.text.clone());
                    }
                }
            }
            _ => {}
        }
    }

    fn bind(&mut self, name: &str, _span: Span, how: Bind) {
        if self.binding_float && matches!(how, Bind::Let | Bind::Var) {
            self.grew |= self.floats.names.insert(name.to_string());
            self.binding_float = false;
        }
    }
}

struct Report<'a> {
    src: &'a str,
    base: u32,
    floats: &'a Floats,
    findings: Vec<Finding>,
}

impl Visitor for Report<'_> {
    fn expr(&mut self, expr: &Expr) {
        let ExprKind::Compare { first, rest } = &expr.kind else {
            return;
        };
        let mut lhs: &Expr = first;
        for (op, rhs) in rest {
            if matches!(op, CmpOp::Eq | CmpOp::Ne)
                && (self.floats.is_float(lhs) || self.floats.is_float(rhs))
            {
                let text = |e: &Expr| e.span.shift(self.base).text(self.src);
                let (a, b) = (text(lhs), text(rhs));
                let (symbol, instead) = match op {
                    CmpOp::Eq => ("==", format!("abs({a} - {b}) < EPS")),
                    _ => ("!=", format!("abs({a} - {b}) >= EPS")),
                };
                let diag = Diagnostic::warning(
                    FLOAT_EQUALITY.code,
                    lhs.span.to(rhs.span).shift(self.base),
                    format!("`{symbol}` compares floats bit for bit"),
                )
                .with_help(format!(
                    "compare within a tolerance: `{instead}`, with `EPS` chosen for the answer's precision"
                ));
                self.findings.push(Finding {
                    lint: &FLOAT_EQUALITY,
                    diag,
                });
            }
            lhs = rhs;
        }
    }
}

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut floats = Floats::default();
    loop {
        let mut collect = Collect {
            floats: &mut floats,
            grew: false,
            binding_float: false,
        };
        file.walk(&mut collect, |_, _| {});
        if !collect.grew {
            break;
        }
    }
    let mut report = Report {
        src: file.src,
        base: 0,
        floats: &floats,
        findings: Vec::new(),
    };
    file.walk(&mut report, |report, base| report.base = base);
    report.findings
}
