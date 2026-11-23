//! A bare variant that two enums declare reads as the expected enum's.
//! (R02.7.1)

use super::*;

impl<'a> Lowerer<'a> {
    /// `name` as a variant of the enum `hint` wants, or of its sum domain.
    pub(in crate::lower) fn expected_variant(
        &self,
        name: &str,
        hint: Option<&Ty>,
    ) -> Option<(EnumId, usize)> {
        let id = match hint? {
            Ty::Enum(id) => *id,
            Ty::Pos(d) => self.sum_enums.get(d)?.enum_id,
            _ => return None,
        };
        let tag = self.enums[id]
            .variants
            .iter()
            .position(|(v, _)| v == name)?;
        Some((id, tag))
    }
}
