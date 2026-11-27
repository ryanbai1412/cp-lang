//! The primitive operations `@intrinsic("id")` names (spec R12.14): each one's id and its
//! stable number. The compiler matches [`Prim`] values, never the names libraries give them;
//! what each computes and the C++ it emits live with the engines that read this.

/// One primitive of the table. Its index in [`Prim::ALL`] is its stable number.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Prim {
    Popcount,
    Abs,
    Sqrt,
    Floor,
    Ceil,
    Round,
    Trunc,
    Ctz,
    Clz,
    BitWidth,
    BitCeil,
    Acos,
    Asin,
    Atan2,
    Cos,
    Sin,
    Tan,
    Hypot,
    Exp,
    Ln,
}

impl Prim {
    /// The table: each primitive, at its stable number.
    pub const ALL: [Prim; 20] = [
        Prim::Popcount,
        Prim::Abs,
        Prim::Sqrt,
        Prim::Floor,
        Prim::Ceil,
        Prim::Round,
        Prim::Trunc,
        Prim::Ctz,
        Prim::Clz,
        Prim::BitWidth,
        Prim::BitCeil,
        Prim::Acos,
        Prim::Asin,
        Prim::Atan2,
        Prim::Cos,
        Prim::Sin,
        Prim::Tan,
        Prim::Hypot,
        Prim::Exp,
        Prim::Ln,
    ];

    /// Each primitive's name, as `@intrinsic("...")` writes it.
    pub const IDS: [&'static str; 20] = [
        "popcount",
        "abs",
        "sqrt",
        "floor",
        "ceil",
        "round",
        "trunc",
        "ctz",
        "clz",
        "bit_width",
        "bit_ceil",
        "acos",
        "asin",
        "atan2",
        "cos",
        "sin",
        "tan",
        "hypot",
        "exp",
        "ln",
    ];

    /// The primitive `@intrinsic(id)` names.
    pub fn from_id(id: &str) -> Option<Prim> {
        let at = Prim::IDS.iter().position(|i| *i == id)?;
        Prim::ALL.get(at).copied()
    }

    /// The name `@intrinsic("...")` writes.
    pub fn id(self) -> &'static str {
        Prim::IDS[self as usize]
    }
}
