//! What the checker found about an expression, read where lowering would
//! otherwise lower it a second time to learn it (the `ir_reinfer` debt).

use super::*;
use cpl_check::Ty as Checked;

impl<'a> Lowerer<'a> {
    /// The type the checker gave `e`, resolved as the same type written out
    /// would be: generic params through the instance's env, domains and
    /// structs by name.
    pub(in crate::lower) fn checked_ty(&mut self, e: &ast::Expr) -> Option<Ty> {
        let at = e.span.shift(self.offset);
        let facts = self.checked_facts(at);
        let checked = facts.type_of(at)?;
        if let Some(name) = product_name(checked) {
            if let Some(d) = self.inline_product_named(&name, e.span) {
                return Some(Ty::Pos(d));
            }
        }
        let written = self.written(checked, e.span)?;
        let diags = self.diags.len();
        let ty = self.resolve_ty(&written);
        self.diags.truncate(diags);
        (ty != Ty::Error).then_some(ty)
    }

    /// Whether the checker resolved the call or getter `e` to a user `yields`
    /// fn: a call to one is typed `Gen`.
    pub(in crate::lower) fn checked_generator_call(&mut self, e: &ast::Expr) -> bool {
        let at = e.span.shift(self.offset);
        let facts = self.checked_facts(at);
        facts.callee(at).is_some_and(|c| !c.builtin)
            && matches!(facts.type_of(at), Some(Checked::Named { name, .. }) if name == "Gen")
    }
}

impl Lowerer<'_> {
    /// The checker's facts around `at` in the module being lowered: an
    /// imported body's from the caller's source, else the module's own.
    fn checked_facts(&mut self, at: Span) -> std::sync::Arc<cpl_check::Facts> {
        let m = self.cur_mod;
        if self.mod_paths.len() <= m {
            self.mod_paths.resize(self.mods.len().max(m + 1), None);
        }
        let mods = &self.mods;
        let path = *self.mod_paths[m].get_or_insert_with(|| {
            (mods.iter())
                .flat_map(|a| a.modules())
                .find(|i| std::ptr::eq(&*i.analysis, mods[m]))
                .map(|i| i.path.as_str())
        });
        (path.and_then(|p| self.facts.body_facts(p, at)))
            .unwrap_or_else(|| std::sync::Arc::clone(self.mods[m].facts()))
    }

    /// The inline product domain a checker's `(R, C)` names.
    fn inline_product_named(&mut self, name: &str, at: Span) -> Option<DomainId> {
        let inner = name.strip_prefix('(')?.strip_suffix(')')?;
        if inner.contains(['(', ')']) {
            return None;
        }
        let mut items = Vec::new();
        for n in inner.split(", ") {
            items.push(ast::Expr {
                kind: ast::ExprKind::Name(n.to_string()),
                span: at,
            });
        }
        let tuple = ast::Expr {
            kind: ast::ExprKind::Tuple(items),
            span: at,
        };
        self.intern_inline_product(&tuple, at)
    }
}

/// The product domain a checked position names: `(R, C)` itself, or a
/// tuple of positions, which is that product's position viewed by parts.
fn product_name(ty: &Checked) -> Option<String> {
    match ty {
        Checked::Pos(name) => Some(name.clone()),
        Checked::Tuple(ts) if ts.len() > 1 => {
            let mut name = String::from("(");
            for (i, t) in ts.iter().enumerate() {
                let Checked::Pos(n) = t else { return None };
                if n.contains(['(', ')']) {
                    return None;
                }
                if i > 0 {
                    name.push_str(", ");
                }
                name.push_str(n);
            }
            name.push(')');
            Some(name)
        }
        _ => None,
    }
}

impl Lowerer<'_> {
    /// `ty` as the type a program would write for it, if it has a spelling
    /// that names what the checker meant: not a param outside the instance's
    /// env, nor a generic struct, whose written form would make a new one.
    fn written(&self, ty: &Checked, at: Span) -> Option<ast::Type> {
        let name = |text: &str| ast::Name {
            text: text.to_string(),
            span: at,
        };
        let named = |text: &str, args| ast::TypeKind::Named {
            name: name(text),
            args,
        };
        let all = |tys: &[Checked]| {
            let mut out = Vec::with_capacity(tys.len());
            for t in tys {
                out.push(self.written(t, at)?);
            }
            Some(out)
        };
        let plain = match ty {
            Checked::Int | Checked::Range { .. } => Some("Int"),
            Checked::I128 => Some("I128"),
            Checked::U64 => Some("U64"),
            Checked::Float => Some("F64"),
            Checked::Bool => Some("Bool"),
            Checked::Char => Some("Char"),
            Checked::Str => Some("Str"),
            Checked::Param(n) if !self.gen_env.contains_key(n.as_str()) => return None,
            Checked::Pos(n) | Checked::Param(n) => Some(n.as_str()),
            _ => None,
        };
        let kind = match ty {
            _ if plain.is_some() => named(plain.unwrap_or_default(), Vec::new()),
            Checked::Named { name, args } => {
                let generic = !args.is_empty()
                    && (self.view_struct(name).is_some()
                        || matches!(
                            self.mod_syms[self.cur_mod].get(name.as_str()),
                            Some(Sym::GenStruct(_))
                        ));
                if generic {
                    return None;
                }
                named(cpl_check::written_name(name), all(args)?)
            }
            Checked::Array { index, elem } => ast::TypeKind::Array {
                index: all(index)?,
                elem: Box::new(self.written(elem, at)?),
            },
            Checked::Optional(t) => ast::TypeKind::Optional(Box::new(self.written(t, at)?)),
            Checked::Tuple(items) => ast::TypeKind::Tuple(all(items)?),
            Checked::Record(fields) => {
                let mut out = Vec::with_capacity(fields.len());
                for (n, t) in fields {
                    out.push((name(n), self.written(t, at)?));
                }
                ast::TypeKind::Record(out)
            }
            _ => return None,
        };
        Some(ast::Type { kind, span: at })
    }
}
