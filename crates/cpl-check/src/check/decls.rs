//! Rules a struct declaration must meet on its own, before any use.

use super::*;
use crate::ty::Bindings;
use cpl_syntax::ast;

impl Checker<'_> {
    /// R02.6.6 getters and R03.1.3 field bounds, plus the header's bounds.
    pub(super) fn struct_decl_rules(&mut self, s: &ast::StructDecl) {
        self.bounds_exist(&s.bounds);
        for field in &s.fields {
            self.field_bound_static(field);
        }
        let held = match self.global(&s.name.text).as_deref() {
            Some(Def::Struct { fields, .. }) => fields.iter().map(|p| p.ty.clone()).collect(),
            _ => Vec::new(),
        };
        for (field, ty) in s.fields.iter().zip(&held) {
            self.holds_itself(&s.name.text, &field.name.text, field.ty.span, ty);
        }
        for m in &s.methods {
            if let ast::StmtKind::Fn(f) = &m.kind {
                if m.decorators.iter().any(|d| d.name.text == "get") {
                    self.getter_defaults(f);
                }
            }
        }
    }

    /// The enum's own rules: `|` between variants, and no variant holding the enum.
    pub(super) fn enum_decl_rules(&mut self, e: &ast::EnumDecl) {
        self.no_comma_variants(&e.variants);
        let held = match self.global(&e.name.text).as_deref() {
            Some(Def::Enum { variants, .. }) => variants.clone(),
            _ => Vec::new(),
        };
        for (variant, (_, tys)) in e.variants.iter().zip(&held) {
            for ((name, ast_ty), ty) in variant.fields.iter().zip(tys) {
                let label = name.as_ref().map_or(&variant.name.text, |n| &n.text);
                self.holds_itself(&e.name.text, label, ast_ty.span, ty);
            }
        }
    }

    /// R03.2.9: a value stores its fields inline, so `l: ?N` inside `N` would
    /// make `N` endless. A `List<N>` or an `Id<N>` handle holds it by reference.
    fn holds_itself(&mut self, owner: &str, field: &str, span: Span, ty: &Ty) {
        if !self.held_by_value(owner, ty, &mut Vec::new()) {
            return;
        }
        let message = format!(
            "`{field}: {}` stores `{owner}` inside `{owner}` itself, so its size would never end",
            self.quote(span)
        );
        if let Some(diag) = self.error(codes::RECURSIVE_FIELD, span, message) {
            diag.help = Some(format!(
                "keep the nodes in a `var nodes = PArena<{owner}>()` from `std.persistent` and link them by handle, `?Id<{owner}>`, or hold them in a `List<{owner}>` (R03.2.9)"
            ));
        }
    }

    /// Whether `ty` stores a `target` inline: directly, in an optional, tuple,
    /// record or array, or in a struct or enum that does.
    fn held_by_value(&mut self, target: &str, ty: &Ty, seen: &mut Vec<String>) -> bool {
        match ty {
            Ty::Optional(t) | Ty::Array { elem: t, .. } => self.held_by_value(target, t, seen),
            Ty::Tuple(ts) => ts.iter().any(|t| self.held_by_value(target, t, seen)),
            Ty::Record(fs) => fs.iter().any(|(_, t)| self.held_by_value(target, t, seen)),
            Ty::Named { name, args } => {
                if name == target {
                    return true;
                }
                if seen.contains(name) {
                    return false;
                }
                seen.push(name.clone());
                let inner: Vec<Ty> = match self.global(name).as_deref() {
                    Some(Def::Struct {
                        generics, fields, ..
                    }) => {
                        let bind: Bindings =
                            generics.iter().cloned().zip(args.iter().cloned()).collect();
                        fields.iter().map(|p| p.ty.subst(&bind)).collect()
                    }
                    Some(Def::Enum { variants, .. }) => variants
                        .iter()
                        .flat_map(|(_, ts)| ts.iter().cloned())
                        .collect(),
                    _ => Vec::new(),
                };
                inner.iter().any(|t| self.held_by_value(target, t, seen))
            }
            _ => false,
        }
    }

    /// R03.1.3: layout never depends on input, so a field's range bounds
    /// name only constants, never a `let`, `var` or parameter.
    fn field_bound_static(&mut self, field: &ast::FieldDecl) {
        let Some(name) = self.input_bound(&field.ty) else {
            return;
        };
        let message = format!(
            "`{}: {}`: a field's bounds are fixed before input is read, but `{name}` is only known when the program runs",
            field.name.text,
            self.quote(field.ty.span)
        );
        if let Some(diag) = self.error(codes::FIELD_BOUND_FROM_INPUT, field.ty.span, message) {
            diag.help = Some(
                "bound it by a constant, or keep the value outside the struct (R03.1.3)".into(),
            );
        }
    }

    /// The first run-time value a range bound inside `ty` names.
    fn input_bound(&mut self, ty: &ast::Type) -> Option<String> {
        match &ty.kind {
            ast::TypeKind::Range { lo, hi, .. } => [lo, hi]
                .into_iter()
                .flatten()
                .flat_map(|e| super::lends::names_in(e))
                .find(|n| self.run_value(n)),
            ast::TypeKind::Tuple(ts) => ts.iter().find_map(|t| self.input_bound(t)),
            ast::TypeKind::Optional(t) => self.input_bound(t),
            _ => None,
        }
    }

    fn run_value(&mut self, name: &str) -> bool {
        if let Some(local) = self.local(name) {
            return matches!(local.keyword, "let" | "var" | "param");
        }
        matches!(
            self.global(name).as_deref(),
            Some(Def::Value {
                keyword: "let" | "var",
                ..
            })
        )
    }

    /// R02.6.6: a getter is read like a field, so every parameter but `self`
    /// has a default.
    fn getter_defaults(&mut self, f: &ast::FnDecl) {
        for p in f.params.iter().filter(|p| p.name.text != "self") {
            if p.default.is_some() {
                continue;
            }
            let message = format!(
                "`@get fn {}`: a getter is read like a field, so `{}` needs a default",
                f.name.text, p.name.text
            );
            if let Some(diag) = self.error(codes::GETTER_NEEDS_DEFAULTS, p.span, message) {
                diag.help = Some(format!(
                    "give `{}` a default, or drop `@get` and call it with `()` (R02.6.6)",
                    p.name.text
                ));
            }
        }
    }
}
