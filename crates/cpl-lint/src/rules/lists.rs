//! `unproven_list_index`: `xs[i]` on a `List` is bounds-checked on every
//! access unless the compiler can see `i` is in range. Inside two or more
//! nested loops that check is paid millions of times.
//!
//! An index is proven when it is the variable of an enclosing
//! `for i in 0..xs.len` or `for i in xs.indices` over the same list.

use crate::walk::{unparen, Bind, Scope, Visitor};
use crate::{File, Finding, UNPROVEN_LIST_INDEX};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{Expr, ExprKind, Pattern, PatternKind, Stmt, StmtKind, Type, TypeKind};
use std::collections::HashSet;

/// How many loops deep an index has to be before it is worth a warning.
const HOT_DEPTH: usize = 2;

fn is_list_type(ty: &Type) -> bool {
    matches!(&ty.kind, TypeKind::Named { name, .. } if name.text == "List")
}

/// `[]`, `[1, 2]`, `List<Int>()`.
fn is_list_value(expr: &Expr) -> bool {
    match &unparen(expr).kind {
        ExprKind::List(_) => true,
        ExprKind::Call { callee, .. } => match &callee.kind {
            ExprKind::Generic { base, .. } => {
                matches!(&base.kind, ExprKind::Name(n) if n == "List")
            }
            ExprKind::Name(name) => name == "List",
            _ => false,
        },
        _ => false,
    }
}

/// The names declared as a `List`, by annotation or by what they are set to.
struct Lists {
    names: HashSet<String>,
    binding_list: bool,
}

impl Visitor for Lists {
    fn stmt(&mut self, stmt: &Stmt) {
        self.binding_list = false;
        match &stmt.kind {
            StmtKind::Let { pat, ty, init, .. } => {
                let list = match ty {
                    Some(ty) => is_list_type(ty),
                    None => init.as_ref().is_some_and(is_list_value),
                };
                self.binding_list = list && matches!(pat.kind, PatternKind::Name(_));
            }
            StmtKind::Fn(decl) => {
                for param in &decl.params {
                    if param.ty.as_ref().is_some_and(is_list_type) {
                        self.names.insert(param.name.text.clone());
                    }
                }
            }
            _ => {}
        }
    }

    fn bind(&mut self, name: &str, _span: Span, how: Bind) {
        if self.binding_list && matches!(how, Bind::Let | Bind::Var) {
            self.names.insert(name.to_string());
            self.binding_list = false;
        }
    }
}

struct Indexes<'a> {
    src: &'a str,
    base: u32,
    lists: &'a HashSet<String>,
    /// One entry per open scope: whether it is a loop, and the
    /// `(index variable, list)` pair it proves in range.
    scopes: Vec<(Scope, Option<(String, String)>)>,
    findings: Vec<Finding>,
}

impl Indexes<'_> {
    /// Loops open since the enclosing fn began.
    fn loop_depth(&self) -> usize {
        self.scopes
            .iter()
            .rev()
            .take_while(|(scope, _)| *scope != Scope::Fn)
            .filter(|(scope, _)| *scope == Scope::Loop)
            .count()
    }

    fn proven(&self, index: &str, list: &str) -> bool {
        self.scopes
            .iter()
            .any(|(_, fact)| matches!(fact, Some((i, xs)) if i == index && xs == list))
    }
}

/// The list `iter` walks the indices of: `0..xs.len`, `xs.indices`.
fn indexed_list(iter: &Expr) -> Option<&str> {
    /// `xs` in `xs.field`.
    fn receiver<'a>(expr: &'a Expr, field: &str) -> Option<&'a str> {
        match &unparen(expr).kind {
            ExprKind::Field { base, name } if name.text == field => match &base.kind {
                ExprKind::Name(list) => Some(list),
                _ => None,
            },
            _ => None,
        }
    }
    match &unparen(iter).kind {
        ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive: false,
            step: None,
        } if matches!(lo.kind, ExprKind::Int(0)) => receiver(hi, "len"),
        _ => receiver(iter, "indices"),
    }
}

impl Visitor for Indexes<'_> {
    fn enter(&mut self, scope: Scope) {
        self.scopes.push((scope, None));
    }

    fn exit(&mut self, _scope: Scope) {
        self.scopes.pop();
    }

    fn for_loop(&mut self, pat: &Pattern, iter: &Expr) {
        if let (PatternKind::Name(index), Some(list)) = (&pat.kind, indexed_list(iter)) {
            if let Some(top) = self.scopes.last_mut() {
                top.1 = Some((index.clone(), list.to_string()));
            }
        }
    }

    fn expr(&mut self, expr: &Expr) {
        let ExprKind::Index { base, indices } = &expr.kind else {
            return;
        };
        let (ExprKind::Name(list), [index]) = (&base.kind, indices.as_slice()) else {
            return;
        };
        if !self.lists.contains(list) || self.loop_depth() < HOT_DEPTH {
            return;
        }
        match &unparen(index).kind {
            ExprKind::Name(name) if self.proven(name, list) => return,
            // A slice, not an element.
            ExprKind::Range { .. } => return,
            _ => {}
        }
        let written = index.span.shift(self.base).text(self.src);
        let diag = Diagnostic::warning(
            UNPROVEN_LIST_INDEX.code,
            expr.span.shift(self.base),
            format!("`{list}[{written}]` is bounds-checked on every pass of a nested loop"),
        )
        .with_help(format!(
            "loop over the values (`for x in {list}`) or the indices (`for i in {list}.indices`), or index an array by a domain: `[D]T` needs no check"
        ));
        self.findings.push(Finding {
            lint: &UNPROVEN_LIST_INDEX,
            diag,
        });
    }
}

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut lists = Lists {
        names: HashSet::new(),
        binding_list: false,
    };
    file.walk(&mut lists, |_, _| {});
    let mut indexes = Indexes {
        src: file.src,
        base: 0,
        lists: &lists.names,
        scopes: Vec::new(),
        findings: Vec::new(),
    };
    file.walk(&mut indexes, |indexes, base| indexes.base = base);
    indexes.findings
}
