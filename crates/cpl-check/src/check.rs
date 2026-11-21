//! The checker for one item: names, types, domains, and `mut`.
//!
//! It never reports twice for one root cause. Anything it can't type is
//! `Unknown`, anything already reported is `Error`, and both are compatible
//! with every type. It also never reports inside a span the parser
//! poisoned.

mod call;
mod expr;
pub(crate) mod lends;
mod stmt;
mod types;

pub(crate) use types::{domain_binding, memo_table, own_domains, type_head};

use crate::def::Def;
use crate::prelude;
use crate::ty::Ty;
use cpl_diag::{codes, Code, Diagnostic, Sink, Span};
use cpl_syntax::narrow;
use std::collections::HashMap;
use std::sync::Arc;

/// How the checker sees the rest of the file. Every answer is recorded as
/// a dependency of the item being checked.
pub(crate) trait Env {
    /// The definition of a top-level name.
    fn global(&mut self, name: &str) -> Option<Arc<Def>>;
    /// The struct or enum behind the type of a value, which may come from
    /// a module this file reaches without importing the type by name.
    fn type_def(&mut self, name: &str) -> Option<Arc<Def>>;
    /// True if `alias` names a module imported whole: `use lib.treap`.
    fn is_module(&mut self, alias: &str) -> bool;
    /// What calling the top-level fn `name` touches outside itself,
    /// counting the fns it calls.
    fn effects(&mut self, name: &str) -> Arc<lends::Effects>;
    /// The earlier top-level statements whose lends and facts reach the
    /// item being checked.
    fn lent_before(&mut self) -> Vec<lends::Replay>;
    /// Every impl for the type `target`, with the trait it implements
    /// (`None` for `impl Target:`).
    fn impls(&mut self, target: &str) -> Vec<(Option<String>, Arc<Def>)>;
    /// True if the file imports modules. They are not loaded yet, so names
    /// that are not found may come from them.
    fn has_imports(&mut self) -> bool;
    /// Every top-level name, for "did you mean".
    fn names(&mut self) -> Vec<String>;
}

#[derive(Clone, Debug)]
pub(crate) struct Local {
    pub ty: Ty,
    pub mutable: bool,
    pub span: Span,
    pub keyword: &'static str,
}

/// What hovering a span shows. Spans are relative to the item.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct HoverEntry {
    pub span: Span,
    pub name: String,
    pub ty: Ty,
    /// `let`, `var`, `param`, `for`, `fn`, `field`; or `global` / `type`
    /// for a top-level name, which is looked up again when shown.
    pub keyword: &'static str,
    /// Where the name is declared, if that is in this item.
    pub decl: Option<Span>,
    /// For a `field`, the type it is a member of: `Fenwick` for `bit.add`.
    pub owner: Option<String>,
}

/// A use of a top-level name.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GlobalRef {
    pub span: Span,
    pub name: String,
    /// True if the use runs as part of the file's top-level code, not
    /// inside a fn body.
    pub top_level: bool,
}

pub(crate) struct Checker<'a> {
    pub(crate) env: &'a mut dyn Env,
    /// The item's text; spans index into it.
    src: &'a str,
    sink: Sink,
    scopes: Vec<HashMap<String, Local>>,
    generics: Vec<String>,
    hovers: Vec<HoverEntry>,
    /// The optionals a test has shown to hold a value here, by name.
    present: Vec<String>,
    /// The name about to be read stands where an optional is asked for.
    as_written: bool,
    /// What the fns declared inside the item being checked assign.
    mutators: narrow::Mutators,
    /// Where a step along a domain stays inside it, in the item being
    /// checked.
    steps: cpl_syntax::order::Steps,
    refs: Vec<GlobalRef>,
    /// Name, return type, whether declared, and scope depth at fn entry.
    fns: Vec<(String, Ty, bool, usize)>,
    /// For each of `fns`: whether its body has given a value so far.
    gives: Vec<bool>,
    /// Inside a fn signature or body, where a single capital letter is a
    /// type parameter without being declared.
    in_fn: u32,
    /// How many branches, lambdas and comprehensions the expression being
    /// checked is inside, within its statement: `e?` can't leave from there.
    branch: u32,
    /// The `if` or `match` that is a statement of its own, not a value.
    plain_if: Option<Span>,
    /// What `Self` is: the struct, enum or impl target whose body this is.
    self_ty: Option<Ty>,
    /// What is lent and what is known, for the exclusivity of lends.
    lends: lends::Lends,
}

pub(crate) struct Checked {
    pub diagnostics: Vec<Diagnostic>,
    pub hovers: Vec<HoverEntry>,
    pub refs: Vec<GlobalRef>,
}

impl<'a> Checker<'a> {
    pub(crate) fn new(env: &'a mut dyn Env, src: &'a str, poisoned: Vec<Span>) -> Checker<'a> {
        Checker {
            env,
            src,
            sink: Sink::with_poisoned(poisoned),
            scopes: vec![HashMap::new()],
            generics: Vec::new(),
            hovers: Vec::new(),
            present: Vec::new(),
            as_written: false,
            mutators: narrow::Mutators::default(),
            steps: Default::default(),
            refs: Vec::new(),
            fns: Vec::new(),
            gives: Vec::new(),
            in_fn: 0,
            branch: 0,
            plain_if: None,
            self_ty: None,
            lends: lends::Lends::default(),
        }
    }

    pub(crate) fn finish(self) -> Checked {
        Checked {
            diagnostics: self.sink.finish(),
            hovers: self.hovers,
            refs: self.refs,
        }
    }

    fn text(&self, span: Span) -> &'a str {
        self.src
            .get(span.start as usize..span.end as usize)
            .unwrap_or("")
    }

    /// Source text for a message: one line, shortened if long.
    fn quote(&self, span: Span) -> String {
        let text = self.text(span);
        let line = text.lines().next().unwrap_or("").trim();
        if line.chars().count() > 40 || line.len() < text.trim().len() {
            let short: String = line.chars().take(37).collect();
            format!("{short}...")
        } else {
            line.to_string()
        }
    }

    fn error(&mut self, code: Code, span: Span, message: String) -> Option<&mut Diagnostic> {
        let before = self.sink.len();
        self.sink.push(Diagnostic::error(code, span, message));
        (self.sink.len() > before)
            .then(|| self.sink.last_mut())
            .flatten()
    }

    /// Runs `f` without keeping what it reports or records: for reading a
    /// type out of code that is checked, and reported, somewhere else.
    fn quietly<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let (hovers, refs) = (self.hovers.len(), self.refs.len());
        let sink = std::mem::replace(&mut self.sink, Sink::new());
        let out = f(self);
        self.sink = sink;
        self.hovers.truncate(hovers);
        self.refs.truncate(refs);
        out
    }

    /// A type that came out of a generic, as the code here sees it: type
    /// parameters that nothing bound, and that are not this code's own,
    /// are unknown.
    fn settle(&self, ty: &Ty) -> Ty {
        ty.settle(&self.generics)
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn local(&self, name: &str) -> Option<&Local> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    /// Scope depth (0-based) and declaration span of a block-local `domain`, if any.
    pub(crate) fn domain_depth(&self, name: &str) -> Option<(usize, Span)> {
        self.scopes.iter().enumerate().rev().find_map(|(depth, scope)| {
            scope
                .get(name)
                .filter(|loc| loc.keyword == "domain")
                .map(|loc| (depth, loc.span))
        })
    }

    /// Scope depth where `name` was declared, or `0` if top-level/global.
    pub(crate) fn var_depth(&self, name: &str) -> usize {
        self.scopes
            .iter()
            .enumerate()
            .rev()
            .find_map(|(depth, scope)| scope.contains_key(name).then_some(depth))
            .unwrap_or(0)
    }

    /// Reports an error if `ty` mentions a fresh domain declared deeper than
    /// `max_depth` (and not listed in `allowed`).
    pub(crate) fn check_domain_escape(
        &mut self,
        ty: &Ty,
        max_depth: usize,
        at: Span,
        how: &str,
        allowed: &[String],
        in_fn_return: bool,
    ) {
        for d in ty.mentioned_domains() {
            if allowed.iter().any(|a| a == &d) {
                continue;
            }
            if let Some((decl_depth, decl_span)) = self.domain_depth(&d) {
                if decl_depth > max_depth {
                    let msg = format!(
                        "a value of fresh domain `{d}` cannot escape its declaring block ({how})"
                    );
                    if let Some(diag) = self.error(codes::DOMAIN_ESCAPES, at, msg) {
                        diag.labels.push(cpl_diag::Label {
                            span: decl_span,
                            message: format!("domain `{d}` is declared here"),
                        });
                        diag.help = Some(if in_fn_return {
                            format!(
                                "return it existentially as `-> (domain {d}, ...)` or declare `domain {d}` in an outer scope"
                            )
                        } else {
                            format!(
                                "declare `domain {d}` in an outer scope if its values need to live past this block"
                            )
                        });
                    }
                }
            }
        }
    }

    /// Checks both `ty` and any local variables captured by a lambda expression
    /// against `max_depth`.
    pub(crate) fn check_expr_domain_escape(
        &mut self,
        expr: &cpl_syntax::ast::Expr,
        ty: &Ty,
        max_depth: usize,
        how: &str,
        allowed: &[String],
        in_fn_return: bool,
    ) {
        self.check_domain_escape(ty, max_depth, expr.span, how, allowed, in_fn_return);
        if let cpl_syntax::ast::ExprKind::Lambda { body, .. } = &expr.kind {
            let mut names = Vec::new();
            collect_expr_names(body, &mut names);
            for (name, span) in names {
                if let Some(local) = self.local(&name) {
                    let local_ty = local.ty.clone();
                    self.check_domain_escape(
                        &local_ty,
                        max_depth,
                        span,
                        &format!("captured in a closure {how}"),
                        allowed,
                        in_fn_return,
                    );
                }
            }
        }
    }

    /// Checks whether a mutating store call (`outer.push(x)`, `outer.insert(k, v)`,
    /// `outer.set(...)`, `outer.add(x)`) stores a value mentioning a fresh domain
    /// declared inside a deeper block than the receiver's root variable.
    pub(crate) fn check_store_domain_escape(
        &mut self,
        callee: &cpl_syntax::ast::Expr,
        args: &[cpl_syntax::ast::Arg],
    ) {
        let cpl_syntax::ast::ExprKind::Field { base, name } = &callee.kind else {
            return;
        };
        if !matches!(name.text.as_str(), "push" | "insert" | "set" | "add") {
            return;
        }
        let Some((root, _)) = Self::root_name(base) else {
            return;
        };
        let target_depth = self.var_depth(root);
        let how = format!("stored in `{root}` from an outer scope");
        for arg in args {
            let arg_ty = self.quietly(|c| c.infer(&arg.value));
            self.check_expr_domain_escape(&arg.value, &arg_ty, target_depth, &how, &[], false);
        }
    }

    fn declare(&mut self, name: &str, span: Span, ty: Ty, mutable: bool, keyword: &'static str) {
        if name.is_empty() || name == "_" {
            return;
        }
        self.hovers.push(HoverEntry {
            span,
            name: name.to_string(),
            ty: ty.clone(),
            keyword,
            decl: Some(span),
            owner: None,
        });
        self.scopes.last_mut().unwrap().insert(
            name.to_string(),
            Local {
                ty,
                mutable,
                span,
                keyword,
            },
        );
    }

    /// The type of a name used as a value.
    /// The type of a name as a value. One that a test has shown to hold a
    /// value is the value it holds (see [`cpl_syntax::narrow`]).
    fn name_ty(&mut self, name: &str, span: Span) -> Ty {
        let hover = self.hovers.len();
        let written = std::mem::take(&mut self.as_written);
        match self.name_ty_as_written(name, span) {
            Ty::Optional(inner) if !written && self.present.iter().any(|n| n == name) => {
                if let Some(entry) = self.hovers.get_mut(hover) {
                    entry.ty = (*inner).clone();
                }
                *inner
            }
            ty => ty,
        }
    }

    /// The type of `e`, read as an optional where it is a name: see
    /// [`narrow::asks_optional`].
    fn infer_optional(&mut self, e: &cpl_syntax::ast::Expr) -> Ty {
        self.as_written = narrow::asks_optional(e);
        let ty = self.infer(e);
        self.as_written = false;
        ty
    }

    /// Whether calling `callee` may assign `name`, which ends what a test
    /// showed about it.
    fn call_writes(&mut self, callee: &str, name: &str) -> bool {
        self.mutators.writes(callee, name)
            || self.env.effects(callee).writes.iter().any(|n| n == name)
    }

    fn name_ty_as_written(&mut self, name: &str, span: Span) -> Ty {
        if let Some(local) = self.local(name) {
            let (ty, keyword, decl) = (local.ty.clone(), local.keyword, local.span);
            // A local type alias used as a value names the domain or the
            // constructor: `Len.first`.
            let ty = match ty {
                Ty::Pos(domain) if keyword == "type" => Ty::Domain(domain),
                Ty::Named { name, .. } if keyword == "type" => Ty::TypeName(name),
                ty => ty,
            };
            self.hovers.push(HoverEntry {
                span,
                name: name.to_string(),
                ty: ty.clone(),
                keyword,
                decl: Some(decl),
                owner: None,
            });
            return ty;
        }
        if name == "Self" {
            if let Some(self_ty) = self.self_ty.clone() {
                return match self_ty {
                    Ty::Named { name, .. } => Ty::TypeName(name),
                    Ty::Pos(domain) => Ty::Domain(domain),
                    ty => ty,
                };
            }
        }
        // A type parameter: `T.zero`, `V.K`, or the `P` of `Mod<P>` as a number.
        if self.generics.iter().any(|g| g == name) {
            return Ty::Param(name.to_string());
        }
        if let Some(def) = self.env.global(name) {
            let ty = match &*def {
                // `Sum` without `<Int>`: nothing says what `T` is.
                Def::Value { ty, generics, .. } if !generics.is_empty() => self.settle(ty),
                def => def.value_ty(name),
            };
            self.refs.push(GlobalRef {
                span,
                name: name.to_string(),
                top_level: self.fns.is_empty(),
            });
            self.hovers.push(HoverEntry {
                span,
                name: name.to_string(),
                ty: ty.clone(),
                keyword: "global",
                decl: None,
                owner: None,
            });
            return ty;
        }
        if let Some(ty) = prelude::value(name) {
            return ty;
        }
        if self.env.has_imports() || (self.in_fn > 0 && is_implicit_generic(name)) {
            return Ty::Unknown;
        }
        self.unresolved(codes::UNRESOLVED_NAME, name, span, true);
        Ty::Error
    }

    /// Reports a name that is not in scope, suggesting the closest one.
    fn unresolved(&mut self, code: Code, name: &str, span: Span, value: bool) {
        if name.is_empty() {
            return;
        }
        let mut candidates: Vec<String> = self.env.names();
        candidates.extend(prelude::VALUE_NAMES.iter().map(|n| n.to_string()));
        if value {
            for scope in &self.scopes {
                candidates.extend(scope.keys().cloned());
            }
        }
        let suggestion = closest(name, candidates.iter().map(String::as_str));
        // Neither here nor in `stdlib/`: say so, and offer the closest
        // library name, since a program reaching for a helper that does not
        // exist yet usually means one that does under another name.
        // No stdlib export is one character long, and contest code is full
        // of one-letter locals, so "not in the stdlib" there is only noise.
        let unknown = suggestion.is_none()
            && name.chars().nth(1).is_some()
            && prelude::std_module(name).is_none()
            && cpl_syntax::removed_keyword(name).is_none();
        let library = if unknown {
            closest(name, prelude::STD_NAMES.iter().map(|(n, _)| *n))
        } else {
            None
        };
        let message = match (value, unknown) {
            (true, false) => format!("no `{name}` in scope"),
            (false, false) => format!("no type or domain named `{name}`"),
            (true, true) => format!("no `{name}` in scope, and not in the stdlib (yet)"),
            (false, true) => {
                format!("no type or domain named `{name}`, and not in the stdlib (yet)")
            }
        };
        if let Some(diag) = self.error(code, span, message) {
            match suggestion {
                _ if cpl_syntax::removed_keyword(name).is_some() => {
                    let fix = cpl_syntax::removed_keyword(name).unwrap_or_default();
                    diag.help = Some(format!("`{name}` was removed in CP Lang 0.2.2: {fix}"));
                }
                _ if prelude::std_module(name).is_some() => {
                    let module = prelude::std_module(name).unwrap_or_default();
                    let import = format!("use std.{module}.{{{name}}}");
                    diag.help = Some(format!(
                        "`{name}` is in the standard library: add `{import}`"
                    ));
                    diag.fixes.push(cpl_diag::Fix {
                        title: format!("add `{import}`"),
                        edits: vec![(Span::point(0), format!("{import}\n"))],
                    });
                }
                Some(s) => {
                    diag.help = Some(format!("did you mean `{s}`?"));
                    diag.fixes.push(cpl_diag::Fix {
                        title: format!("change to `{s}`"),
                        edits: vec![(span, s)],
                    });
                }
                None if library.is_some() => {
                    let s = library.unwrap_or_default();
                    let module = prelude::std_module(&s).unwrap_or_default();
                    let import = format!("use std.{module}.{{{s}}}");
                    diag.help = Some(format!("did you mean `{s}`, from `std.{module}`?"));
                    diag.fixes.push(cpl_diag::Fix {
                        title: format!("change to `{s}` and add `{import}`"),
                        edits: vec![(Span::point(0), format!("{import}\n")), (span, s)],
                    });
                }
                None if value => {
                    diag.help = Some(format!("declare it first: `let {name} = ...`"));
                }
                None => {}
            }
        }
    }

    /// The variable an assignment or `mut` argument ultimately changes.
    fn root_name(expr: &cpl_syntax::ast::Expr) -> Option<(&str, Span)> {
        use cpl_syntax::ast::ExprKind as E;
        match &expr.kind {
            E::Name(name) => Some((name, expr.span)),
            E::SelfValue => Some(("self", expr.span)),
            E::Field { base, .. } | E::Index { base, .. } => Self::root_name(base),
            E::Paren(inner) | E::Unwrap(inner) => Self::root_name(inner),
            _ => None,
        }
    }

    /// Checks that `expr` can be changed. `action` finishes the sentence
    /// "`x` is a `let`, so it can't be ...".
    fn require_mutable(&mut self, expr: &cpl_syntax::ast::Expr, action: &str) {
        self.writable(expr);
        let Some((name, span)) = Self::root_name(expr) else {
            return;
        };
        let (mutable, keyword, decl) = match self.local(name) {
            Some(local) => (local.mutable, local.keyword, Some(local.span)),
            None => match self.env.global(name).as_deref() {
                Some(Def::Value {
                    mutable, keyword, ..
                }) => (*mutable, *keyword, None),
                _ => return,
            },
        };
        if mutable {
            return;
        }
        let (what, help) = match keyword {
            "param" => (
                "a parameter without `mut`".to_string(),
                format!("declare it `mut {name}` to change it"),
            ),
            "for" => (
                "a loop variable".to_string(),
                "copy it into a `var` to change it".to_string(),
            ),
            "self" => (
                "`self` without `mut`".to_string(),
                "declare the method with `mut self` to change it".to_string(),
            ),
            other => (
                format!("a `{other}`"),
                format!("declare it with `var {name}` to change it"),
            ),
        };
        let subject = if name == "self" {
            "`self`".to_string()
        } else {
            format!("`{name}`")
        };
        let message = if keyword == "self" {
            format!("this method takes {what}, so `self` can't be {action}")
        } else {
            format!("{subject} is {what}, so it can't be {action}")
        };
        if let Some(diag) = self.error(codes::ASSIGN_TO_LET, span, message) {
            diag.help = Some(help);
            if let Some(decl) = decl {
                diag.labels.push(cpl_diag::Label {
                    span: decl,
                    message: "declared here".into(),
                });
            }
        }
    }
}

/// `fn peel(s: [D]Bool)`: a single capital letter is a type parameter
/// without being declared (`plan/17-syntax.md`).
pub(crate) fn is_implicit_generic(name: &str) -> bool {
    name.len() == 1 && name.as_bytes()[0].is_ascii_uppercase()
}

/// The candidate closest to `name`, if it is close enough to be a typo.
pub(crate) fn closest<'c>(name: &str, candidates: impl Iterator<Item = &'c str>) -> Option<String> {
    let limit = match name.chars().count() {
        0 | 1 => return None,
        2..=4 => 1,
        _ => 2,
    };
    candidates
        .filter(|c| *c != name)
        .map(|c| (edit_distance(name, c), c))
        .filter(|(d, _)| *d <= limit)
        .min_by_key(|(d, c)| (*d, c.to_string()))
        .map(|(_, c)| c.to_string())
}

/// Edits to turn `a` into `b`, counting a swap of two neighbours as one:
/// `raods` is one edit from `roads`.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut rows = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in rows.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in rows[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let substitute = rows[i - 1][j - 1] + usize::from(a[i - 1] != b[j - 1]);
            let mut best = substitute.min(rows[i - 1][j] + 1).min(rows[i][j - 1] + 1);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(rows[i - 2][j - 2] + 1);
            }
            rows[i][j] = best;
        }
    }
    rows[a.len()][b.len()]
}

fn collect_expr_names(expr: &cpl_syntax::ast::Expr, out: &mut Vec<(String, Span)>) {
    use cpl_syntax::visit::{walk_expr, Visitor};
    struct NameCollector<'a>(&'a mut Vec<(String, Span)>);
    impl<'a> Visitor for NameCollector<'a> {
        fn expr(&mut self, expr: &cpl_syntax::ast::Expr) {
            if let cpl_syntax::ast::ExprKind::Name(name) = &expr.kind {
                self.0.push((name.clone(), expr.span));
            }
        }
    }
    walk_expr(expr, &mut NameCollector(out));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggestions_are_close_names_only() {
        let names = ["total", "roads", "dist", "n"];
        assert_eq!(closest("totl", names.iter().copied()), Some("total".into()));
        assert_eq!(
            closest("raods", names.iter().copied()),
            Some("roads".into())
        );
        assert_eq!(closest("answer", names.iter().copied()), None);
        assert_eq!(closest("m", names.iter().copied()), None);
        assert_eq!(closest("dits", names.iter().copied()), Some("dist".into()));
        assert_eq!(edit_distance("kitten", "sitting"), 3);
    }
}
