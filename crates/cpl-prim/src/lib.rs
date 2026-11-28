//! The primitive operations `@intrinsic("id")` names (spec R12.14): each one's id and its
//! stable number. The compiler matches [`PrimOp`] values, never the names libraries give them;
//! what each computes and the C++ it emits live with the engines that read this.

mod core_ty;
pub use core_ty::Prim;

/// A scalar type a primitive takes or gives, as its table cells write it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Int,
    U64,
    I128,
    F64,
}
impl Kind {
    pub const ALL: [Kind; 4] = [Kind::Int, Kind::U64, Kind::I128, Kind::F64];
    /// How a cell of the table writes it.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Int => "Int",
            Kind::U64 => "U64",
            Kind::I128 => "I128",
            Kind::F64 => "F64",
        }
    }
    /// The kinds a cell of [`PrimOp::params`] or [`PrimOp::result`] allows, `|` between them.
    pub fn in_cell(cell: &str) -> impl Iterator<Item = Kind> + '_ {
        (Kind::ALL.into_iter()).filter(move |k| cell.split('|').any(|w| w == k.name()))
    }
}

/// One primitive of the table. Its index in [`PrimOp::ALL`] is its stable number.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum PrimOp {
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

impl PrimOp {
    /// The table: each primitive, at its stable number.
    pub const ALL: [PrimOp; 20] = [
        PrimOp::Popcount,
        PrimOp::Abs,
        PrimOp::Sqrt,
        PrimOp::Floor,
        PrimOp::Ceil,
        PrimOp::Round,
        PrimOp::Trunc,
        PrimOp::Ctz,
        PrimOp::Clz,
        PrimOp::BitWidth,
        PrimOp::BitCeil,
        PrimOp::Acos,
        PrimOp::Asin,
        PrimOp::Atan2,
        PrimOp::Cos,
        PrimOp::Sin,
        PrimOp::Tan,
        PrimOp::Hypot,
        PrimOp::Exp,
        PrimOp::Ln,
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

    /// The parameter types, `|` between the types one accepts.
    pub fn params(self) -> &'static [&'static str] {
        const REAL: &[&str] = &["Int|F64"];
        const REALS: &[&str] = &["Int|F64", "Int|F64"];
        const PARAMS: [&[&str]; 20] = [
            &["Int|U64"],
            &["Int|I128|F64"],
            &["Int|F64"],
            &["F64"],
            &["F64"],
            &["F64"],
            &["F64"],
            &["Int|U64"],
            &["Int|U64"],
            &["Int|U64"],
            &["Int"],
            REAL,
            REAL,
            REALS,
            REAL,
            REAL,
            REAL,
            REALS,
            REAL,
            REAL,
        ];
        PARAMS[self as usize]
    }

    /// The result type, `|` between the types it may be.
    pub fn result(self) -> &'static str {
        const RESULTS: [&str; 20] = [
            "Int",
            "Int|I128|F64",
            "F64",
            "Int",
            "Int",
            "Int",
            "Int",
            "Int",
            "Int",
            "Int",
            "Int",
            "F64",
            "F64",
            "F64",
            "F64",
            "F64",
            "F64",
            "F64",
            "F64",
            "F64",
        ];
        RESULTS[self as usize]
    }

    /// The primitive `@intrinsic(id)` names.
    pub fn from_id(id: &str) -> Option<PrimOp> {
        let at = PrimOp::IDS.iter().position(|i| *i == id)?;
        PrimOp::ALL.get(at).copied()
    }

    /// The name `@intrinsic("...")` writes.
    pub const fn id(self) -> &'static str {
        PrimOp::IDS[self as usize]
    }
}
