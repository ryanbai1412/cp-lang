//! `s.cut_after(i)`, `s.cut_before(i)`: the cut beside key `i` in walk order
//! (R07.4.14).

use super::*;

impl Checker<'_> {
    /// The type of `cut_after`/`cut_before` on a sequence with cuts between
    /// its keys, through any `.rev` or slice view.
    pub(super) fn cut_member(ty: &Ty, name: &str) -> Option<Ty> {
        if !matches!(name, "cut_after" | "cut_before") {
            return None;
        }
        let mut seq = ty;
        while let Some(source) = seq.view_source() {
            seq = source;
        }
        let key = match seq {
            Ty::Str => Ty::Int,
            Ty::Named { name, .. } if name == "List" => Ty::Int,
            Ty::Array { index, .. } if index.len() == 1 => match &index[0] {
                Ty::Pos(d) => Ty::Pos(d.clone()),
                _ => Ty::Int,
            },
            _ => return None,
        };
        let ret = match &key {
            Ty::Pos(d) => Ty::Pos(format!("{d}.cuts")),
            _ => Ty::Int,
        };
        Some(Ty::Fn {
            params: vec![crate::Param {
                name: "i".into(),
                ty: key,
                mutable: false,
                has_default: false,
                named_only: false,
                variadic: false,
                lazy: false,
            }],
            ret: Box::new(ret),
        })
    }
}
