//! What a method is declared to return: its std signature, or the type text
//! of its built-in catalog row, read as a type.

use super::*;

impl Checker<'_> {
    /// The type `base.name(...)` is declared to return on `base_ty`: the
    /// signature of a std method, or a built-in's catalog row
    /// (`Set.range` is `SortedRange<T>`).
    pub(super) fn declared_ret(
        &mut self,
        base: &Expr,
        base_ty: &Ty,
        name: &str,
        span: Span,
    ) -> Option<Ty> {
        if let Ty::Named { name: owner, args } = base_ty {
            if let Some(members) = self.members_of(owner) {
                let bindings = members.bindings(args);
                if let Some((_, Ty::Fn { ret, .. })) =
                    members.members.iter().find(|(m, _)| m == name)
                {
                    return Some(ret.subst(&bindings));
                }
            }
        }
        let (member, elem) = self.catalog_row(base, name)?;
        let key_value = match base_ty {
            Ty::Named { name, args } if matches!(name.as_str(), "Map" | "HashMap") => args.clone(),
            _ => Vec::new(),
        };
        // The row's text is no use in the source: hover and references at
        // `span` stay the call's.
        let (hovers, refs) = (self.hovers.len(), self.refs.len());
        let ret = self.row_ty(member.ret, &elem, &key_value, span);
        self.hovers.truncate(hovers);
        self.refs.truncate(refs);
        ret
    }

    /// A catalog row's type `text` (`?T`, `(K, V)`, `SortedRange<(K, V)>`),
    /// a named head resolved as any written type is.
    fn row_ty(&mut self, text: &str, elem: &Option<Ty>, kv: &[Ty], span: Span) -> Option<Ty> {
        let text = text.trim();
        if let Some(inner) = text.strip_prefix('?') {
            return Some(Ty::Optional(Box::new(self.row_ty(inner, elem, kv, span)?)));
        }
        let items = |c: &mut Self, inner: &str| {
            split_top(inner)
                .into_iter()
                .map(|t| c.row_ty(t, elem, kv, span))
                .collect::<Option<Vec<_>>>()
        };
        if let Some(inner) = text.strip_prefix('(').and_then(|t| t.strip_suffix(')')) {
            return Some(Ty::Tuple(items(self, inner)?));
        }
        if let Some((head, inner)) = text
            .split_once('<')
            .and_then(|(h, rest)| Some((h, rest.strip_suffix('>')?)))
        {
            let args = items(self, inner)?;
            return Some(self.named_type(head, span, args));
        }
        catalog_ty(text, elem, kv)
    }
}
