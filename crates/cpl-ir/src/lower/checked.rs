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
        if !self.checked_names_known(checked) {
            return None;
        }
        let written = written(checked, e.span)?;
        let diags = self.diags.len();
        let ty = self.resolve_ty(&written);
        self.diags.truncate(diags);
        (ty != Ty::Error).then_some(ty)
    }
}

impl Lowerer<'_> {
    /// False when writing `ty` out would not name what the checker meant:
    /// a param outside the instance's env, or a generic struct, whose
    /// written form would instantiate a new struct, not name this value's.
    fn checked_names_known(&self, ty: &Checked) -> bool {
        match ty {
            Checked::Param(n) => self.gen_env.contains_key(n.as_str()),
            Checked::Named { name, args } => {
                let generic = !args.is_empty()
                    && (self.view_struct(name).is_some()
                        || matches!(
                            self.mod_syms[self.cur_mod].get(name.as_str()),
                            Some(Sym::GenStruct(_))
                        ));
                !generic && args.iter().all(|t| self.checked_names_known(t))
            }
            Checked::Array { index, elem } => {
                index.iter().all(|t| self.checked_names_known(t)) && self.checked_names_known(elem)
            }
            Checked::Optional(t) => self.checked_names_known(t),
            Checked::Tuple(ts) => ts.iter().all(|t| self.checked_names_known(t)),
            Checked::Record(fs) => fs.iter().all(|(_, t)| self.checked_names_known(t)),
            Checked::Fn { params, ret } => {
                params.iter().all(|p| self.checked_names_known(&p.ty))
                    && self.checked_names_known(ret)
            }
            _ => true,
        }
    }

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
        let items = (inner.split(", "))
            .map(|n| ast::Expr {
                kind: ast::ExprKind::Name(n.to_string()),
                span: at,
            })
            .collect();
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
            let parts = ts.iter().map(|t| match t {
                Checked::Pos(n) if !n.contains(['(', ')']) => Some(n.as_str()),
                _ => None,
            });
            Some(format!(
                "({})",
                parts.collect::<Option<Vec<_>>>()?.join(", ")
            ))
        }
        _ => None,
    }
}

/// `ty` as the type a program would write for it, if it has a spelling.
fn written(ty: &Checked, at: Span) -> Option<ast::Type> {
    let name = |text: &str| ast::Name {
        text: text.to_string(),
        span: at,
    };
    let named = |text: &str, args| ast::TypeKind::Named {
        name: name(text),
        args,
    };
    let all = |tys: &[Checked]| {
        tys.iter()
            .map(|t| written(t, at))
            .collect::<Option<Vec<_>>>()
    };
    let kind = match ty {
        Checked::Int | Checked::Range { .. } => named("Int", Vec::new()),
        Checked::I128 => named("I128", Vec::new()),
        Checked::U64 => named("U64", Vec::new()),
        Checked::Float => named("F64", Vec::new()),
        Checked::Bool => named("Bool", Vec::new()),
        Checked::Char => named("Char", Vec::new()),
        Checked::Str => named("Str", Vec::new()),
        Checked::Pos(n) | Checked::Param(n) => named(n, Vec::new()),
        Checked::Named { name, args } => named(cpl_check::written_name(name), all(args)?),
        Checked::Array { index, elem } => ast::TypeKind::Array {
            index: all(index)?,
            elem: Box::new(written(elem, at)?),
        },
        Checked::Optional(t) => ast::TypeKind::Optional(Box::new(written(t, at)?)),
        Checked::Tuple(items) => ast::TypeKind::Tuple(all(items)?),
        Checked::Record(fields) => ast::TypeKind::Record(
            (fields.iter())
                .map(|(n, t)| Some((name(n), written(t, at)?)))
                .collect::<Option<_>>()?,
        ),
        _ => return None,
    };
    Some(ast::Type { kind, span: at })
}
