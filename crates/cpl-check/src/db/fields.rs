//! Where a struct declares a field, for an error elsewhere that offers to
//! change the declaration: `var` on a fixed field (E0405).

use super::*;

pub(super) const DEP_FIELD: &str = "\0field:";

impl FileCtx<'_> {
    /// Where `ty` declares `field` on a line of its own, relative to the
    /// item being checked: before its start when the struct comes first.
    pub(super) fn field_decl_at(&mut self, ty: &str, field: &str) -> Option<Span> {
        let key = format!("{DEP_FIELD}{ty}");
        let fingerprint = self.field_fp(&key);
        self.record(&key, fingerprint);
        let (k, at) = self.field_name(ty, field)?;
        Some(at.shift(self.back_to(k)?.wrapping_neg()))
    }

    /// How far before the item being checked item `k` starts.
    fn back_to(&self, k: usize) -> Option<u32> {
        let current = *self.offsets.get(self.current?)?;
        Some(current.wrapping_sub(*self.offsets.get(k)?))
    }

    fn field_name(&self, ty: &str, field: &str) -> Option<(usize, Span)> {
        let k = *self.scope.names.get(ty)?;
        let item = &self.items[k];
        let StmtKind::Struct(s) = &item.parsed.stmts.first()?.kind else {
            return None;
        };
        let i = s.fields.iter().position(|f| f.name.text == field)?;
        let name = s.fields[i].name.span;
        let head = item.text.get(..name.start as usize)?;
        let line = head[head.rfind('\n').map_or(0, |i| i + 1)..].trim();
        // After a comma only when the field before has its own type (not `u, w: Int`).
        let after_typed =
            line.ends_with(',') && i > 0 && s.fields[i - 1].ty.span != s.fields[i].ty.span;
        let alone = line.is_empty() || line.ends_with(':') || after_typed;
        let rest = item.text.get(name.end as usize..)?.trim_start();
        (alone && rest.starts_with(':')).then_some((k, Span::point(name.start)))
    }

    /// Changes when the struct's item, or how far it is from the item
    /// being checked, does.
    pub(super) fn field_fp(&mut self, key: &str) -> u64 {
        let ty = &key[DEP_FIELD.len()..];
        let Some(&k) = self.scope.names.get(ty) else {
            return 0;
        };
        let back = u64::from(self.back_to(k).unwrap_or(0));
        self.items[k].hash ^ back.wrapping_mul(0x9e37_79b9_7f4a_7c15)
    }
}
