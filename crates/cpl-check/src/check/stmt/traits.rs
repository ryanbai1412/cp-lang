//! Methods, trait bounds and `impl` blocks: that a named trait exists and
//! an impl supplies what it asks for.

use super::*;

impl<'a> Checker<'a> {
    /// Checks the fns, laws and constants of a struct, enum, trait or impl
    /// body. The struct's fields can be named in them without `self.`.
    pub(crate) fn methods(&mut self, methods: &[Stmt], self_ty: &Ty) {
        let fields = match self_ty {
            Ty::Named { name, .. } => match self.global(name).as_deref() {
                Some(Def::Struct { fields, .. }) => fields.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        for method in methods {
            match &method.kind {
                StmtKind::Fn(f) => self.fn_decl(f, Some(self_ty), false, memo_of(method)),
                StmtKind::Law { name, .. } => {
                    self.field_scope(&fields, name.span);
                    self.stmt(method, false);
                    self.pop_scope();
                }
                // Owned domains and constants were declared on entry; this
                // checks their values.
                // A domain may size itself from the fields: `domain Idx = 0..src.len`.
                StmtKind::Domain { def, .. } => {
                    self.field_scope(&fields, def.span);
                    if !self.domain_sum(def) {
                        self.infer(def);
                    }
                    self.pop_scope();
                }
                StmtKind::Const {
                    init: Some(init), ..
                }
                | StmtKind::Let {
                    init: Some(init), ..
                } => {
                    self.infer(init);
                }
                _ => {
                    self.stmt(method, false);
                }
            }
        }
    }

    /// A scope naming the trait's own fns without `self`, so its laws and default
    /// bodies call them unqualified: `law assoc(..) = op(op(a, b), c) == ..` (R08.3.1).
    pub(crate) fn own_fns_scope(&mut self, trait_name: &str, body: &[Stmt]) {
        let sigs = match self.global(trait_name).as_deref() {
            Some(Def::Trait { methods, .. }) => methods.clone(),
            _ => Vec::new(),
        };
        self.push_scope();
        for stmt in body {
            let StmtKind::Fn(f) = &stmt.kind else {
                continue;
            };
            if f.params.first().is_some_and(|p| p.name.text == "self") {
                continue;
            }
            if let Some(m) = sigs.iter().find(|m| m.name == f.name.text) {
                self.declare(&f.name.text, f.name.span, m.ty.clone(), false, "fn");
            }
        }
    }

    /// Every trait a bound names must exist: `where T: Ord + Bounded`.
    pub(crate) fn bounds_exist(&mut self, bounds: &[ast::Bound]) {
        for bound in bounds {
            for trait_ty in &bound.traits {
                if matches!(trait_ty.kind, ast::TypeKind::Row(_)) {
                    continue;
                }
                self.trait_exists(trait_ty);
            }
        }
    }

    /// Reports a trait name that is not a trait. Returns the trait's
    /// methods when it is declared in cp source.
    pub(super) fn trait_exists(
        &mut self,
        trait_ty: &ast::Type,
    ) -> Option<Vec<crate::def::TraitMethod>> {
        let ast::TypeKind::Named { name, args } = &trait_ty.kind else {
            return None;
        };
        for arg in args {
            self.resolve_type(arg);
        }
        match self.global(&name.text).as_deref() {
            Some(Def::Trait { methods, .. }) => {
                self.refs.push(GlobalRef {
                    span: name.span,
                    name: name.text.clone(),
                    top_level: false,
                    in_lambda: false,
                });
                self.hovers.push(HoverEntry {
                    span: name.span,
                    name: name.text.clone(),
                    ty: Ty::TypeName(name.text.clone()),
                    keyword: "type",
                    decl: None,
                    owner: None,
                });
                return Some(methods.clone());
            }
            // A value such as the enum variant `Sign.Neg` doesn't hide the trait `Neg`.
            Some(Def::Value { .. } | Def::Fn { .. } | Def::Variant { .. }) | None
                if prelude::is_trait(&name.text) =>
            {
                self.hovers.push(HoverEntry {
                    span: name.span,
                    name: name.text.clone(),
                    ty: Ty::TypeName(name.text.clone()),
                    keyword: "trait",
                    decl: None,
                    owner: None,
                });
            }
            Some(Def::Value { .. } | Def::Fn { .. } | Def::Variant { .. }) | None
                if name.text.is_empty() => {}
            Some(_) => {
                let message = format!("`{}` is not a trait", name.text);
                self.error(codes::UNKNOWN_TYPE, name.span, message);
            }
            None if self.env.has_imports() => {}
            None => {
                let message = format!("no trait named `{}`", name.text);
                self.error(codes::UNKNOWN_TYPE, name.span, message);
            }
        }
        None
    }

    /// An impl of a trait written in cp must define every fn the trait
    /// leaves without a body, and no fn the trait does not have.
    pub(super) fn impl_matches_trait(
        &mut self,
        trait_ty: &ast::Type,
        target: &ast::Type,
        body: &[Stmt],
    ) {
        let Some(methods) = self.trait_exists(trait_ty) else {
            return;
        };
        let trait_text = self.quote(trait_ty.span);
        if let Some(Def::Trait {
            supers,
            alias: true,
            ..
        }) = type_head(trait_ty).and_then(|n| self.global(&n)).as_deref()
        {
            let impls = supers
                .iter()
                .map(|s| format!("`impl {s} for {}`", self.quote(target.span)))
                .collect::<Vec<_>>()
                .join(" and ");
            let message = format!(
                "`{trait_text}` is a trait alias: it names bounds and has no impl of its own"
            );
            if let Some(diag) = self.error(codes::UNSUPPORTED, trait_ty.span, message) {
                diag.help = Some(format!("write {impls} instead"));
            }
            return;
        }
        let defined: Vec<&ast::Name> = body
            .iter()
            .filter_map(|m| match &m.kind {
                StmtKind::Fn(f) => Some(&f.name),
                _ => None,
            })
            .collect();
        for name in &defined {
            if methods.iter().all(|m| m.name != name.text) {
                let known: Vec<String> = methods.iter().map(|m| m.name.clone()).collect();
                let suggestion = closest(&name.text, known.iter().map(String::as_str));
                let message = format!("`{trait_text}` has no fn `{}`", name.text);
                if let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) {
                    match suggestion {
                        Some(s) => diag.suggest(name.span, s),
                        None => {
                            diag.help =
                                Some(format!("its fns are {}", super::expr::list_names(&known)));
                        }
                    }
                }
            }
        }
        let own = match self.resolve_type(target) {
            Ty::Named { name, .. } => self.members_of(&name).map(|m| m.fields),
            _ => None,
        };
        let missing: Vec<String> = methods
            .iter()
            .filter(|m| m.required && defined.iter().all(|d| d.text != m.name))
            .filter(|m| own.iter().flatten().all(|f| f.name != m.name))
            .map(|m| m.name.clone())
            .collect();
        if !missing.is_empty() {
            let message = format!(
                "`impl {trait_text} for {}` is missing {}",
                self.quote(target.span),
                super::expr::list_names(&missing)
            );
            let span = trait_ty.span.to(target.span);
            if let Some(diag) = self.error(codes::WRONG_ARG_COUNT, span, message) {
                diag.help = Some(format!(
                    "`{trait_text}` declares {} without a body, so each impl writes {}",
                    if missing.len() == 1 { "it" } else { "them" },
                    if missing.len() == 1 { "it" } else { "them" },
                ));
            }
        }
        self.impl_chooses_assoc_types(trait_ty, target, body);
    }
}
