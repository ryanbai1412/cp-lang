//! `float_equality`: `==` and `!=` on floats compare every bit, and two
//! ways of computing the same number rarely agree to the last one.
//!
//! There are no types here, so "is a float" is read off the source: a
//! float literal, a binding declared `F64` or set to a float, a fn declared
//! to return `Float`, a prelude fn that always does, and arithmetic on any
//! of those.

use crate::walk::{Bind, Scope, Visitor};
use crate::{File, Finding, FLOAT_EQUALITY};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{CmpOp, Expr, ExprKind, PatternKind, Stmt, StmtKind, Type, TypeKind, UnOp};
use std::collections::{HashMap, HashSet};

/// Prelude fns whose result is a float whatever they are given.
const FLOAT_FNS: &[&str] = &[
    "sqrt", "cbrt", "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "exp", "ln", "log",
    "log2", "log10", "hypot", "float", "F64",
];

fn is_float_type(ty: &Type) -> bool {
    matches!(&ty.kind, TypeKind::Named { name, args } if name.text == "F64" && args.is_empty())
}

/// The bindings and fns of a file that are floats.
#[derive(Default)]
struct Floats {
    /// Where each float `let`, `var`, `const` or parameter is bound, as an
    /// offset in the file.
    bindings: HashSet<u32>,
    /// Float consts by name, for a use before the declaration.
    consts: HashSet<String>,
    fns: HashSet<String>,
    fields: HashSet<String>,
    /// Every fn the file declares: a declared `exp` is not the prelude's.
    declared: HashSet<String>,
}

/// Which binding each name in scope is, innermost last.
struct Scopes(Vec<HashMap<String, u32>>);

impl Default for Scopes {
    fn default() -> Scopes {
        Scopes(vec![HashMap::new()])
    }
}

impl Scopes {
    fn bind(&mut self, name: &str, at: u32) {
        if let Some(top) = self.0.last_mut() {
            top.insert(name.to_string(), at);
        }
    }

    fn resolve(&self, name: &str) -> Option<u32> {
        self.0
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn enter(&mut self) {
        self.0.push(HashMap::new());
    }

    fn exit(&mut self) {
        if self.0.len() > 1 {
            self.0.pop();
        }
    }
}

impl Floats {
    fn is_float(&self, expr: &Expr, scopes: &Scopes) -> bool {
        match &expr.kind {
            ExprKind::Float(_) => true,
            ExprKind::Name(name) => match scopes.resolve(name) {
                Some(at) => self.bindings.contains(&at),
                None => self.consts.contains(name),
            },
            ExprKind::Paren(inner) | ExprKind::Unwrap(inner) => self.is_float(inner, scopes),
            ExprKind::Unary {
                op: UnOp::Neg,
                expr,
            } => self.is_float(expr, scopes),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.is_float(lhs, scopes) || self.is_float(rhs, scopes)
            }
            ExprKind::Field { name, .. } => self.fields.contains(&name.text),
            ExprKind::Call { callee, .. } => match &callee.kind {
                ExprKind::Name(name) if self.declared.contains(name) => self.fns.contains(name),
                ExprKind::Name(name) => FLOAT_FNS.contains(&name.as_str()),
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
    base: u32,
    scopes: Scopes,
    /// The statement being bound is a float.
    binding_float: bool,
    /// Where the float parameters of the fns seen so far are bound.
    float_params: HashSet<u32>,
}

impl Visitor for Collect<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        self.binding_float = false;
        match &stmt.kind {
            StmtKind::Let { pat, ty, init, .. } => {
                let annotated = ty.as_ref().is_some_and(is_float_type);
                let inferred = ty.is_none()
                    && init
                        .as_ref()
                        .is_some_and(|init| self.floats.is_float(init, &self.scopes));
                // `let (a, b) = ..` says nothing about either alone.
                self.binding_float =
                    (annotated || inferred) && matches!(pat.kind, PatternKind::Name(_));
            }
            StmtKind::Const { name, ty, init, .. } => {
                let float = match ty {
                    Some(ty) => is_float_type(ty),
                    None => init
                        .as_ref()
                        .is_some_and(|init| self.floats.is_float(init, &self.scopes)),
                };
                if float {
                    self.grew |= self.floats.consts.insert(name.text.clone());
                    self.binding_float = true;
                }
            }
            StmtKind::Fn(decl) => {
                if decl.ret.as_ref().is_some_and(is_float_type) {
                    self.grew |= self.floats.fns.insert(decl.name.text.clone());
                }
                for param in &decl.params {
                    if param.ty.as_ref().is_some_and(is_float_type) {
                        self.float_params.insert(self.base + param.name.span.start);
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

    fn bind(&mut self, name: &str, span: Span, how: Bind) {
        let at = self.base + span.start;
        self.scopes.bind(name, at);
        let float = match how {
            Bind::Let | Bind::Var | Bind::Const => std::mem::take(&mut self.binding_float),
            Bind::Param => self.float_params.contains(&at),
            Bind::Pattern => false,
        };
        if float {
            self.grew |= self.floats.bindings.insert(at);
        }
    }

    fn enter(&mut self, _scope: Scope) {
        self.scopes.enter();
    }

    fn exit(&mut self, _scope: Scope) {
        self.scopes.exit();
    }
}

/// Collects the fns a file declares.
struct Declared<'a>(&'a mut HashSet<String>);

impl Visitor for Declared<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Fn(decl) = &stmt.kind {
            self.0.insert(decl.name.text.clone());
        }
    }
}

struct Report<'a> {
    src: &'a str,
    base: u32,
    scopes: Scopes,
    floats: &'a Floats,
    findings: Vec<Finding>,
}

impl Visitor for Report<'_> {
    fn bind(&mut self, name: &str, span: Span, _how: Bind) {
        self.scopes.bind(name, self.base + span.start);
    }

    fn enter(&mut self, _scope: Scope) {
        self.scopes.enter();
    }

    fn exit(&mut self, _scope: Scope) {
        self.scopes.exit();
    }

    fn expr(&mut self, expr: &Expr) {
        let ExprKind::Compare { first, rest } = &expr.kind else {
            return;
        };
        let mut lhs: &Expr = first;
        for (op, rhs) in rest {
            if matches!(op, CmpOp::Eq | CmpOp::Ne)
                && (self.floats.is_float(lhs, &self.scopes)
                    || self.floats.is_float(rhs, &self.scopes))
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
    file.walk(&mut Declared(&mut floats.declared), |_, _| {});
    loop {
        let mut collect = Collect {
            floats: &mut floats,
            grew: false,
            base: 0,
            scopes: Scopes::default(),
            binding_float: false,
            float_params: HashSet::new(),
        };
        file.walk(&mut collect, |collect, base| collect.base = base);
        if !collect.grew {
            break;
        }
    }
    let mut report = Report {
        src: file.src,
        base: 0,
        scopes: Scopes::default(),
        floats: &floats,
        findings: Vec::new(),
    };
    file.walk(&mut report, |report, base| report.base = base);
    report.findings
}
