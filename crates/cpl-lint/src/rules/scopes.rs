//! `var_never_reassigned` and `shadows_top_level`: both need to know which
//! binding a name refers to.

use crate::walk::{root_name, Bind, Scope, Visitor};
use crate::{File, Finding, SHADOWS_TOP_LEVEL, VAR_NEVER_REASSIGNED};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{Expr, ExprKind, Stmt, StmtKind};
use std::collections::HashMap;

struct Binding {
    /// The `var` statement that made it, for a `var`.
    var: Option<usize>,
    /// Where the name is written, in the file.
    span: Span,
    top_level: bool,
}

/// One `var` statement. `var (a, b) = ..` is one statement with two names,
/// and stays a `var` if either changes.
struct VarStmt {
    /// The `var` keyword.
    keyword: Span,
    names: Vec<String>,
    changed: bool,
}

struct Scopes<'a> {
    src: &'a str,
    base: u32,
    bindings: Vec<Binding>,
    /// Innermost last; the first is the top level.
    scopes: Vec<HashMap<String, usize>>,
    vars: Vec<VarStmt>,
    /// The `var` statement whose pattern is being bound.
    current_var: Option<usize>,
    /// How many `fn` bodies enclose the walk.
    fns: usize,
    findings: Vec<Finding>,
}

impl Scopes<'_> {
    fn resolve(&self, name: &str) -> Option<&Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
            .map(|id| &self.bindings[*id])
    }

    /// Records that the variable `expr` reads through may change.
    fn changes(&mut self, expr: &Expr) {
        // `(x, y) = (y, x % y)` changes every place it lists.
        if let ExprKind::Tuple(places) = &expr.kind {
            for place in places {
                self.changes(place);
            }
            return;
        }
        let var = root_name(expr).and_then(|name| self.resolve(name)?.var);
        if let Some(var) = var {
            self.vars[var].changed = true;
        }
    }
}

impl Visitor for Scopes<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let { mutable: true, .. } => {
                // The statement's span starts at its decorators, if any.
                let text = stmt.span.text_at(self.src, self.base);
                let at = text.find("var").unwrap_or(0) as u32 + stmt.span.start + self.base;
                self.vars.push(VarStmt {
                    keyword: Span::new(at, at + 3),
                    names: Vec::new(),
                    changed: false,
                });
                self.current_var = Some(self.vars.len() - 1);
            }
            StmtKind::Assign { target, .. } => self.changes(target),
            _ => {}
        }
    }

    fn expr(&mut self, expr: &Expr) {
        let ExprKind::Call { callee, args } = &expr.kind else {
            return;
        };
        for arg in args.iter().filter(|arg| arg.mutable) {
            self.changes(&arg.value);
        }
        // `xs.push(1)`: without types, any method may change its receiver.
        if let ExprKind::Field { base, .. } = &callee.kind {
            self.changes(base);
        }
    }

    fn bind(&mut self, name: &str, span: Span, how: Bind) {
        let span = span.shift(self.base);
        let top_level = self.scopes.len() == 1;
        let var = if how == Bind::Var {
            self.current_var
        } else {
            None
        };
        if let Some(var) = var {
            self.vars[var].names.push(name.to_string());
        }
        if !top_level && name != "_" {
            let hidden = self.scopes[0].get(name).map(|id| &self.bindings[*id]);
            // A fn can name a top-level `let` written below it, so there a
            // later one is hidden too. Top-level code runs in order: a loop
            // that ends before `var i` is written hides nothing.
            let in_reach = |b: &&Binding| self.fns > 0 || b.span.start < span.start;
            if let Some(hidden) = hidden.filter(|b| b.top_level).filter(in_reach) {
                let what = match how {
                    Bind::Param => "parameter",
                    Bind::Pattern => "binding",
                    Bind::Let | Bind::Var | Bind::Const => "local",
                };
                let line = cpl_diag::LineIndex::new(self.src)
                    .line_col(hidden.span.start)
                    .line
                    + 1;
                let diag = Diagnostic::warning(
                    SHADOWS_TOP_LEVEL.code,
                    span,
                    format!("this {what} hides the top-level `{name}`"),
                )
                .with_label(hidden.span, format!("`{name}` is defined here, on line {line}"))
                .with_help(format!(
                    "from here on `{name}` means the {what}; give it another name if the top-level one is meant below"
                ));
                self.findings.push(Finding {
                    lint: &SHADOWS_TOP_LEVEL,
                    diag,
                });
            }
        }
        // The top level was bound in full before the walk, so that a fn can
        // name a `let` written below it.
        if top_level {
            if let Some(id) = self.scopes[0].get(name) {
                self.bindings[*id].var = var;
                return;
            }
        }
        self.bindings.push(Binding {
            var,
            span,
            top_level,
        });
        let id = self.bindings.len() - 1;
        self.scopes.last_mut().unwrap().insert(name.to_string(), id);
    }

    fn enter(&mut self, scope: Scope) {
        self.fns += usize::from(scope == Scope::Fn);
        self.scopes.push(HashMap::new());
    }

    fn exit(&mut self, scope: Scope) {
        self.fns -= usize::from(scope == Scope::Fn);
        self.scopes.pop();
    }
}

/// Binds the names of the top-level `let`, `var` and `const` statements.
struct TopLevel<'a, 'b> {
    scopes: &'a mut Scopes<'b>,
    depth: usize,
}

impl Visitor for TopLevel<'_, '_> {
    fn bind(&mut self, name: &str, span: Span, how: Bind) {
        if self.depth == 0 && matches!(how, Bind::Let | Bind::Var | Bind::Const) {
            let span = span.shift(self.scopes.base);
            self.scopes.bindings.push(Binding {
                var: None,
                span,
                top_level: true,
            });
            let id = self.scopes.bindings.len() - 1;
            // The first definition is the one a later local hides.
            self.scopes.scopes[0].entry(name.to_string()).or_insert(id);
        }
    }

    fn enter(&mut self, _scope: Scope) {
        self.depth += 1;
    }

    fn exit(&mut self, _scope: Scope) {
        self.depth -= 1;
    }
}

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut scopes = Scopes {
        src: file.src,
        base: 0,
        bindings: Vec::new(),
        scopes: vec![HashMap::new()],
        vars: Vec::new(),
        current_var: None,
        fns: 0,
        findings: Vec::new(),
    };
    let mut top = TopLevel {
        scopes: &mut scopes,
        depth: 0,
    };
    file.walk(&mut top, |top, base| top.scopes.base = base);
    file.walk(&mut scopes, |scopes, base| scopes.base = base);

    let Scopes {
        vars, mut findings, ..
    } = scopes;
    for var in vars
        .iter()
        .filter(|var| !var.changed && !var.names.is_empty())
    {
        let names: Vec<String> = var.names.iter().map(|name| format!("`{name}`")).collect();
        let (subject, verb) = match names.as_slice() {
            [one] => (one.clone(), "is"),
            many => (many.join(" and "), "are"),
        };
        let diag = Diagnostic::warning(
            VAR_NEVER_REASSIGNED.code,
            var.keyword,
            format!("{subject} {verb} declared `var` but never changed"),
        )
        .with_help("declare it with `let`, so a later assignment by mistake is an error")
        .with_fix("change `var` to `let`", var.keyword, "let");
        findings.push(Finding {
            lint: &VAR_NEVER_REASSIGNED,
            diag,
        });
    }
    findings
}

/// The text of an item-relative span.
trait TextAt {
    fn text_at(self, src: &str, base: u32) -> &str;
}

impl TextAt for Span {
    fn text_at(self, src: &str, base: u32) -> &str {
        self.shift(base).text(src)
    }
}
