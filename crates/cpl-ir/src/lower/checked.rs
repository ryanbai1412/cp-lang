//! What the checker found about an expression, read where lowering would
//! otherwise lower it a second time to learn it (the `ir_reinfer` debt).

use super::*;
use cpl_check::Ty as Checked;

impl<'a> Lowerer<'a> {
    /// The type the checker gave `e`, resolved as the same type written out
    /// would be: generic params through the instance's env, domains and
    /// structs by name.
    pub(in crate::lower) fn checked_ty(&mut self, e: &ast::Expr) -> Option<Ty> {
        let facts = std::sync::Arc::clone(self.mods[self.cur_mod].facts());
        let written = written(facts.type_of(e.span.shift(self.offset))?, e.span)?;
        let diags = self.diags.len();
        let ty = self.resolve_ty(&written);
        self.diags.truncate(diags);
        (ty != Ty::Error).then_some(ty)
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
    let all = |tys: &[Checked]| tys.iter().map(|t| written(t, at)).collect::<Option<Vec<_>>>();
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
