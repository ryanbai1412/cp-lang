//! Typed register bytecode VM for plain (untraced, unprofiled) runs.
//!
//! `cpl-ir` already resolves every local to a `Slot` and gives every slot,
//! global, and expression a static [`Ty`]. The VM exploits that static typing:
//! - Scalar values (`Int`, `Pos`, `Bound`, `Bool`, `Char`, and 1-field `Int`
//!   structs like `Mod<P>`) live unboxed in a flat `Vec<i64>` register bank
//!   (`regs_i`) with zero `Value` tagging, cloning, or dropping.
//! - Optional scalars (`?Int`, `?Pos`, `?Bound`, `?Bool`, `?Char`, `?Mod<P>`)
//!   live unboxed as a `(val, is_some)` pair of `i64` registers, backed by
//!   `ArrayData::OptInt` for `@memo` and DP tables.
//! - Compound values (`Array`, `List`, `Str`, `Map`, `Set`, multi-field
//!   tuples/structs/enums, closures) live in `regs_v: Vec<Value>`.
//! - Small pure leaf functions (e.g. `Mod<P>` arithmetic) are inlined into the
//!   caller's bytecode stream at compile time.
//! - Hot contest loops use superinstructions (`ForInc`, `ForIncClosed`,
//!   `ForDec`, `LoadGArr1I`/`StoreGArr1I`, `LoadGArr2I`/`StoreGArr2I`,
//!   `LoadGArr2OptI`/`StoreGArr2OptI`, `PosNext`/`PosPrev`/`PosOffset`,
//!   `JmpLtI`/`JmpLeI`/`JmpEqI`/`JmpNeI`).
//! - Any IR construct not lowered natively executes via `FallbackStmt` /
//!   `FallbackExpr`, syncing live local slots with `Interp` so 100% of CP Lang
//!   features work identically.

use crate::value::{Array, ArrayData, CaptureVal, OptVal, SmallFieldKind, TupleVal, Value};
use crate::{describe, trap, Flow, Interp, Res, LENT};
use cpl_diag::Span;
use cpl_ir::{
    Arg, AssignOp, BinOp, Block, Builtin, Cast, CmpOp, Expr, ExprKind, FmtArg, FmtPiece, FnId, Iter, Local, LoopId,
    Mutation, Place, PlaceKind, Program, Slot, Stmt, Ty, UnOp,
};
use std::rc::Rc;

/// How an unboxed `i64` register maps back to a [`Value`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ScalarKind {
    Int,
    U64,
    Bool,
    Char,
    OneIntStruct,
}

impl ScalarKind {
    pub fn of_ty(ty: &Ty, program: &Program) -> Option<ScalarKind> {
        match ty {
            Ty::Int | Ty::Pos(_) | Ty::Bound(_) => Some(ScalarKind::Int),
            Ty::U64 => Some(ScalarKind::U64),
            Ty::Bool => Some(ScalarKind::Bool),
            Ty::Char => Some(ScalarKind::Char),
            Ty::Struct(id) => {
                let s = &program.structs[*id];
                if s.fields.len() == 1
                    && matches!(s.fields[0].1, Ty::Int | Ty::Pos(_) | Ty::Bound(_))
                {
                    Some(ScalarKind::OneIntStruct)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn of_opt_ty(ty: &Ty, program: &Program) -> Option<ScalarKind> {
        match ty {
            Ty::Opt(inner) => ScalarKind::of_ty(inner, program),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn box_val(self, raw: i64) -> Value {
        match self {
            ScalarKind::Int => Value::Int(raw),
            ScalarKind::U64 => Value::U64(raw as u64),
            ScalarKind::Bool => Value::Bool(raw != 0),
            ScalarKind::Char => Value::Char(raw as u8),
            ScalarKind::OneIntStruct => Value::Tuple(TupleVal::OneInt(raw)),
        }
    }

    #[inline(always)]
    pub fn box_opt(self, raw: i64, some: i64) -> Value {
        if some == 0 {
            Value::Opt(OptVal::None)
        } else if self == ScalarKind::Int {
            Value::Opt(OptVal::Int(raw))
        } else {
            Value::Opt(OptVal::some(self.box_val(raw)))
        }
    }

    #[inline(always)]
    pub fn unbox_val(val: &Value) -> i64 {
        match val {
            Value::Int(v) => *v,
            Value::U64(v) => *v as i64,
            Value::Bool(b) => i64::from(*b),
            Value::Char(c) => i64::from(*c),
            Value::Tuple(TupleVal::OneInt(v)) => *v,
            Value::Tuple(TupleVal::Boxed(fs)) if fs.len() == 1 => Self::unbox_val(&fs[0]),
            _ => 0,
        }
    }

    #[inline(always)]
    pub fn unbox_opt(val: &Value) -> (i64, i64) {
        match val {
            Value::Opt(OptVal::None) => (0, 0),
            Value::Opt(OptVal::Int(v)) => (*v, 1),
            Value::Opt(OptVal::Boxed(rc)) => (ScalarKind::unbox_val(rc), 1),
            _ => (0, 0),
        }
    }
}

fn small_tuple_shape(ty: &Ty, program: &Program) -> Option<(u8, [SmallFieldKind; 4])> {
    let field_tys: Vec<&Ty> = match ty {
        Ty::Tuple(fs) => fs.iter().collect(),
        Ty::Struct(id) => program.structs[*id].fields.iter().map(|(_, t)| t).collect(),
        _ => return None,
    };
    let len = field_tys.len();
    if !(2..=4).contains(&len) {
        return None;
    }
    let mut kinds = [SmallFieldKind::Int; 4];
    for (i, t) in field_tys.iter().enumerate() {
        let sk = ScalarKind::of_ty(t, program)?;
        kinds[i] = match sk {
            ScalarKind::Int => SmallFieldKind::Int,
            ScalarKind::OneIntStruct => SmallFieldKind::OneIntStruct,
            ScalarKind::Bool => SmallFieldKind::Bool,
            ScalarKind::Char => SmallFieldKind::Char,
            ScalarKind::U64 => return None,
        };
    }
    Some((len as u8, kinds))
}

/// Where a local `Slot` lives inside a VM frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SlotLoc {
    I(u16, ScalarKind),
    OptI { val: u16, some: u16, kind: ScalarKind },
    SmallTuple { base: u16, len: u8, kinds: [SmallFieldKind; 4] },
    V(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CallRet {
    Unit,
    I,
    OptI,
    V,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum PlaceRoot {
    LocalV(u16),
    Global(u16),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum PlaceStep {
    Field(u16),
    Index1(u16),
    Index2(u16, u16),
}

#[derive(Clone, Debug)]
pub(crate) struct PlacePath {
    pub root: PlaceRoot,
    pub steps: Vec<PlaceStep>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Op {
    // Constants & moves
    ConstImmI { dst: u16, val: i32 },
    ConstI { dst: u16, idx: u32 },
    MoveI { dst: u16, src: u16 },
    MoveV { dst: u16, src: u16 },
    MoveVConsume { dst: u16, src: u16 },
    BoxI { dst_v: u16, src_i: u16, kind: ScalarKind },
    UnboxI { dst_i: u16, src_v: u16 },
    BoxOptI { dst_v: u16, val: u16, some: u16, kind: ScalarKind },
    UnboxOptI { dst_val: u16, dst_some: u16, src_v: u16 },

    // Globals
    LoadGlobalI { dst: u16, gid: u16 },
    StoreGlobalI { gid: u16, src: u16, kind: ScalarKind },
    LoadGlobalOptI { dst_val: u16, dst_some: u16, gid: u16 },
    StoreGlobalOptI { gid: u16, val: u16, some: u16, kind: ScalarKind },
    LoadGlobalV { dst: u16, gid: u16 },
    StoreGlobalV { gid: u16, src: u16 },
    StoreGlobalVMove { gid: u16, src: u16 },

    // Checked i64 arithmetic
    AddI { dst: u16, a: u16, b: u16 },
    SubI { dst: u16, a: u16, b: u16 },
    MulI { dst: u16, a: u16, b: u16 },
    DivI { dst: u16, a: u16, b: u16 },
    RemI { dst: u16, a: u16, b: u16 },
    ModAddI { dst: u16, a: u16, b: u16, modulus: u32 },
    ModSubI { dst: u16, a: u16, b: u16, modulus: u32 },
    ModMulI { dst: u16, a: u16, b: u16, modulus: u32 },
    ModNegI { dst: u16, a: u16, modulus: u32 },
    AddImmI { dst: u16, a: u16, imm: i32 },
    NegI { dst: u16, a: u16 },
    BitAndI { dst: u16, a: u16, b: u16 },
    BitOrI { dst: u16, a: u16, b: u16 },
    BitXorI { dst: u16, a: u16, b: u16 },
    BitAndU64 { dst: u16, a: u16, b: u16 },
    BitOrU64 { dst: u16, a: u16, b: u16 },
    BitXorU64 { dst: u16, a: u16, b: u16 },
    ShlI { dst: u16, a: u16, b: u16 },
    ShrI { dst: u16, a: u16, b: u16 },
    NotBool { dst: u16, a: u16 },
    MinI { dst: u16, a: u16, b: u16 },
    MaxI { dst: u16, a: u16, b: u16 },
    AbsI { dst: u16, a: u16 },
    WrapAddI { dst: u16, a: u16, b: u16 },
    WrapSubI { dst: u16, a: u16, b: u16 },
    WrapMulI { dst: u16, a: u16, b: u16 },

    // Domain/Pos/Bound & Option superinstructions
    DomainLen { dst: u16, dom: u16 },
    DomainLo { dst: u16, dom: u16 },
    FromLabel { dst: u16, src: u16, dom: u16 },
    FromOrd { dst: u16, src: u16, dom: u16 },
    PosNext { dst_val: u16, dst_some: u16, src: u16, len: u16 },
    PosPrev { dst_val: u16, dst_some: u16, src: u16 },
    PosOffset { dst_val: u16, dst_some: u16, src: u16, delta: u16, dom: u16 },
    PosTryOrd { dst_val: u16, dst_some: u16, src: u16, dom: u16 },
    BoundStepL { dst: u16, ord: u16, k: u16, dom: u16 },
    BoundStepR { dst: u16, ord: u16, k: u16, dom: u16 },
    UnwrapOptI { dst: u16, val: u16, some: u16 },

    // Small unboxed scalar tuples (2..=4 fields)
    LoadGArr1SmallTuple { dst_base: u16, len: u8, gid: u16, r_i: u16 },
    LoadLArr1SmallTuple { dst_base: u16, len: u8, arr_v: u16, r_i: u16 },
    StoreGArr1SmallTuple { gid: u16, r_i: u16, src_base: u16, len: u8, kinds: [SmallFieldKind; 4] },
    StoreLArr1SmallTuple { arr_v: u16, r_i: u16, src_base: u16, len: u8, kinds: [SmallFieldKind; 4] },
    LoadGArr2SmallTuple { dst_base: u16, len: u8, gid: u16, r_i: u16, r_j: u16 },
    LoadLArr2SmallTuple { dst_base: u16, len: u8, arr_v: u16, r_i: u16, r_j: u16 },
    StoreGArr2SmallTuple { gid: u16, r_i: u16, r_j: u16, src_base: u16, len: u8, kinds: [SmallFieldKind; 4] },
    StoreLArr2SmallTuple { arr_v: u16, r_i: u16, r_j: u16, src_base: u16, len: u8, kinds: [SmallFieldKind; 4] },
    BoxSmallTuple { dst_v: u16, src_base: u16, len: u8, kinds: [SmallFieldKind; 4] },
    UnboxSmallTuple { dst_base: u16, len: u8, src_v: u16 },

    // Comparisons & branches
    CmpI { dst: u16, a: u16, b: u16, op: CmpOp },
    Jmp { target: u32 },
    /// Jumps when the last fallback statement broke out of or continued
    /// the loop `id`, or any loop when `innermost` and it named none.
    JmpEscape { brk: bool, id: LoopId, innermost: bool, target: u32 },
    JmpIfZero { reg: u16, target: u32 },
    JmpIfNonZero { reg: u16, target: u32 },
    JmpLtI { a: u16, b: u16, target: u32 },
    JmpLeI { a: u16, b: u16, target: u32 },
    JmpGtI { a: u16, b: u16, target: u32 },
    JmpGeI { a: u16, b: u16, target: u32 },
    JmpEqI { a: u16, b: u16, target: u32 },
    JmpNeI { a: u16, b: u16, target: u32 },

    // Counted loop superinstructions
    ForInc { r_i: u16, r_hi: u16, target: u32 },
    ForIncClosed { r_i: u16, r_hi: u16, target: u32 },
    ForDec { r_i: u16, r_lo: u16, target: u32 },

    // Unboxed array & list superinstructions (global and local)
    LoadGArr1I { dst: u16, gid: u16, r_i: u16 },
    StoreGArr1I { gid: u16, r_i: u16, r_val: u16 },
    LoadGArr2I { dst: u16, gid: u16, r_i: u16, r_j: u16 },
    StoreGArr2I { gid: u16, r_i: u16, r_j: u16, r_val: u16 },
    LoadLArr1I { dst: u16, arr_v: u16, r_i: u16 },
    StoreLArr1I { arr_v: u16, r_i: u16, r_val: u16 },
    LoadLArr2I { dst: u16, arr_v: u16, r_i: u16, r_j: u16 },
    StoreLArr2I { arr_v: u16, r_i: u16, r_j: u16, r_val: u16 },
    LoadGArr1OptI { dst_val: u16, dst_some: u16, gid: u16, r_i: u16 },
    StoreGArr1OptI { gid: u16, r_i: u16, r_val: u16, r_some: u16, kind: ScalarKind },
    LoadGArr2OptI { dst_val: u16, dst_some: u16, gid: u16, r_i: u16, r_j: u16 },
    StoreGArr2OptI { gid: u16, r_i: u16, r_j: u16, r_val: u16, r_some: u16, kind: ScalarKind },
    LoadLArr1OptI { dst_val: u16, dst_some: u16, arr_v: u16, r_i: u16 },
    StoreLArr1OptI { arr_v: u16, r_i: u16, r_val: u16, r_some: u16, kind: ScalarKind },
    LoadLArr2OptI { dst_val: u16, dst_some: u16, arr_v: u16, r_i: u16, r_j: u16 },
    StoreLArr2OptI { arr_v: u16, r_i: u16, r_j: u16, r_val: u16, r_some: u16, kind: ScalarKind },
    LoadGList1I { dst: u16, gid: u16, r_i: u16 },
    StoreGList1I { gid: u16, r_i: u16, r_val: u16, kind: ScalarKind },
    LoadLList1I { dst: u16, list_v: u16, r_i: u16 },
    StoreLList1I { list_v: u16, r_i: u16, r_val: u16, kind: ScalarKind },
    GArrLen { dst: u16, gid: u16 },
    LArrLen { dst: u16, arr_v: u16 },
    GListLen { dst: u16, gid: u16 },
    LListLen { dst: u16, list_v: u16 },

    LoadGArr1V { dst: u16, gid: u16, r_i: u16 },
    StoreGArr1V { gid: u16, r_i: u16, r_val: u16 },
    LoadLArr1V { dst: u16, arr_v: u16, r_i: u16 },
    StoreLArr1V { arr_v: u16, r_i: u16, r_val: u16 },
    FieldI { dst: u16, obj_v: u16, field: u16 },
    FieldOptI { dst_val: u16, dst_some: u16, obj_v: u16, field: u16 },
    FieldV { dst: u16, obj_v: u16, field: u16 },
    LoadFieldList1I { dst: u16, obj_v: u16, field: u16, r_i: u16 },
    FieldListLen { dst: u16, obj_v: u16, field: u16 },
    LoadFieldArr1I { dst: u16, obj_v: u16, field: u16, r_i: u16 },
    FieldArrLen { dst: u16, obj_v: u16, field: u16 },
    StrGetI { dst: u16, str_v: u16, r_i: u16 },
    GStrGetI { dst: u16, gid: u16, r_i: u16 },
    MakeTuple { dst: u16, base_v: u16, len: u8 },

    // Direct VM call (unboxed i64 args/return, or mixed I/V args & Unit/I/OptI/V return)
    CallVmI { fn_id: u16, arg_base: u16, argc: u8, dst: u16 },
    CallVmMix {
        fn_id: u16,
        arg_base_i: u16,
        arg_base_v: u16,
        dst_i: u16,
        dst_v: u16,
        dst_some: u16,
        ret_mode: CallRet,
    },
    CallClosure {
        callee_v: u16,
        arg_base_i: u16,
        arg_base_v: u16,
        dst_i: u16,
        dst_v: u16,
        dst_some: u16,
        ret_mode: CallRet,
    },
    RetI { src: u16 },
    RetV { src: u16 },
    RetUnit,

    // General V-channel optionals & sequence/range/bit/list ops
    IsSomeV { dst_i: u16, src_v: u16 },
    UnwrapV { dst_v: u16, src_v: u16 },
    UnwrapVI { dst_i: u16, src_v: u16 },
    WrapSomeV { dst_v: u16, src_v: u16 },
    ConstNoneV { dst_v: u16 },
    SeqLastV { dst_v: u16, seq_v: u16 },
    SeqLastOptI { dst_val: u16, dst_some: u16, seq_v: u16 },
    SeqFirstV { dst_v: u16, seq_v: u16 },
    SeqFirstOptI { dst_val: u16, dst_some: u16, seq_v: u16 },
    SeqLenI { dst: u16, seq_v: u16 },
    RangeLoI { dst: u16, src_v: u16 },
    RangeHiI { dst: u16, src_v: u16 },
    BitCeilI { dst: u16, src: u16 },
    PopcountI { dst: u16, src: u16 },
    CtzI { dst: u16, src: u16 },
    ClzI { dst: u16, src: u16 },
    MinMaxGArr1SmallTuple { gid: u16, r_i: u16, src_base: u16, len: u8, is_max: bool },
    MinMaxLArr1SmallTuple { arr_v: u16, r_i: u16, src_base: u16, len: u8, is_max: bool },
    ListPushGArr1I { gid: u16, r_i: u16, val_i: u16, kind: ScalarKind },
    ListPushGArr1V { gid: u16, r_i: u16, val_v: u16 },
    ListPushLArr1I { arr_v: u16, r_i: u16, val_i: u16, kind: ScalarKind },
    ListPushLArr1V { arr_v: u16, r_i: u16, val_v: u16 },
    ListPushLV { list_v: u16, val_v: u16 },
    ListPushGV { gid: u16, val_v: u16 },
    ListPushLI { list_v: u16, val_i: u16, kind: ScalarKind },
    ListPushGI { gid: u16, val_i: u16, kind: ScalarKind },
    ListPopLV { dst_v: u16, list_v: u16 },
    ListPopGV { dst_v: u16, gid: u16 },
    ListPopOptIL { dst_val: u16, dst_some: u16, list_v: u16 },
    ListPopOptIG { dst_val: u16, dst_some: u16, gid: u16 },
    ListPopDiscardL { list_v: u16 },
    ListPopDiscardG { gid: u16 },
    LoadGlobalVMove { dst: u16, gid: u16 },
    StoreLFieldI { obj_v: u16, field: u16, src_i: u16, kind: ScalarKind },
    StoreLFieldV { obj_v: u16, field: u16, src_v: u16 },
    StoreLFieldOptI { obj_v: u16, field: u16, val: u16, some: u16, kind: ScalarKind },
    StoreLFieldIdx1I { obj_v: u16, field: u16, r_i: u16, r_val: u16, kind: ScalarKind },
    StoreLFieldIdx1V { obj_v: u16, field: u16, r_i: u16, r_val: u16 },
    StoreLFieldIdx1OptI { obj_v: u16, field: u16, r_i: u16, val: u16, some: u16, kind: ScalarKind },
    ListPushFieldLI { obj_v: u16, field: u16, val_i: u16, kind: ScalarKind },
    ListPushFieldLV { obj_v: u16, field: u16, val_v: u16 },
    ListPopDiscardFieldL { obj_v: u16, field: u16 },
    StoreGList1V { gid: u16, r_i: u16, r_val: u16 },
    StoreLList1V { list_v: u16, r_i: u16, r_val: u16 },
    LoadGList1V { dst: u16, gid: u16, r_i: u16 },
    LoadLList1V { dst: u16, list_v: u16, r_i: u16 },
    LoadLList1OptI { dst_val: u16, dst_some: u16, list_v: u16, r_i: u16 },
    ListPopFieldOptIL { dst_val: u16, dst_some: u16, obj_v: u16, field: u16 },
    ListPopFieldLV { dst_v: u16, obj_v: u16, field: u16 },
    MakeRange { dst: u16, r_lo: u16, r_hi: u16 },
    CmpOptI { dst: u16, a_val: u16, a_some: u16, b_val: u16, b_some: u16, op: CmpOp },
    CmpV { dst: u16, a_v: u16, b_v: u16, op: CmpOp },
    MakeList { dst: u16, base_v: u16, len: u8 },
    ListFilled { dst: u16, r_n: u16, val_v: u16 },
    ToList { dst: u16, src_v: u16 },
    MakeFill { dst: u16, dims_base: u16, ndims: u8, fill_v: u16, fill_ty: u16 },
    ListSlice { dst: u16, src_v: u16, r_lo: u16, r_hi: u16 },
    StrSlice { dst: u16, src_v: u16, r_lo: u16, r_hi: u16 },
    TakeLArr1V { dst: u16, arr_v: u16, r_i: u16 },
    TakeGArr1V { dst: u16, gid: u16, r_i: u16 },
    TakeLList1V { dst: u16, list_v: u16, r_i: u16 },
    TakeGList1V { dst: u16, gid: u16, r_i: u16 },
    LoadLArr2V { dst: u16, arr_v: u16, r_i: u16, r_j: u16 },
    LoadGArr2V { dst: u16, gid: u16, r_i: u16, r_j: u16 },
    StoreLArr2V { arr_v: u16, r_i: u16, r_j: u16, r_val: u16 },
    StoreGArr2V { gid: u16, r_i: u16, r_j: u16, r_val: u16 },
    ConstF64 { dst: u16, idx: u16 },
    FloatBin { dst: u16, a: u16, b: u16, op: BinOp },
    FloatMinMax { dst: u16, a: u16, b: u16, is_max: bool },
    LoadGIdxFieldI { dst: u16, gid: u16, r_i: u16, field: u16 },
    StoreGIdxFieldI { gid: u16, r_i: u16, field: u16, r_val: u16, kind: ScalarKind },
    LoadLIdxFieldI { dst: u16, seq_v: u16, r_i: u16, field: u16 },
    StoreLIdxFieldI { seq_v: u16, r_i: u16, field: u16, r_val: u16, kind: ScalarKind },
    LStrLen { dst: u16, str_v: u16 },
    ClearV { reg: u16 },

    // Wrapping u64 arithmetic, casts & comparisons
    BinU64 { dst: u16, a: u16, b: u16, op: BinOp },
    NegU64 { dst: u16, a: u16 },
    BitNotI { dst: u16, a: u16 },
    CmpU64 { dst: u16, a: u16, b: u16, op: CmpOp },
    U64ToInt { dst: u16, src: u16 },

    // General nested PlacePath mutations & sort
    StorePlaceI { path: u16, r_val: u16, kind: ScalarKind },
    StorePlaceOptI { path: u16, r_val: u16, r_some: u16, kind: ScalarKind },
    StorePlaceV { path: u16, r_val: u16 },
    AssignPlaceI { path: u16, r_rhs: u16, op: AssignOp, kind: ScalarKind },
    ListPopDiscardPlace { path: u16 },
    ListPushPlaceI { path: u16, val_i: u16, kind: ScalarKind },
    ListPushPlaceV { path: u16, val_v: u16 },
    SortPlace { path: u16, desc_r: u16 },

    // Closure construction, Take mutation, and fed `yields` loops
    MakeClosure { dst: u16, spec: u16 },
    TakeLocalV { dst: u16, src: u16 },
    TakePlaceV { dst: u16, path: u16 },
    CallVmFeed { spec: u16 },
    FeedYieldI { src: u16, kind: ScalarKind },
    FeedYieldV { src: u16 },
    FeedYieldExtend { src: u16 },
    FeedBodyNext,
    FeedBodyBreak,
    FeedEnd,

    // RangeCheck, Unit, SetMin/SetMax, Reverse, BitWidth, IntToFloat, FloatSqrt, Map/Set
    RangeCheckI { dst: u16, val: u16, lo: u16, hi: u16, text_idx: u16 },
    ConstUnit { dst: u16 },
    SetMinMaxV { dst: u16, src_v: u16, is_max: bool },
    ReversePlace { path: u16 },
    BitWidthI { dst: u16, src: u16 },
    IntToFloat { dst: u16, src: u16 },
    FloatSqrt { dst: u16, src: u16 },
    ContainsV { dst: u16, src_v: u16, key_v: u16 },
    MapGetV { dst: u16, map_v: u16, key_v: u16 },
    MapIndexV { dst: u16, map_v: u16, key_v: u16, key_ty: u16 },
    SetMapMutPlace { dst: u16, path: u16, val_v: u16, is_remove: bool },
    MapPlaceAssign { path: u16, key_v: u16, val_v: u16, op: AssignOp, val_ty: u16 },
    SetMapBoundV { dst: u16, src_v: u16, key_v: u16, mode: u8 },
    SetMapPopEndPlace { dst: u16, path: u16, is_last: bool },
    LoadList1SmallTuple { dst_base: u16, len: u8, root: u16, is_global: bool, r_i: u16 },
    MinMaxArrOptI {
        root: u16,
        is_global: bool,
        is_2d: bool,
        is_max: bool,
        r_i: u16,
        r_j: u16,
        r_val: u16,
    },
    LoadIdxFieldIdxOptI {
        dst_val: u16,
        dst_some: u16,
        root: u16,
        is_global: bool,
        r_i: u16,
        field: u16,
        r_j: u16,
    },
    LoadNestedArrOptI {
        dst_val: u16,
        dst_some: u16,
        root: u16,
        is_global: bool,
        r_i: u16,
        r_j: u16,
    },
    LoadNestedArrI { dst: u16, root: u16, is_global: bool, r_i: u16, r_j: u16 },
    SeqIdxEndOptI {
        dst_val: u16,
        dst_some: u16,
        root: u16,
        is_global: bool,
        r_i: u16,
        is_last: bool,
    },

    // Seamless fallback to tree-walker for any non-lowered IR stmt/expr
    ConstV { dst: u16, idx: u32 },
    Print { idx: u32 },
    Format { idx: u32, dst: u16 },
    FallbackStmt { idx: u32, clear_from_v: u16, in_loop: bool },
    FallbackExprI { idx: u32, dst: u16, in_loop: bool },
    FallbackExprV { idx: u32, dst: u16, in_loop: bool },
    /// A traced run tells the tracer of a write the VM just did itself.
    Trace { idx: u32 },
}

/// A write a traced run reports: the same steps the tree-walker's
/// `Stmt::Let`, `Stmt::Assign` and `for` rounds record.
pub(crate) enum TracePoint<'p> {
    Let(Slot, &'p Expr),
    Assign(&'p Place),
    Loop(Slot, Span),
    /// After the loop the variable keeps its last value, not one past the end.
    LoopEnd(Slot),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum CapSrc {
    GlobalRef(u16),
    GlobalVal(u16),
    LocalI(u16, ScalarKind),
    LocalOptI { val: u16, some: u16, kind: ScalarKind },
    LocalSmallTuple { base: u16, len: u8, kinds: [SmallFieldKind; 4] },
    LocalV(u16),
}

#[derive(Clone, Debug)]
pub(crate) struct ClosureMakeSpec {
    pub cid: u32,
    pub caps: Vec<CapSrc>,
}

#[derive(Clone, Debug)]
pub(crate) struct FeedCallSpec {
    pub fn_id: FnId,
    pub args_i: Vec<u16>,
    pub args_v: Vec<u16>,
    pub body_pc: usize,
    pub var_loc: SlotLoc,
}

pub(crate) struct PrintSpec<'p> {
    pub args: &'p [FmtArg],
    pub regs: Vec<u16>,
    pub end: Option<u16>,
}

pub(crate) struct FormatSpec<'p> {
    pub pieces: &'p [FmtPiece],
    pub regs: Vec<u16>,
}

pub(crate) struct Chunk<'p> {
    pub ops: Vec<Op>,
    pub spans: Vec<Span>,
    pub consts_i: Vec<i64>,
    pub range_texts: Vec<&'p str>,
    pub places: Vec<PlacePath>,
    pub closure_specs: Vec<ClosureMakeSpec>,
    pub feed_calls: Vec<FeedCallSpec>,
    pub consts_v: Vec<Value>,
    pub prints: Vec<PrintSpec<'p>>,
    pub formats: Vec<FormatSpec<'p>>,
    pub fallback_stmts: Vec<&'p Stmt>,
    pub fallback_exprs: Vec<&'p Expr>,
    pub trace_points: Vec<TracePoint<'p>>,
    pub traced: bool,
    pub fill_tys: Vec<&'p Ty>,
    pub num_i: usize,
    pub num_v: usize,
    pub num_local_v: usize,
    pub slot_map: Vec<SlotLoc>,
    pub param_count: usize,
    pub ret_kind: Option<ScalarKind>,
    pub guarded_domains: Vec<cpl_ir::DomainId>,
    pub needs_stack: bool,
    pub has_by_ref_closure: bool,
}

#[derive(Clone, Copy)]
pub(crate) enum FastClosure {
    Param(u8),
    Add(u8, u8),
    Sub(u8, u8),
    Mul(u8, u8),
    Min(u8, u8),
    Max(u8, u8),
    BitAnd(u8, u8),
    BitOr(u8, u8),
    BitXor(u8, u8),
    MulAdd(u8, u8, u8),
}

fn recognize_fast_closure(def: &cpl_ir::ClosureDef, program: &Program) -> Option<FastClosure> {
    if !def.captures.is_empty()
        || !def.muts.is_empty()
        || def.params.len() > 4
        || !def.params.iter().enumerate().all(|(i, &s)| s == i)
        || ScalarKind::of_ty(&def.ret, program) != Some(ScalarKind::Int)
        || !def
            .params
            .iter()
            .all(|&s| ScalarKind::of_ty(&def.locals[s].ty, program) == Some(ScalarKind::Int))
    {
        return None;
    }
    if def.body.stmts.len() != 1 {
        return None;
    }
    let expr = match &def.body.stmts[0] {
        Stmt::Return(Some(e)) | Stmt::Expr(e) => e,
        _ => return None,
    };
    let param_of = |e: &Expr| -> Option<u8> {
        if let ExprKind::Local(s) = e.kind {
            if s < def.params.len() {
                return Some(s as u8);
            }
        }
        None
    };
    if let Some(p) = param_of(expr) {
        return Some(FastClosure::Param(p));
    }
    match &expr.kind {
        ExprKind::Binary(op, a, b) => {
            if let (Some(pa), Some(pb)) = (param_of(a), param_of(b)) {
                return match op {
                    BinOp::Add => Some(FastClosure::Add(pa, pb)),
                    BinOp::Sub => Some(FastClosure::Sub(pa, pb)),
                    BinOp::Mul => Some(FastClosure::Mul(pa, pb)),
                    BinOp::BitAnd => Some(FastClosure::BitAnd(pa, pb)),
                    BinOp::BitOr => Some(FastClosure::BitOr(pa, pb)),
                    BinOp::BitXor => Some(FastClosure::BitXor(pa, pb)),
                    _ => None,
                };
            }
            if matches!(op, BinOp::Add) {
                if let (Some(pc), ExprKind::Binary(BinOp::Mul, ma, mb)) = (param_of(a), &b.kind) {
                    if let (Some(pa), Some(pb)) = (param_of(ma), param_of(mb)) {
                        return Some(FastClosure::MulAdd(pa, pb, pc));
                    }
                }
                if let (ExprKind::Binary(BinOp::Mul, ma, mb), Some(pc)) = (&a.kind, param_of(b)) {
                    if let (Some(pa), Some(pb)) = (param_of(ma), param_of(mb)) {
                        return Some(FastClosure::MulAdd(pa, pb, pc));
                    }
                }
            }
            None
        }
        ExprKind::Builtin(Builtin::Min, args) if args.len() == 2 => {
            Some(FastClosure::Min(param_of(&args[0])?, param_of(&args[1])?))
        }
        ExprKind::Builtin(Builtin::Max, args) if args.len() == 2 => {
            Some(FastClosure::Max(param_of(&args[0])?, param_of(&args[1])?))
        }
        _ => None,
    }
}

pub(crate) struct CompiledProgram<'p> {
    pub main: Chunk<'p>,
    pub fns: Vec<Option<Chunk<'p>>>,
    pub fed_fns: Vec<Option<Chunk<'p>>>,
    pub closures: Vec<Option<Chunk<'p>>>,
    pub fast_closures: Vec<Option<FastClosure>>,
}

impl<'p> CompiledProgram<'p> {
    /// With `traced`, every write, print and call is reported to the
    /// tracer as the tree-walker reports it.
    pub fn compile(program: &'p Program, feeds: &cpl_ir::pushed::Feeds, traced: bool) -> Self {
        let empty_fed: Vec<bool> = vec![false; program.fns.len()];
        let mut fed_fns = Vec::with_capacity(program.fns.len());
        for (id, f) in program.fns.iter().enumerate() {
            if traced || f.locals.len() > 512 || program.judge_fns.contains(&id) {
                fed_fns.push(None);
            } else if let Some(&acc_slot) = feeds.pushers.get(&id) {
                let guarded_domains = program.fn_guarded_domains(&f.ret, &f.body);
                let mut fc =
                    Compiler::new(program, feeds, &empty_fed, &f.locals, Some(id), f.params.len());
                fc.feed_acc = Some(acc_slot);
                fc.allow_fed_loops = false;
                fc.compile_block(&f.body, None, false);
                fc.emit(Op::FeedEnd, Span::default());
                let mut fchunk = fc.finish();
                if fchunk.fallback_stmts.is_empty() && fchunk.fallback_exprs.is_empty() {
                    fchunk.param_count = f.params.len();
                    fchunk.ret_kind = None;
                    fchunk.guarded_domains = guarded_domains;
                    fchunk.has_by_ref_closure = block_has_by_ref_closure(&f.body, program);
                    if fchunk.has_by_ref_closure {
                        fchunk.needs_stack = true;
                    }
                    fed_fns.push(Some(fchunk));
                } else {
                    fed_fns.push(None);
                }
            } else {
                fed_fns.push(None);
            }
        }
        let fed_ok: Vec<bool> = fed_fns.iter().map(Option::is_some).collect();
        let mut fns = Vec::with_capacity(program.fns.len());
        for (id, f) in program.fns.iter().enumerate() {
            let guarded_domains = program.fn_guarded_domains(&f.ret, &f.body);
            // A traced `mut` parameter is the caller's variable, written in
            // place, which the VM only writes back on return.
            let traced_mut = traced && f.params.iter().any(|p| p.mutable);
            if f.locals.len() > 512 || program.judge_fns.contains(&id) || traced_mut {
                fns.push(None);
            } else {
                let mut c =
                    Compiler::new(program, feeds, &fed_ok, &f.locals, Some(id), f.params.len());
                c.traced = traced;
                let ret_kind = ScalarKind::of_ty(&f.ret, program);
                c.compile_block(&f.body, ret_kind, true);
                let mut chunk = c.finish();
                chunk.param_count = f.params.len();
                chunk.ret_kind = ret_kind;
                chunk.guarded_domains = guarded_domains;
                chunk.has_by_ref_closure = block_has_by_ref_closure(&f.body, program);
                if chunk.has_by_ref_closure {
                    chunk.needs_stack = true;
                }
                fns.push(Some(chunk));
            }
        }
        let mut closures = Vec::with_capacity(program.closures.len());
        let mut fast_closures = Vec::with_capacity(program.closures.len());
        for def in &program.closures {
            fast_closures.push(recognize_fast_closure(def, program));
            let can_compile = def.locals.len() <= 512
                && def.muts.is_empty()
                && def.captures.iter().all(|cap| !cap.by_ref)
                && def.params.iter().enumerate().all(|(i, &s)| s == i);
            if !can_compile {
                closures.push(None);
            } else {
                let guarded_domains = program.fn_guarded_domains(&def.ret, &def.body);
                let mut c =
                    Compiler::new(program, feeds, &fed_ok, &def.locals, None, def.params.len());
                c.allow_fed_loops = false;
                c.traced = traced;
                let ret_kind = ScalarKind::of_ty(&def.ret, program);
                c.compile_block(&def.body, ret_kind, true);
                let mut chunk = c.finish();
                chunk.param_count = def.params.len();
                chunk.ret_kind = ret_kind;
                chunk.guarded_domains = guarded_domains;
                chunk.has_by_ref_closure = block_has_by_ref_closure(&def.body, program);
                if chunk.has_by_ref_closure {
                    chunk.needs_stack = true;
                }
                closures.push(Some(chunk));
            }
        }
        let mut main_c = Compiler::new(program, feeds, &fed_ok, &program.main_locals, None, 0);
        main_c.traced = traced;
        main_c.compile_block(&program.main, None, false);
        main_c.emit(Op::RetUnit, Span::default());
        let mut main = main_c.finish();
        main.has_by_ref_closure = block_has_by_ref_closure(&program.main, program);
        if main.has_by_ref_closure {
            main.needs_stack = true;
        }
        CompiledProgram { main, fns, fed_fns, closures, fast_closures }
    }
}

struct LoopTargets {
    id: LoopId,
    breaks: Vec<usize>,
    continues: Vec<usize>,
}

enum GiveTarget {
    I { dst: u16, exits: Vec<usize> },
    OptI { dst_val: u16, dst_some: u16, exits: Vec<usize> },
    V { dst: u16, exits: Vec<usize> },
    Discard { exits: Vec<usize> },
}

impl GiveTarget {
    fn exits_mut(&mut self) -> &mut Vec<usize> {
        match self {
            GiveTarget::I { exits, .. }
            | GiveTarget::OptI { exits, .. }
            | GiveTarget::V { exits, .. }
            | GiveTarget::Discard { exits } => exits,
        }
    }
}

#[derive(Clone, Copy)]
enum ModSuperOp {
    Add(u32),
    Sub(u32),
    Mul(u32),
    Neg(u32),
    New(u32),
}

struct Compiler<'p, 'f> {
    program: &'p Program,
    feeds: &'f cpl_ir::pushed::Feeds,
    fed_ok: &'f [bool],
    ops: Vec<Op>,
    spans: Vec<Span>,
    consts_i: Vec<i64>,
    range_texts: Vec<&'p str>,
    places: Vec<PlacePath>,
    closure_specs: Vec<ClosureMakeSpec>,
    feed_calls: Vec<FeedCallSpec>,
    consts_v: Vec<Value>,
    prints: Vec<PrintSpec<'p>>,
    formats: Vec<FormatSpec<'p>>,
    fallback_stmts: Vec<&'p Stmt>,
    fallback_exprs: Vec<&'p Expr>,
    fill_tys: Vec<&'p Ty>,
    slot_map: Vec<SlotLoc>,
    next_i: u16,
    next_v: u16,
    num_local_v: u16,
    pinned_v: u16,
    loops: Vec<LoopTargets>,
    give_targets: Vec<GiveTarget>,
    cur_fn: Option<FnId>,
    feed_acc: Option<usize>,
    allow_fed_loops: bool,
    inline_slots: Option<Vec<SlotLoc>>,
    inline_ret: Option<(u16, Vec<usize>)>,
    traced: bool,
    trace_points: Vec<TracePoint<'p>>,
    /// The `for` whose body is compiled next, whose variable each round reports.
    traced_loop_var: Option<(Slot, Span)>,
}

impl<'p, 'f> Compiler<'p, 'f> {
    fn new(
        program: &'p Program,
        feeds: &'f cpl_ir::pushed::Feeds,
        fed_ok: &'f [bool],
        locals: &'p [Local],
        cur_fn: Option<FnId>,
        param_count: usize,
    ) -> Self {
        let mut slot_map = Vec::with_capacity(locals.len());
        let mut next_i: u16 = 0;
        let mut next_v: u16 = 0;
        for (idx, local) in locals.iter().enumerate() {
            if let Some(kind) = ScalarKind::of_ty(&local.ty, program) {
                slot_map.push(SlotLoc::I(next_i, kind));
                next_i += 1;
            } else if let Some(kind) = ScalarKind::of_opt_ty(&local.ty, program) {
                let val = next_i;
                let some = next_i + 1;
                next_i += 2;
                slot_map.push(SlotLoc::OptI { val, some, kind });
            } else if idx >= param_count {
                if let Some((len, kinds)) = small_tuple_shape(&local.ty, program) {
                    let base = next_i;
                    next_i += len as u16;
                    slot_map.push(SlotLoc::SmallTuple { base, len, kinds });
                } else {
                    slot_map.push(SlotLoc::V(next_v));
                    next_v += 1;
                }
            } else {
                slot_map.push(SlotLoc::V(next_v));
                next_v += 1;
            }
        }
        Self {
            program,
            feeds,
            fed_ok,
            ops: Vec::new(),
            spans: Vec::new(),
            consts_i: Vec::new(),
            range_texts: Vec::new(),
            places: Vec::new(),
            closure_specs: Vec::new(),
            feed_calls: Vec::new(),
            consts_v: Vec::new(),
            prints: Vec::new(),
            formats: Vec::new(),
            fallback_stmts: Vec::new(),
            fallback_exprs: Vec::new(),
            fill_tys: Vec::new(),
            slot_map,
            next_i,
            next_v,
            num_local_v: next_v,
            pinned_v: next_v,
            loops: Vec::new(),
            give_targets: Vec::new(),
            cur_fn,
            feed_acc: None,
            allow_fed_loops: true,
            inline_slots: None,
            inline_ret: None,
            traced: false,
            trace_points: Vec::new(),
            traced_loop_var: None,
        }
    }

    fn finish(self) -> Chunk<'p> {
        let needs_stack = !self.fallback_stmts.is_empty()
            || !self.fallback_exprs.is_empty()
            || !self.trace_points.is_empty();
        Chunk {
            ops: self.ops,
            spans: self.spans,
            consts_i: self.consts_i,
            range_texts: self.range_texts,
            places: self.places,
            closure_specs: self.closure_specs,
            feed_calls: self.feed_calls,
            consts_v: self.consts_v,
            prints: self.prints,
            formats: self.formats,
            fallback_stmts: self.fallback_stmts,
            fallback_exprs: self.fallback_exprs,
            trace_points: self.trace_points,
            traced: self.traced,
            fill_tys: self.fill_tys,
            num_i: self.next_i as usize,
            num_v: self.next_v as usize,
            num_local_v: self.num_local_v as usize,
            slot_map: self.slot_map,
            param_count: 0,
            ret_kind: None,
            guarded_domains: Vec::new(),
            needs_stack,
            has_by_ref_closure: false,
        }
    }

    fn const_bool(e: &Expr) -> Option<bool> {
        match &e.kind {
            ExprKind::Bool(b) => Some(*b),
            ExprKind::Unary(UnOp::Not, a) => Self::const_bool(a).map(|b| !b),
            ExprKind::Binary(BinOp::And, a, b) => Some(Self::const_bool(a)? && Self::const_bool(b)?),
            ExprKind::Binary(BinOp::Or, a, b) => Some(Self::const_bool(a)? || Self::const_bool(b)?),
            ExprKind::Cmp(op, a, b) => {
                if let (ExprKind::Int(av), ExprKind::Int(bv)) = (&a.kind, &b.kind) {
                    Some(match op {
                        CmpOp::Eq => av == bv,
                        CmpOp::Ne => av != bv,
                        CmpOp::Lt => av < bv,
                        CmpOp::Le => av <= bv,
                        CmpOp::Gt => av > bv,
                        CmpOp::Ge => av >= bv,
                        _ => return None,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn mut_fn_args_alias_free(&self, fn_id: FnId, args: &'p [Arg]) -> bool {
        let f = &self.program.fns[fn_id];
        fn place_root_global(p: &Place) -> Option<cpl_ir::GlobalId> {
            match &p.kind {
                PlaceKind::Global(g) => Some(*g),
                PlaceKind::Local(_) => None,
                PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                    place_root_global(base)
                }
            }
        }
        fn place_may_touch_globals(p: &Place, globals: &[cpl_ir::GlobalId]) -> bool {
            match &p.kind {
                PlaceKind::Global(g) => globals.contains(g),
                PlaceKind::Local(_) => false,
                PlaceKind::Index(base, idxs) => {
                    place_may_touch_globals(base, globals)
                        || idxs.iter().any(|e| expr_may_touch_globals(e, globals))
                }
                PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                    place_may_touch_globals(base, globals)
                }
            }
        }
        fn expr_may_touch_globals(e: &Expr, globals: &[cpl_ir::GlobalId]) -> bool {
            match &e.kind {
                ExprKind::Unit
                | ExprKind::Int(_)
                | ExprKind::Float(_)
                | ExprKind::Bool(_)
                | ExprKind::Char(_)
                | ExprKind::Str(_)
                | ExprKind::None
                | ExprKind::Local(_)
                | ExprKind::DomainVal(_)
                | ExprKind::DomainLen(_)
                | ExprKind::DomainLo(_)
                | ExprKind::Error => false,
                ExprKind::Global(g) => globals.contains(g),
                ExprKind::Some(x)
                | ExprKind::Unary(_, x)
                | ExprKind::Cast(_, x)
                | ExprKind::Field(x, _)
                | ExprKind::IsVariant(x, _)
                | ExprKind::Payload(x, _, _)
                | ExprKind::IsSome(x)
                | ExprKind::Unwrap(x, _) => expr_may_touch_globals(x, globals),
                ExprKind::Binary(_, a, b)
                | ExprKind::Cmp(_, a, b)
                | ExprKind::Range { lo: a, hi: b } => {
                    expr_may_touch_globals(a, globals) || expr_may_touch_globals(b, globals)
                }
                ExprKind::RangeCheck { value, lo, hi, .. } => {
                    expr_may_touch_globals(value, globals)
                        || lo.as_deref().is_some_and(|x| expr_may_touch_globals(x, globals))
                        || hi.as_deref().is_some_and(|x| expr_may_touch_globals(x, globals))
                }
                ExprKind::If(c, t, f) => {
                    expr_may_touch_globals(c, globals)
                        || expr_may_touch_globals(t, globals)
                        || expr_may_touch_globals(f, globals)
                }
                ExprKind::Index(b, idxs) => {
                    expr_may_touch_globals(b, globals)
                        || idxs.iter().any(|x| expr_may_touch_globals(x, globals))
                }
                ExprKind::Tuple(xs) | ExprKind::Variant(_, xs) | ExprKind::Builtin(_, xs) => {
                    xs.iter().any(|x| expr_may_touch_globals(x, globals))
                }
                ExprKind::Do(blk) => block_may_touch_globals(blk, globals),
                ExprKind::Call(..)
                | ExprKind::CallExpr(..)
                | ExprKind::Closure(..)
                | ExprKind::Mutate(..)
                | ExprKind::Sort(..)
                | ExprKind::Read(..)
                | ExprKind::Parse(..)
                | ExprKind::Print(..)
                | ExprKind::Debug(..)
                | ExprKind::Format(..)
                | ExprKind::Fill(..)
                | ExprKind::MutSlice(..) => true,
            }
        }
        fn block_may_touch_globals(b: &Block, globals: &[cpl_ir::GlobalId]) -> bool {
            b.stmts.iter().any(|s| match s {
                Stmt::Let(_, init) => init.as_ref().is_some_and(|e| expr_may_touch_globals(e, globals)),
                Stmt::Assign(place, _, rhs) => {
                    place_may_touch_globals(place, globals) || expr_may_touch_globals(rhs, globals)
                }
                Stmt::Expr(e) | Stmt::Give(e) => expr_may_touch_globals(e, globals),
                Stmt::Return(e) => e.as_ref().is_some_and(|e| expr_may_touch_globals(e, globals)),
                Stmt::If(c, t, f) => {
                    expr_may_touch_globals(c, globals)
                        || block_may_touch_globals(t, globals)
                        || block_may_touch_globals(f, globals)
                }
                Stmt::While(_, c, body) => {
                    expr_may_touch_globals(c, globals) || block_may_touch_globals(body, globals)
                }
                Stmt::Assert { cond, message, .. } => {
                    expr_may_touch_globals(cond, globals)
                        || message.as_ref().is_some_and(|e| expr_may_touch_globals(e, globals))
                }
                Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => false,
                Stmt::For(..) | Stmt::DefDomain { .. } | Stmt::DefDomainFromVal { .. } => true,
            })
        }

        let mut mut_globals = Vec::new();
        for arg in args {
            if let Arg::Mut(place) = arg {
                if let Some(g) = place_root_global(place) {
                    mut_globals.push(g);
                }
            }
        }
        for (i, arg) in args.iter().enumerate() {
            if !matches!(f.locals[i].ty, Ty::Fn(..)) {
                continue;
            }
            let Arg::Value(ae) = arg else {
                return false;
            };
            let ExprKind::Closure(cid) = &ae.kind else {
                return false;
            };
            let cdef = &self.program.closures[*cid];
            if cdef.captures.iter().any(|c| c.by_ref) {
                return false;
            }
            if !mut_globals.is_empty() && block_may_touch_globals(&cdef.body, &mut_globals) {
                return false;
            }
        }
        true
    }

    fn try_compile_call_mix(
        &mut self,
        fn_id: FnId,
        args: &'p [Arg],
        ret_mode: CallRet,
        span: Span,
    ) -> Option<(u16, u16, u16)> {
        let f = &self.program.fns[fn_id];
        let has_mut = f.params.iter().any(|p| p.mutable);
        let has_fn_param = f
            .locals
            .iter()
            .take(f.params.len())
            .any(|l| matches!(l.ty, Ty::Fn(..)));
        if f.locals.len() > 512
            || self.program.judge_fns.contains(&fn_id)
            || args.len() != f.params.len()
            || (has_mut && has_fn_param && !self.mut_fn_args_alias_free(fn_id, args))
        {
            return None;
        }
        // Check that all mut args are simple local/global places or a single 1D indexed V place.
        let mut idx_mut_count = 0usize;
        for (i, a) in args.iter().enumerate() {
            match a {
                Arg::Value(_) => {
                    if f.params[i].mutable {
                        return None;
                    }
                }
                Arg::Mut(place) => {
                    if !f.params[i].mutable {
                        return None;
                    }
                    match &place.kind {
                        PlaceKind::Local(slot) => {
                            if matches!(self.slot_loc(*slot), SlotLoc::SmallTuple { .. }) {
                                return None;
                            }
                        }
                        PlaceKind::Global(_) => {}
                        PlaceKind::Index(base, indices)
                            if indices.len() == 1
                                && idx_mut_count == 0
                                && ScalarKind::of_ty(&indices[0].ty, self.program).is_some()
                                && ScalarKind::of_ty(&f.locals[i].ty, self.program).is_none()
                                && ScalarKind::of_opt_ty(&f.locals[i].ty, self.program).is_none() =>
                        {
                            let base_ok = match &base.kind {
                                PlaceKind::Local(slot) => {
                                    matches!(self.slot_loc(*slot), SlotLoc::V(_))
                                }
                                PlaceKind::Global(_) => true,
                                _ => false,
                            };
                            let container_ok = match &base.ty {
                                Ty::Array { dims, .. } => {
                                    dims.len() == 1
                                        && !matches!(
                                            dims[0],
                                            cpl_ir::Dim::IntFrom(_) | cpl_ir::Dim::Sym(_)
                                        )
                                }
                                Ty::List(_) => true,
                                _ => false,
                            };
                            if !base_ok || !container_ok {
                                return None;
                            }
                            idx_mut_count += 1;
                        }
                        _ => return None,
                    }
                }
            }
        }
        if idx_mut_count > 0 && args.iter().filter(|a| matches!(a, Arg::Mut(_))).count() > 1 {
            return None;
        }
        let mut ni = 0usize;
        let mut nv = 0usize;
        for local in f.locals.iter().take(f.params.len()) {
            let ty = &local.ty;
            if ScalarKind::of_ty(ty, self.program).is_some() {
                ni += 1;
            } else if ScalarKind::of_opt_ty(ty, self.program).is_some() {
                ni += 2;
            } else {
                nv += 1;
            }
        }
        if ni > 8 || nv > 8 {
            return None;
        }
        enum WriteBack {
            LocalI { dst: u16, arg_off: u16 },
            LocalOptI { dst_val: u16, dst_some: u16, arg_off: u16 },
            LocalV { dst: u16, arg_off: u16 },
            GlobalI { gid: u16, kind: ScalarKind, arg_off: u16 },
            GlobalOptI { gid: u16, kind: ScalarKind, arg_off: u16 },
            GlobalV { gid: u16, arg_off: u16 },
            LocalArr1V { arr_v: u16, r_i: u16, arg_off: u16 },
            GlobalArr1V { gid: u16, r_i: u16, arg_off: u16 },
            LocalList1V { list_v: u16, r_i: u16, arg_off: u16 },
            GlobalList1V { gid: u16, r_i: u16, arg_off: u16 },
        }
        let mut i_args: Vec<u16> = vec![0; ni];
        let mut v_args: Vec<(u16, bool)> = vec![(0, false); nv];
        let mut writebacks: Vec<WriteBack> = Vec::new();
        // Precompute argument slot offsets so all Arg::Value expressions are evaluated
        // before any Arg::Mut place is moved out (e.g. `a.union(a.shifted(w))`).
        let mut param_offs: Vec<usize> = Vec::with_capacity(args.len());
        let mut cur_i = 0usize;
        let mut cur_v = 0usize;
        for local in f.locals.iter().take(f.params.len()) {
            let ty = &local.ty;
            if ScalarKind::of_ty(ty, self.program).is_some() {
                param_offs.push(cur_i);
                cur_i += 1;
            } else if ScalarKind::of_opt_ty(ty, self.program).is_some() {
                param_offs.push(cur_i);
                cur_i += 2;
            } else {
                param_offs.push(cur_v);
                cur_v += 1;
            }
        }
        // Pass 1: evaluate all value arguments first.
        for (i, a) in args.iter().enumerate() {
            if let Arg::Value(ae) = a {
                let ty = &f.locals[i].ty;
                let off = param_offs[i];
                if ScalarKind::of_ty(ty, self.program).is_some() {
                    i_args[off] = self.compile_expr_i(ae);
                } else if ScalarKind::of_opt_ty(ty, self.program).is_some() {
                    let (v, s) = self.compile_expr_opt_i(ae);
                    i_args[off] = v;
                    i_args[off + 1] = s;
                } else {
                    v_args[off] = (self.compile_expr_v(ae), false);
                }
            }
        }
        // Pass 2: move out all mut arguments after value arguments are evaluated.
        for (i, a) in args.iter().enumerate() {
            if let Arg::Mut(place) = a {
                let ty = &f.locals[i].ty;
                let off = param_offs[i];
                match &place.kind {
                    PlaceKind::Local(slot) => match self.slot_loc(*slot) {
                        SlotLoc::I(r, _) => {
                            i_args[off] = r;
                            writebacks.push(WriteBack::LocalI { dst: r, arg_off: off as u16 });
                        }
                        SlotLoc::OptI { val, some, .. } => {
                            i_args[off] = val;
                            i_args[off + 1] = some;
                            writebacks.push(WriteBack::LocalOptI {
                                dst_val: val,
                                dst_some: some,
                                arg_off: off as u16,
                            });
                        }
                        SlotLoc::V(r) => {
                            v_args[off] = (r, true);
                            writebacks.push(WriteBack::LocalV { dst: r, arg_off: off as u16 });
                        }
                        SlotLoc::SmallTuple { .. } => unreachable!(),
                    },
                    PlaceKind::Global(gid) => {
                        let gid = *gid as u16;
                        if let Some(kind) = ScalarKind::of_ty(ty, self.program) {
                            let r = self.alloc_i();
                            self.emit(Op::LoadGlobalI { dst: r, gid }, place.span);
                            i_args[off] = r;
                            writebacks.push(WriteBack::GlobalI { gid, kind, arg_off: off as u16 });
                        } else if let Some(kind) = ScalarKind::of_opt_ty(ty, self.program) {
                            let rv = self.alloc_i();
                            let rs = self.alloc_i();
                            self.emit(
                                Op::LoadGlobalOptI { dst_val: rv, dst_some: rs, gid },
                                place.span,
                            );
                            i_args[off] = rv;
                            i_args[off + 1] = rs;
                            writebacks.push(WriteBack::GlobalOptI {
                                gid,
                                kind,
                                arg_off: off as u16,
                            });
                        } else {
                            let r = self.alloc_v();
                            self.emit(Op::LoadGlobalVMove { dst: r, gid }, place.span);
                            v_args[off] = (r, true);
                            writebacks.push(WriteBack::GlobalV { gid, arg_off: off as u16 });
                        }
                    }
                    PlaceKind::Index(base, indices) => {
                        let is_arr = matches!(&base.ty, Ty::Array { .. });
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r = self.alloc_v();
                        match (&base.kind, is_arr) {
                            (PlaceKind::Local(slot), true) => {
                                let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                                    unreachable!();
                                };
                                self.emit(Op::TakeLArr1V { dst: r, arr_v, r_i }, place.span);
                                writebacks.push(WriteBack::LocalArr1V {
                                    arr_v,
                                    r_i,
                                    arg_off: off as u16,
                                });
                            }
                            (PlaceKind::Global(gid), true) => {
                                let gid = *gid as u16;
                                self.emit(Op::TakeGArr1V { dst: r, gid, r_i }, place.span);
                                writebacks.push(WriteBack::GlobalArr1V {
                                    gid,
                                    r_i,
                                    arg_off: off as u16,
                                });
                            }
                            (PlaceKind::Local(slot), false) => {
                                let SlotLoc::V(list_v) = self.slot_loc(*slot) else {
                                    unreachable!();
                                };
                                self.emit(Op::TakeLList1V { dst: r, list_v, r_i }, place.span);
                                writebacks.push(WriteBack::LocalList1V {
                                    list_v,
                                    r_i,
                                    arg_off: off as u16,
                                });
                            }
                            (PlaceKind::Global(gid), false) => {
                                let gid = *gid as u16;
                                self.emit(Op::TakeGList1V { dst: r, gid, r_i }, place.span);
                                writebacks.push(WriteBack::GlobalList1V {
                                    gid,
                                    r_i,
                                    arg_off: off as u16,
                                });
                            }
                            _ => unreachable!(),
                        }
                        v_args[off] = (r, true);
                    }
                    _ => unreachable!(),
                }
            }
        }
        let arg_base_i = self.next_i;
        for r in i_args {
            let slot = self.alloc_i();
            if slot != r {
                self.emit(Op::MoveI { dst: slot, src: r }, span);
            }
        }
        let arg_base_v = self.next_v;
        for (r, consume) in v_args {
            let slot = self.alloc_v();
            if slot != r {
                if consume {
                    self.emit(Op::MoveVConsume { dst: slot, src: r }, span);
                } else {
                    self.emit(Op::MoveV { dst: slot, src: r }, span);
                }
            }
        }
        let (dst_i, dst_v, dst_some) = match ret_mode {
            CallRet::Unit => (0, 0, 0),
            CallRet::I => (self.alloc_i(), 0, 0),
            CallRet::OptI => (self.alloc_i(), 0, self.alloc_i()),
            CallRet::V => (0, self.alloc_v(), 0),
        };
        self.emit(
            Op::CallVmMix {
                fn_id: fn_id as u16,
                arg_base_i,
                arg_base_v,
                dst_i,
                dst_v,
                dst_some,
                ret_mode,
            },
            span,
        );
        for wb in writebacks {
            match wb {
                WriteBack::LocalI { dst, arg_off } => {
                    let src = arg_base_i + arg_off;
                    if dst != src {
                        self.emit(Op::MoveI { dst, src }, span);
                    }
                }
                WriteBack::LocalOptI { dst_val, dst_some, arg_off } => {
                    let sv = arg_base_i + arg_off;
                    let ss = arg_base_i + arg_off + 1;
                    if dst_val != sv {
                        self.emit(Op::MoveI { dst: dst_val, src: sv }, span);
                    }
                    if dst_some != ss {
                        self.emit(Op::MoveI { dst: dst_some, src: ss }, span);
                    }
                }
                WriteBack::LocalV { dst, arg_off } => {
                    let src = arg_base_v + arg_off;
                    if dst != src {
                        self.emit(Op::MoveVConsume { dst, src }, span);
                    }
                }
                WriteBack::GlobalI { gid, kind, arg_off } => {
                    self.emit(Op::StoreGlobalI { gid, src: arg_base_i + arg_off, kind }, span);
                }
                WriteBack::GlobalOptI { gid, kind, arg_off } => {
                    self.emit(
                        Op::StoreGlobalOptI {
                            gid,
                            val: arg_base_i + arg_off,
                            some: arg_base_i + arg_off + 1,
                            kind,
                        },
                        span,
                    );
                }
                WriteBack::GlobalV { gid, arg_off } => {
                    self.emit(Op::StoreGlobalVMove { gid, src: arg_base_v + arg_off }, span);
                }
                WriteBack::LocalArr1V { arr_v, r_i, arg_off } => {
                    self.emit(
                        Op::StoreLArr1V { arr_v, r_i, r_val: arg_base_v + arg_off },
                        span,
                    );
                }
                WriteBack::GlobalArr1V { gid, r_i, arg_off } => {
                    self.emit(
                        Op::StoreGArr1V { gid, r_i, r_val: arg_base_v + arg_off },
                        span,
                    );
                }
                WriteBack::LocalList1V { list_v, r_i, arg_off } => {
                    self.emit(
                        Op::StoreLList1V { list_v, r_i, r_val: arg_base_v + arg_off },
                        span,
                    );
                }
                WriteBack::GlobalList1V { gid, r_i, arg_off } => {
                    self.emit(
                        Op::StoreGList1V { gid, r_i, r_val: arg_base_v + arg_off },
                        span,
                    );
                }
            }
        }
        Some((dst_i, dst_v, dst_some))
    }

    fn try_compile_call_closure(
        &mut self,
        callee: &'p Expr,
        args: &'p [Arg],
        ret_mode: CallRet,
        span: Span,
    ) -> Option<(u16, u16, u16)> {
        let Ty::Fn(params, _) = &callee.ty else {
            return None;
        };
        if args.len() != params.len() || !args.iter().all(|a| matches!(a, Arg::Value(_))) {
            return None;
        }
        let mut ni = 0usize;
        let mut nv = 0usize;
        for ty in params {
            if ScalarKind::of_ty(ty, self.program).is_some() {
                ni += 1;
            } else if ScalarKind::of_opt_ty(ty, self.program).is_some() {
                ni += 2;
            } else {
                nv += 1;
            }
        }
        if ni > 8 || nv > 8 {
            return None;
        }
        let callee_v = self.compile_expr_v(callee);
        let mut i_args = Vec::with_capacity(ni);
        let mut v_args = Vec::with_capacity(nv);
        for (ty, a) in params.iter().zip(args.iter()) {
            let Arg::Value(ae) = a else { unreachable!() };
            if ScalarKind::of_ty(ty, self.program).is_some() {
                i_args.push(self.compile_expr_i(ae));
            } else if ScalarKind::of_opt_ty(ty, self.program).is_some() {
                let (v, s) = self.compile_expr_opt_i(ae);
                i_args.push(v);
                i_args.push(s);
            } else {
                v_args.push(self.compile_expr_v(ae));
            }
        }
        let arg_base_i = self.next_i;
        for r in i_args {
            let slot = self.alloc_i();
            if slot != r {
                self.emit(Op::MoveI { dst: slot, src: r }, span);
            }
        }
        let arg_base_v = self.next_v;
        for r in v_args {
            let slot = self.alloc_v();
            if slot != r {
                self.emit(Op::MoveV { dst: slot, src: r }, span);
            }
        }
        let (dst_i, dst_v, dst_some) = match ret_mode {
            CallRet::Unit => (0, 0, 0),
            CallRet::I => (self.alloc_i(), 0, 0),
            CallRet::OptI => (self.alloc_i(), 0, self.alloc_i()),
            CallRet::V => (0, self.alloc_v(), 0),
        };
        self.emit(
            Op::CallClosure {
                callee_v,
                arg_base_i,
                arg_base_v,
                dst_i,
                dst_v,
                dst_some,
                ret_mode,
            },
            span,
        );
        Some((dst_i, dst_v, dst_some))
    }

    fn alloc_i(&mut self) -> u16 {
        let r = self.next_i;
        self.next_i += 1;
        r
    }

    fn alloc_v(&mut self) -> u16 {
        let r = self.next_v;
        self.next_v += 1;
        r
    }

    fn slot_loc(&self, slot: Slot) -> SlotLoc {
        if let Some(map) = &self.inline_slots {
            map[slot]
        } else {
            self.slot_map[slot]
        }
    }

    fn emit(&mut self, op: Op, span: Span) -> usize {
        let pc = self.ops.len();
        self.ops.push(op);
        self.spans.push(span);
        pc
    }

    fn patch(&mut self, pc: usize, target: u32) {
        match &mut self.ops[pc] {
            Op::Jmp { target: t }
            | Op::JmpEscape { target: t, .. }
            | Op::JmpIfZero { target: t, .. }
            | Op::JmpIfNonZero { target: t, .. }
            | Op::JmpLtI { target: t, .. }
            | Op::JmpLeI { target: t, .. }
            | Op::JmpGtI { target: t, .. }
            | Op::JmpGeI { target: t, .. }
            | Op::JmpEqI { target: t, .. }
            | Op::JmpNeI { target: t, .. }
            | Op::ForInc { target: t, .. }
            | Op::ForIncClosed { target: t, .. }
            | Op::ForDec { target: t, .. } => *t = target,
            _ => unreachable!("patch on non-jump op"),
        }
    }

    fn emit_const_i(&mut self, dst: u16, val: i64, span: Span) {
        if let Ok(imm) = i32::try_from(val) {
            self.emit(Op::ConstImmI { dst, val: imm }, span);
        } else {
            let idx = self.consts_i.len() as u32;
            self.consts_i.push(val);
            self.emit(Op::ConstI { dst, idx }, span);
        }
    }

    fn recognize_mod_op(program: &Program, fn_id: FnId) -> Option<ModSuperOp> {
        let f = &program.fns[fn_id];
        if f.params.iter().any(|p| p.mutable) || f.body.stmts.len() != 1 {
            return None;
        }
        let ret_expr = match &f.body.stmts[0] {
            Stmt::Return(Some(e)) | Stmt::Give(e) => e,
            _ => return None,
        };
        let inner = match &ret_expr.kind {
            ExprKind::Tuple(fs) if fs.len() == 1 => &fs[0],
            _ => ret_expr,
        };
        let is_param_field = |e: &Expr, p: Slot| -> bool {
            match &e.kind {
                ExprKind::Field(base, 0) => matches!(base.kind, ExprKind::Local(s) if s == p),
                ExprKind::Local(s) => *s == p,
                _ => false,
            }
        };
        if f.params.len() == 1 {
            if let ExprKind::Binary(BinOp::Rem, a, m_e) = &inner.kind {
                if let ExprKind::Int(m) = m_e.kind {
                    if m > 0 && m <= u32::MAX as i64 && is_param_field(a, 0) {
                        return Some(ModSuperOp::New(m as u32));
                    }
                }
            }
        }
        let ExprKind::If(cond, then_e, else_e) = &inner.kind else {
            return None;
        };
        if f.params.len() == 2 {
            if let (
                ExprKind::Cmp(CmpOp::Ge, lhs, rhs),
                ExprKind::Binary(BinOp::Sub, t_lhs, t_rhs),
                ExprKind::Binary(BinOp::Add, e_a, e_b),
            ) = (&cond.kind, &then_e.kind, &else_e.kind)
            {
                if let (
                    ExprKind::Binary(BinOp::Add, c_a, c_b),
                    ExprKind::Int(m1),
                    ExprKind::Binary(BinOp::Add, t_a, t_b),
                    ExprKind::Int(m2),
                ) = (&lhs.kind, &rhs.kind, &t_lhs.kind, &t_rhs.kind)
                {
                    if m1 == m2
                        && *m1 > 0
                        && *m1 <= u32::MAX as i64
                        && is_param_field(c_a, 0)
                        && is_param_field(c_b, 1)
                        && is_param_field(t_a, 0)
                        && is_param_field(t_b, 1)
                        && is_param_field(e_a, 0)
                        && is_param_field(e_b, 1)
                    {
                        return Some(ModSuperOp::Add(*m1 as u32));
                    }
                }
            }
            if let (
                ExprKind::Cmp(CmpOp::Ge, c_a, c_b),
                ExprKind::Binary(BinOp::Sub, t_a, t_b),
                ExprKind::Binary(BinOp::Add, e_lhs, e_rhs),
            ) = (&cond.kind, &then_e.kind, &else_e.kind)
            {
                if let (ExprKind::Binary(BinOp::Sub, e_a, e_b), ExprKind::Int(m)) =
                    (&e_lhs.kind, &e_rhs.kind)
                {
                    if *m > 0
                        && *m <= u32::MAX as i64
                        && is_param_field(c_a, 0)
                        && is_param_field(c_b, 1)
                        && is_param_field(t_a, 0)
                        && is_param_field(t_b, 1)
                        && is_param_field(e_a, 0)
                        && is_param_field(e_b, 1)
                    {
                        return Some(ModSuperOp::Sub(*m as u32));
                    }
                }
            }
            if Self::const_bool(cond) == Some(true) {
                if let ExprKind::Binary(BinOp::Rem, mul_e, mod_e) = &then_e.kind {
                    if let (ExprKind::Binary(BinOp::Mul, m_a, m_b), ExprKind::Int(m)) =
                        (&mul_e.kind, &mod_e.kind)
                    {
                        if *m > 0
                            && *m <= 3_037_000_499
                            && is_param_field(m_a, 0)
                            && is_param_field(m_b, 1)
                        {
                            return Some(ModSuperOp::Mul(*m as u32));
                        }
                    }
                }
            }
        } else if f.params.len() == 1 {
            if let (
                ExprKind::Cmp(CmpOp::Eq, c_a, c_z),
                ExprKind::Int(0),
                ExprKind::Binary(BinOp::Sub, e_m, e_a),
            ) = (&cond.kind, &then_e.kind, &else_e.kind)
            {
                if let (ExprKind::Int(0), ExprKind::Int(m)) = (&c_z.kind, &e_m.kind) {
                    if *m > 0
                        && *m <= u32::MAX as i64
                        && is_param_field(c_a, 0)
                        && is_param_field(e_a, 0)
                    {
                        return Some(ModSuperOp::Neg(*m as u32));
                    }
                }
            }
        }
        None
    }

    fn is_pure_leaf_fn(&self, fn_id: FnId) -> bool {
        if self.cur_fn == Some(fn_id)
            || self.inline_slots.is_some()
            || self.program.judge_fns.contains(&fn_id)
        {
            return false;
        }
        let f = &self.program.fns[fn_id];
        if f.params.iter().any(|p| p.mutable) {
            return false;
        }
        if f.locals.len() > 24 || f.body.stmts.len() > 12 {
            return false;
        }
        if !self.program.fn_guarded_domains(&f.ret, &f.body).is_empty() {
            return false;
        }
        if ScalarKind::of_ty(&f.ret, self.program).is_none() {
            return false;
        }
        if !f
            .locals
            .iter()
            .skip(f.params.len())
            .all(|l| ScalarKind::of_ty(&l.ty, self.program).is_some())
        {
            return false;
        }
        fn simple_block(b: &Block, f: &cpl_ir::Function, program: &Program) -> bool {
            b.stmts.iter().all(|s| simple_stmt(s, f, program))
        }
        fn simple_stmt(s: &Stmt, f: &cpl_ir::Function, program: &Program) -> bool {
            match s {
                Stmt::Let(slot, e) => {
                    ScalarKind::of_ty(&f.locals[*slot].ty, program).is_some()
                        && e.as_ref().is_none_or(|e| simple_expr(e, f, program))
                }
                Stmt::Assign(p, _, e) => {
                    matches!(p.kind, PlaceKind::Local(slot) if ScalarKind::of_ty(&f.locals[slot].ty, program).is_some())
                        && simple_expr(e, f, program)
                }
                Stmt::Return(Some(e)) | Stmt::Give(e) | Stmt::Expr(e) => {
                    simple_expr(e, f, program)
                }
                Stmt::If(c, t, e) => {
                    if let Some(b) = Compiler::const_bool(c) {
                        if b { simple_block(t, f, program) } else { simple_block(e, f, program) }
                    } else {
                        simple_expr(c, f, program)
                            && simple_block(t, f, program)
                            && simple_block(e, f, program)
                    }
                }
                _ => false,
            }
        }
        fn simple_v_base(e: &Expr, f: &cpl_ir::Function) -> bool {
            match &e.kind {
                ExprKind::Global(_) => true,
                ExprKind::Local(slot) => *slot < f.params.len(),
                ExprKind::Field(inner, _) => simple_v_base(inner, f),
                _ => false,
            }
        }
        fn simple_expr(e: &Expr, f: &cpl_ir::Function, program: &Program) -> bool {
            if ScalarKind::of_ty(&e.ty, program).is_none() {
                return false;
            }
            match &e.kind {
                ExprKind::Int(_)
                | ExprKind::Bool(_)
                | ExprKind::Char(_)
                | ExprKind::Local(_)
                | ExprKind::Global(_) => true,
                ExprKind::Unary(_, a) | ExprKind::Cast(_, a) => simple_expr(a, f, program),
                ExprKind::Field(a, _) => simple_expr(a, f, program) || simple_v_base(a, f),
                ExprKind::Binary(op, a, b) => {
                    let safe_op = match op {
                        BinOp::Div | BinOp::Rem => matches!(b.kind, ExprKind::Int(v) if v != 0),
                        BinOp::Shl | BinOp::Shr => {
                            matches!(b.kind, ExprKind::Int(v) if (0..64).contains(&v))
                        }
                        _ => true,
                    };
                    safe_op && simple_expr(a, f, program) && simple_expr(b, f, program)
                }
                ExprKind::Cmp(_, a, b) => {
                    simple_expr(a, f, program) && simple_expr(b, f, program)
                }
                ExprKind::Tuple(fs) => fs.len() == 1 && simple_expr(&fs[0], f, program),
                ExprKind::Index(base, indices) => {
                    let container_ok = match &base.ty {
                        Ty::Array { dims, .. } => {
                            (indices.len() == 1 || indices.len() == 2)
                                && dims.len() == indices.len()
                        }
                        Ty::List(_) => indices.len() == 1,
                        _ => false,
                    };
                    container_ok
                        && simple_v_base(base, f)
                        && indices.iter().all(|i| simple_expr(i, f, program))
                }
                ExprKind::Call(callee, args) => {
                    Compiler::recognize_mod_op(program, *callee).is_some()
                        && args
                            .iter()
                            .all(|a| matches!(a, Arg::Value(e) if simple_expr(e, f, program)))
                }
                ExprKind::If(c, t, e) => {
                    if let Some(b) = Compiler::const_bool(c) {
                        if b { simple_expr(t, f, program) } else { simple_expr(e, f, program) }
                    } else {
                        simple_expr(c, f, program)
                            && simple_expr(t, f, program)
                            && simple_expr(e, f, program)
                    }
                }
                ExprKind::Do(b) => simple_block(b, f, program),
                _ => false,
            }
        }
        simple_block(&f.body, f, self.program)
    }

    fn stmt_mentions_slot(stmt: &Stmt, target: Slot) -> bool {
        match stmt {
            Stmt::Let(s, init) => {
                *s == target || init.as_ref().is_some_and(|e| Self::expr_mentions_slot(e, target))
            }
            Stmt::Assign(p, _, e) => {
                Self::place_mentions_slot(p, target) || Self::expr_mentions_slot(e, target)
            }
            Stmt::Expr(e) | Stmt::Give(e) => Self::expr_mentions_slot(e, target),
            Stmt::Return(e) => e.as_ref().is_some_and(|e| Self::expr_mentions_slot(e, target)),
            Stmt::If(c, t, e) => {
                Self::expr_mentions_slot(c, target)
                    || t.stmts.iter().any(|s| Self::stmt_mentions_slot(s, target))
                    || e.stmts.iter().any(|s| Self::stmt_mentions_slot(s, target))
            }
            Stmt::While(_, c, b) => {
                Self::expr_mentions_slot(c, target)
                    || b.stmts.iter().any(|s| Self::stmt_mentions_slot(s, target))
            }
            Stmt::For(_, var, iter, body) => {
                *var == target
                    || Self::iter_mentions_slot(iter, target)
                    || body.stmts.iter().any(|s| Self::stmt_mentions_slot(s, target))
            }
            Stmt::Assert { cond, message, .. } => {
                Self::expr_mentions_slot(cond, target)
                    || message.as_ref().is_some_and(|m| Self::expr_mentions_slot(m, target))
            }
            Stmt::DefDomain { lo, len, .. } => {
                Self::expr_mentions_slot(lo, target) || Self::expr_mentions_slot(len, target)
            }
            Stmt::DefDomainFromVal { val, .. } => Self::expr_mentions_slot(val, target),
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(_, _) => false,
        }
    }

    fn iter_mentions_slot(iter: &Iter, target: Slot) -> bool {
        match iter {
            Iter::Range { lo, hi, .. } => {
                Self::expr_mentions_slot(lo, target) || Self::expr_mentions_slot(hi, target)
            }
            Iter::RangeVal(e, _) | Iter::Seq(e, _) | Iter::Subsets(e) | Iter::Members(e) => {
                Self::expr_mentions_slot(e, target)
            }
            Iter::TreeRange { container, lo, hi, .. } => {
                Self::expr_mentions_slot(container, target)
                    || lo.as_ref().is_some_and(|(e, _)| Self::expr_mentions_slot(e, target))
                    || hi.as_ref().is_some_and(|(e, _)| Self::expr_mentions_slot(e, target))
            }
        }
    }

    fn place_mentions_slot(p: &Place, target: Slot) -> bool {
        match &p.kind {
            PlaceKind::Local(s) => *s == target,
            PlaceKind::Global(_) => false,
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                Self::place_mentions_slot(base, target)
            }
            PlaceKind::Index(base, idxs) => {
                Self::place_mentions_slot(base, target)
                    || idxs.iter().any(|i| Self::expr_mentions_slot(i, target))
            }
        }
    }

    fn expr_mentions_slot(e: &Expr, target: Slot) -> bool {
        match &e.kind {
            ExprKind::Local(s) => *s == target,
            ExprKind::Closure(_) => true,
            ExprKind::Unary(_, a)
            | ExprKind::Cast(_, a)
            | ExprKind::Field(a, _)
            | ExprKind::Some(a)
            | ExprKind::IsSome(a)
            | ExprKind::IsVariant(a, _)
            | ExprKind::Payload(a, _, _)
            | ExprKind::Unwrap(a, _) => Self::expr_mentions_slot(a, target),
            ExprKind::Binary(_, a, b) | ExprKind::Cmp(_, a, b) => {
                Self::expr_mentions_slot(a, target) || Self::expr_mentions_slot(b, target)
            }
            ExprKind::Range { lo, hi } => {
                Self::expr_mentions_slot(lo, target) || Self::expr_mentions_slot(hi, target)
            }
            ExprKind::RangeCheck { value, lo, hi, .. } => {
                Self::expr_mentions_slot(value, target)
                    || lo.as_ref().is_some_and(|e| Self::expr_mentions_slot(e, target))
                    || hi.as_ref().is_some_and(|e| Self::expr_mentions_slot(e, target))
            }
            ExprKind::If(c, t, f) => {
                Self::expr_mentions_slot(c, target)
                    || Self::expr_mentions_slot(t, target)
                    || Self::expr_mentions_slot(f, target)
            }
            ExprKind::Index(b, idxs) => {
                Self::expr_mentions_slot(b, target)
                    || idxs.iter().any(|i| Self::expr_mentions_slot(i, target))
            }
            ExprKind::Tuple(es) | ExprKind::Variant(_, es) | ExprKind::Builtin(_, es) => {
                es.iter().any(|x| Self::expr_mentions_slot(x, target))
            }
            ExprKind::Call(_, args) => args.iter().any(|a| match a {
                Arg::Value(x) => Self::expr_mentions_slot(x, target),
                Arg::Mut(p) => Self::place_mentions_slot(p, target),
            }),
            ExprKind::CallExpr(f, args) => {
                Self::expr_mentions_slot(f, target)
                    || args.iter().any(|a| match a {
                        Arg::Value(x) => Self::expr_mentions_slot(x, target),
                        Arg::Mut(p) => Self::place_mentions_slot(p, target),
                    })
            }
            ExprKind::MutSlice(p, lo, hi) => {
                Self::place_mentions_slot(p, target)
                    || Self::expr_mentions_slot(lo, target)
                    || Self::expr_mentions_slot(hi, target)
            }
            ExprKind::Mutate(_, p, args) => {
                Self::place_mentions_slot(p, target)
                    || args.iter().any(|x| Self::expr_mentions_slot(x, target))
            }
            ExprKind::Sort(p, _, desc) => {
                Self::place_mentions_slot(p, target)
                    || desc.as_ref().is_some_and(|x| Self::expr_mentions_slot(x, target))
                    || true
            }
            ExprKind::Do(b) => b.stmts.iter().any(|s| Self::stmt_mentions_slot(s, target)),
            ExprKind::Fill(_, fill) => {
                fill.as_ref().is_some_and(|x| Self::expr_mentions_slot(x, target))
            }
            ExprKind::Print(args, end) => {
                args.iter().any(|a| Self::expr_mentions_slot(&a.value, target))
                    || end.as_ref().is_some_and(|x| Self::expr_mentions_slot(x, target))
            }
            ExprKind::Debug(_, args) => {
                args.iter().any(|a| Self::expr_mentions_slot(&a.value, target))
            }
            ExprKind::Format(pieces) => pieces.iter().any(|p| match p {
                cpl_ir::FmtPiece::Text(_) => false,
                cpl_ir::FmtPiece::Value(a) => Self::expr_mentions_slot(&a.value, target),
                cpl_ir::FmtPiece::Number(e, _) => Self::expr_mentions_slot(e, target),
            }),
            ExprKind::Read(_) | ExprKind::Parse(..) => true,
            ExprKind::Unit
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Bool(_)
            | ExprKind::Char(_)
            | ExprKind::Str(_)
            | ExprKind::None
            | ExprKind::Global(_)
            | ExprKind::DomainVal(_)
            | ExprKind::DomainLen(_)
            | ExprKind::DomainLo(_)
            | ExprKind::Error => false,
        }
    }

    fn stmts_have_closure(stmts: &[Stmt]) -> bool {
        stmts.iter().any(|s| Self::stmt_mentions_slot(s, usize::MAX))
    }

    fn expr_pure_eq(a: &Expr, b: &Expr) -> bool {
        match (&a.kind, &b.kind) {
            (ExprKind::Int(x), ExprKind::Int(y)) => x == y,
            (ExprKind::Bool(x), ExprKind::Bool(y)) => x == y,
            (ExprKind::Local(x), ExprKind::Local(y)) => x == y,
            (ExprKind::Global(x), ExprKind::Global(y)) => x == y,
            (ExprKind::Binary(op1, l1, r1), ExprKind::Binary(op2, l2, r2)) => {
                op1 == op2 && Self::expr_pure_eq(l1, l2) && Self::expr_pure_eq(r1, r2)
            }
            (ExprKind::Unary(op1, x1), ExprKind::Unary(op2, x2)) => {
                op1 == op2 && Self::expr_pure_eq(x1, x2)
            }
            (ExprKind::Field(e1, f1), ExprKind::Field(e2, f2)) => {
                f1 == f2 && Self::expr_pure_eq(e1, e2)
            }
            (ExprKind::Cast(c1, e1), ExprKind::Cast(c2, e2)) => {
                c1 == c2 && Self::expr_pure_eq(e1, e2)
            }
            _ => false,
        }
    }

    fn extract_arr_place_parts(place: &Place) -> Option<(&Place, &Expr, Option<&Expr>)> {
        let PlaceKind::Index(base, indices) = &place.kind else {
            return None;
        };
        if matches!(base.ty, Ty::Array { .. }) {
            if indices.len() == 2 {
                return Some((base, &indices[0], Some(&indices[1])));
            }
            if indices.len() == 1 {
                if let PlaceKind::Index(outer, outer_idxs) = &base.kind {
                    if matches!(outer.ty, Ty::Array { .. }) && outer_idxs.len() == 1 {
                        return Some((outer, &outer_idxs[0], Some(&indices[0])));
                    }
                }
                return Some((base, &indices[0], None));
            }
        }
        None
    }

    fn extract_arr_expr_parts(expr: &Expr) -> Option<(&Expr, &Expr, Option<&Expr>)> {
        let ExprKind::Index(base, indices) = &expr.kind else {
            return None;
        };
        if matches!(base.ty, Ty::Array { .. }) {
            if indices.len() == 2 {
                return Some((base, &indices[0], Some(&indices[1])));
            }
            if indices.len() == 1 {
                if let ExprKind::Index(outer, outer_idxs) = &base.kind {
                    if matches!(outer.ty, Ty::Array { .. }) && outer_idxs.len() == 1 {
                        return Some((outer, &outer_idxs[0], Some(&indices[0])));
                    }
                }
                return Some((base, &indices[0], None));
            }
        }
        None
    }

    fn expr_matches_arr_place(expr: &Expr, place: &Place) -> bool {
        let (Some((eb, ei0, ei1)), Some((pb, pi0, pi1))) =
            (Self::extract_arr_expr_parts(expr), Self::extract_arr_place_parts(place))
        else {
            return false;
        };
        let base_ok = match (&eb.kind, &pb.kind) {
            (ExprKind::Global(g1), PlaceKind::Global(g2)) => g1 == g2,
            (ExprKind::Local(l1), PlaceKind::Local(l2)) => l1 == l2,
            _ => false,
        };
        if !base_ok || !Self::expr_pure_eq(ei0, pi0) {
            return false;
        }
        match (ei1, pi1) {
            (Some(a), Some(b)) => Self::expr_pure_eq(a, b),
            (None, None) => true,
            _ => false,
        }
    }

    fn try_compile_opt_minmax_pair(
        &mut self,
        s0: &'p Stmt,
        s1: &'p Stmt,
        rest: &'p [Stmt],
    ) -> bool {
        let Stmt::Let(r_slot, Some(rhs)) = s0 else {
            return false;
        };
        let Stmt::If(outer_cond, outer_then, outer_else) = s1 else {
            return false;
        };
        if !outer_else.stmts.is_empty() || outer_then.stmts.len() != 1 {
            return false;
        }
        let ExprKind::IsSome(c_inner) = &outer_cond.kind else {
            return false;
        };
        if !matches!(c_inner.kind, ExprKind::Local(s) if s == *r_slot) {
            return false;
        }
        let Stmt::If(inner_cond, inner_then, inner_else) = &outer_then.stmts[0] else {
            return false;
        };
        if !inner_else.stmts.is_empty() || inner_then.stmts.len() != 1 {
            return false;
        }
        let Stmt::Assign(place, AssignOp::Set, set_val) = &inner_then.stmts[0] else {
            return false;
        };
        let ExprKind::Some(set_inner) = &set_val.kind else {
            return false;
        };
        if !matches!(&set_inner.kind, ExprKind::Unwrap(u, false) if matches!(u.kind, ExprKind::Local(s) if s == *r_slot))
        {
            return false;
        }
        let ExprKind::Binary(BinOp::Or, p_none, better) = &inner_cond.kind else {
            return false;
        };
        let ExprKind::Unary(UnOp::Not, p_some) = &p_none.kind else {
            return false;
        };
        let ExprKind::IsSome(p_expr1) = &p_some.kind else {
            return false;
        };
        let ExprKind::Cmp(cmp_op @ (CmpOp::Lt | CmpOp::Gt), cmp_lhs, cmp_rhs) = &better.kind else {
            return false;
        };
        if !matches!(&cmp_lhs.kind, ExprKind::Unwrap(u, false) if matches!(u.kind, ExprKind::Local(s) if s == *r_slot))
        {
            return false;
        }
        let ExprKind::Unwrap(p_expr2, false) = &cmp_rhs.kind else {
            return false;
        };
        if !Self::expr_matches_arr_place(p_expr1, place)
            || !Self::expr_matches_arr_place(p_expr2, place)
        {
            return false;
        }
        if ScalarKind::of_opt_ty(&place.ty, self.program) != Some(ScalarKind::Int) {
            return false;
        }
        let Some((base, idx0, idx1)) = Self::extract_arr_place_parts(place) else {
            return false;
        };
        if rest.iter().any(|s| Self::stmt_mentions_slot(s, *r_slot)) {
            return false;
        }
        let (root, is_global) = match &base.kind {
            PlaceKind::Global(gid) => (*gid as u16, true),
            PlaceKind::Local(slot) => match self.slot_loc(*slot) {
                SlotLoc::V(v) => (v, false),
                _ => return false,
            },
            _ => return false,
        };
        let is_2d = idx1.is_some();
        let is_max = *cmp_op == CmpOp::Gt;
        let (r_val, r_some) = self.compile_expr_opt_i(rhs);
        let j_skip = self.emit(Op::JmpIfZero { reg: r_some, target: 0 }, place.span);
        let r_i = self.compile_expr_i(idx0);
        let r_j = match idx1 {
            Some(e) => self.compile_expr_i(e),
            None => 0,
        };
        self.emit(
            Op::MinMaxArrOptI { root, is_global, is_2d, is_max, r_i, r_j, r_val },
            place.span,
        );
        let end_pc = self.ops.len() as u32;
        self.patch(j_skip, end_pc);
        true
    }

    fn compile_stmts(&mut self, stmts: &'p [Stmt]) {
        if let Some((slot, span)) = self.traced_loop_var.take() {
            self.emit_trace(TracePoint::Loop(slot, span), span);
        }
        let has_closure = Self::stmts_have_closure(stmts);
        let mut declared = Vec::new();
        let mut i = 0;
        while i < stmts.len() {
            if self.traced {
                self.compile_stmt(&stmts[i]);
                i += 1;
                continue;
            }
            if i + 1 < stmts.len()
                && self.try_compile_opt_minmax_pair(&stmts[i], &stmts[i + 1], &stmts[i + 2..])
            {
                i += 2;
                continue;
            }
            let stmt = &stmts[i];
            if let Stmt::Let(slot, _) = stmt {
                declared.push(*slot);
            }
            if !has_closure {
                if let Stmt::Assign(place, AssignOp::Set, rhs) = stmt {
                    if let ExprKind::Local(src_slot) = rhs.kind {
                        if declared.contains(&src_slot)
                            && !stmts[i + 1..].iter().any(|s| Self::stmt_mentions_slot(s, src_slot))
                        {
                            if let SlotLoc::V(src_reg) = self.slot_loc(src_slot) {
                                match &place.kind {
                                    PlaceKind::Global(gid) => {
                                        self.emit(
                                            Op::StoreGlobalVMove {
                                                gid: *gid as u16,
                                                src: src_reg,
                                            },
                                            rhs.span,
                                        );
                                        i += 1;
                                        continue;
                                    }
                                    PlaceKind::Local(dst_slot) => {
                                        if let SlotLoc::V(dst_reg) = self.slot_loc(*dst_slot) {
                                            if dst_reg != src_reg {
                                                self.emit(
                                                    Op::MoveVConsume {
                                                        dst: dst_reg,
                                                        src: src_reg,
                                                    },
                                                    rhs.span,
                                                );
                                            }
                                            i += 1;
                                            continue;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            self.compile_stmt(stmt);
            i += 1;
        }
    }

    fn compile_block(&mut self, block: &'p Block, ret_kind: Option<ScalarKind>, emit_ret: bool) {
        self.compile_stmts(&block.stmts);
        if emit_ret {
            if ret_kind.is_some() {
                let r = self.alloc_i();
                self.emit(Op::ConstImmI { dst: r, val: 0 }, Span::default());
                self.emit(Op::RetI { src: r }, Span::default());
            } else {
                self.emit(Op::RetUnit, Span::default());
            }
        }
    }

    fn compile_stmt(&mut self, stmt: &'p Stmt) {
        if !self.traced || !self.compile_traced_stmt(stmt) {
            self.compile_untraced_stmt(stmt);
        }
    }

    fn emit_trace(&mut self, point: TracePoint<'p>, span: Span) {
        let idx = self.trace_points.len() as u32;
        self.trace_points.push(point);
        self.emit(Op::Trace { idx }, span);
    }

    /// In a traced run the VM runs control flow and expressions that
    /// neither write nor call; a write it does itself is followed by a
    /// `Trace`, and anything else is left to the tree-walker, which
    /// reports it. Returns whether `stmt` was compiled.
    fn compile_traced_stmt(&mut self, stmt: &'p Stmt) -> bool {
        let fallbacks = self.fallback_stmts.len();
        match stmt {
            Stmt::Let(slot, Some(init)) if self.feed_acc != Some(*slot) && quiet(init) => {
                self.compile_untraced_stmt(stmt);
                if self.fallback_stmts.len() == fallbacks {
                    self.emit_trace(TracePoint::Let(*slot, init), init.span);
                }
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None) => return false,
            Stmt::Assign(place, _, value)
                if quiet_place(place, cpl_ir::walk::root(place)) && quiet(value) =>
            {
                self.compile_untraced_stmt(stmt);
                if self.fallback_stmts.len() == fallbacks {
                    self.emit_trace(TracePoint::Assign(place), place.span);
                }
            }
            Stmt::If(cond, ..) | Stmt::While(_, cond, _) | Stmt::Assert { cond, .. } if quiet(cond) => return false,
            Stmt::Return(Some(value)) | Stmt::Give(value) if quiet(value) => return false,
            Stmt::For(_, slot, iter, _) if self.feed_acc.is_none() => match quiet_iter(iter) {
                Some(span) => {
                    self.traced_loop_var = Some((*slot, span));
                    self.compile_untraced_stmt(stmt);
                    self.traced_loop_var = None;
                    self.emit_trace(TracePoint::LoopEnd(*slot), span);
                }
                None => {
                    self.emit_fallback_stmt(stmt);
                    self.emit_escapes();
                }
            },
            _ => {
                self.emit_fallback_stmt(stmt);
                self.emit_escapes();
            }
        }
        true
    }

    /// After a fallback statement inside loops: where its `break` or
    /// `continue`, run by the tree-walker, goes.
    fn emit_escapes(&mut self) {
        let last = self.loops.len().wrapping_sub(1);
        for pos in (0..self.loops.len()).rev() {
            let (id, innermost) = (self.loops[pos].id, pos == last);
            let pc = self.emit(Op::JmpEscape { brk: true, id, innermost, target: 0 }, Span::default());
            self.loops[pos].breaks.push(pc);
            let pc = self.emit(Op::JmpEscape { brk: false, id, innermost, target: 0 }, Span::default());
            self.loops[pos].continues.push(pc);
        }
    }

    fn compile_untraced_stmt(&mut self, stmt: &'p Stmt) {
        match stmt {
            Stmt::Let(slot, init) => {
                if self.feed_acc == Some(*slot) {
                    return;
                }
                match self.slot_loc(*slot) {
                SlotLoc::I(dst, _) => {
                    if let Some(expr) = init {
                        let r = self.compile_expr_i(expr);
                        if r != dst {
                            self.emit(Op::MoveI { dst, src: r }, expr.span);
                        }
                    } else {
                        self.emit(Op::ConstImmI { dst, val: 0 }, Span::default());
                    }
                }
                SlotLoc::OptI { val, some, .. } => {
                    if let Some(expr) = init {
                        let (rv, rs) = self.compile_expr_opt_i(expr);
                        if rv != val {
                            self.emit(Op::MoveI { dst: val, src: rv }, expr.span);
                        }
                        if rs != some {
                            self.emit(Op::MoveI { dst: some, src: rs }, expr.span);
                        }
                    } else {
                        self.emit(Op::ConstImmI { dst: some, val: 0 }, Span::default());
                    }
                }
                SlotLoc::SmallTuple { base, len, .. } => {
                    if let Some(expr) = init {
                        self.compile_expr_small_tuple(expr, base, len);
                    } else {
                        for k in 0..len as u16 {
                            self.emit(Op::ConstImmI { dst: base + k, val: 0 }, Span::default());
                        }
                    }
                }
                SlotLoc::V(dst) => {
                    if let Some(expr) = init {
                        let r = self.compile_expr_v(expr);
                        if r != dst {
                            self.emit(Op::MoveV { dst, src: r }, expr.span);
                        }
                    } else {
                        self.emit_fallback_stmt(stmt);
                    }
                }
            }
            },
            Stmt::Expr(expr) => {
                if let (Some(acc), ExprKind::Mutate(op, place, args)) = (self.feed_acc, &expr.kind)
                {
                    if matches!(place.kind, PlaceKind::Local(s) if s == acc) {
                        match (*op, args.as_slice()) {
                            (Mutation::Push, [arg]) => {
                                if let Some(kind) = ScalarKind::of_ty(&arg.ty, self.program) {
                                    let src = self.compile_expr_i(arg);
                                    self.emit(Op::FeedYieldI { src, kind }, expr.span);
                                } else {
                                    let src = self.compile_expr_v(arg);
                                    self.emit(Op::FeedYieldV { src }, expr.span);
                                    if src >= self.num_local_v {
                                        self.emit(Op::ClearV { reg: src }, expr.span);
                                    }
                                }
                                return;
                            }
                            (Mutation::Extend, [arg]) => {
                                let src = self.compile_expr_v(arg);
                                self.emit(Op::FeedYieldExtend { src }, expr.span);
                                if src >= self.num_local_v {
                                    self.emit(Op::ClearV { reg: src }, expr.span);
                                }
                                return;
                            }
                            _ => {}
                        }
                    }
                }
                if ScalarKind::of_ty(&expr.ty, self.program).is_some() {
                    let _ = self.compile_expr_i(expr);
                } else if let ExprKind::Print(args, end) = &expr.kind {
                    self.compile_print(args, end.as_deref(), expr.span);
                } else if let ExprKind::Mutate(op, place, args) = &expr.kind {
                    if !self.try_compile_mutate_stmt(*op, place, args, expr.span) {
                        self.emit_fallback_stmt(stmt);
                    }
                } else if let ExprKind::Sort(place, cpl_ir::SortBy::Natural, desc) = &expr.kind {
                    if !self.try_compile_sort_stmt(place, desc.as_deref(), expr.span) {
                        self.emit_fallback_stmt(stmt);
                    }
                } else if let ExprKind::Call(fn_id, args) = &expr.kind {
                    if self
                        .try_compile_call_mix(*fn_id, args, CallRet::Unit, expr.span)
                        .is_none()
                    {
                        self.emit_fallback_stmt(stmt);
                    }
                } else if let ExprKind::CallExpr(callee, args) = &expr.kind {
                    if self
                        .try_compile_call_closure(callee, args, CallRet::Unit, expr.span)
                        .is_none()
                    {
                        self.emit_fallback_stmt(stmt);
                    }
                } else if let ExprKind::Do(block) = &expr.kind {
                    let prev_pinned = self.pinned_v;
                    self.pinned_v = self.next_v;
                    self.give_targets.push(GiveTarget::Discard { exits: Vec::new() });
                    self.compile_stmts(&block.stmts);
                    let mut target = self.give_targets.pop().unwrap();
                    self.pinned_v = prev_pinned;
                    let end_pc = self.ops.len() as u32;
                    for j in target.exits_mut().drain(..) {
                        self.patch(j, end_pc);
                    }
                } else {
                    self.emit_fallback_stmt(stmt);
                }
            }
            Stmt::Assert { cond, span, .. } => {
                let r_cond = self.compile_expr_i(cond);
                let j_ok = self.emit(Op::JmpIfNonZero { reg: r_cond, target: 0 }, *span);
                self.emit_fallback_stmt(stmt);
                let end_pc = self.ops.len() as u32;
                self.patch(j_ok, end_pc);
            }
            Stmt::Assign(place, op, value) => {
                if !self.try_compile_assign(place, *op, value) {
                    self.emit_fallback_stmt(stmt);
                }
            }
            Stmt::If(cond, then_b, else_b) => {
                if let Some(b) = Self::const_bool(cond) {
                    let branch = if b { then_b } else { else_b };
                    for s in &branch.stmts {
                        self.compile_stmt(s);
                    }
                    return;
                }
                let j_else = self.emit_branch_if_false(cond);
                for s in &then_b.stmts {
                    self.compile_stmt(s);
                }
                if else_b.stmts.is_empty() {
                    let end = self.ops.len() as u32;
                    self.patch(j_else, end);
                } else {
                    let j_end = self.emit(Op::Jmp { target: 0 }, cond.span);
                    let else_pc = self.ops.len() as u32;
                    self.patch(j_else, else_pc);
                    for s in &else_b.stmts {
                        self.compile_stmt(s);
                    }
                    let end = self.ops.len() as u32;
                    self.patch(j_end, end);
                }
            }
            Stmt::While(id, cond, body) => {
                let head = self.ops.len() as u32;
                let j_exit = self.emit_branch_if_false(cond);
                self.loops.push(LoopTargets {
                    id: *id,
                    breaks: Vec::new(),
                    continues: Vec::new(),
                });
                self.compile_stmts(&body.stmts);
                let lt = self.loops.pop().unwrap();
                for c in lt.continues {
                    self.patch(c, head);
                }
                self.emit(Op::Jmp { target: head }, cond.span);
                let end = self.ops.len() as u32;
                self.patch(j_exit, end);
                for b in lt.breaks {
                    self.patch(b, end);
                }
            }
            Stmt::For(id, slot, iter, body) => {
                if !self.try_compile_for(*id, *slot, iter, body) {
                    self.emit_fallback_stmt(stmt);
                }
            }
            Stmt::Break(id) => {
                let target_pos = match id {
                    Some(id) => self.loops.iter().rposition(|l| l.id == *id),
                    None => self.loops.len().checked_sub(1),
                };
                if let Some(pos) = target_pos {
                    let pc = self.emit(Op::Jmp { target: 0 }, Span::default());
                    self.loops[pos].breaks.push(pc);
                } else {
                    self.emit_fallback_stmt(stmt);
                }
            }
            Stmt::Continue(id) => {
                let target_pos = match id {
                    Some(id) => self.loops.iter().rposition(|l| l.id == *id),
                    None => self.loops.len().checked_sub(1),
                };
                if let Some(pos) = target_pos {
                    let pc = self.emit(Op::Jmp { target: 0 }, Span::default());
                    self.loops[pos].continues.push(pc);
                } else {
                    self.emit_fallback_stmt(stmt);
                }
            }
            Stmt::Give(e) => {
                if let Some(target) = self.give_targets.last() {
                    match *target {
                        GiveTarget::I { dst, .. } => {
                            let r = self.compile_expr_i(e);
                            if r != dst {
                                self.emit(Op::MoveI { dst, src: r }, e.span);
                            }
                        }
                        GiveTarget::OptI { dst_val, dst_some, .. } => {
                            let (rv, rs) = self.compile_expr_opt_i(e);
                            if rv != dst_val {
                                self.emit(Op::MoveI { dst: dst_val, src: rv }, e.span);
                            }
                            if rs != dst_some {
                                self.emit(Op::MoveI { dst: dst_some, src: rs }, e.span);
                            }
                        }
                        GiveTarget::V { dst, .. } => {
                            let r = self.compile_expr_v(e);
                            if r != dst {
                                self.emit(Op::MoveV { dst, src: r }, e.span);
                            }
                        }
                        GiveTarget::Discard { .. } => {
                            if !matches!(
                                e.kind,
                                ExprKind::Unit
                                    | ExprKind::Int(_)
                                    | ExprKind::Bool(_)
                                    | ExprKind::Char(_)
                                    | ExprKind::None
                                    | ExprKind::Local(_)
                                    | ExprKind::Global(_)
                            ) {
                                let _ = self.compile_expr_v(e);
                            }
                        }
                    }
                    let pc = self.emit(Op::Jmp { target: 0 }, Span::default());
                    if let Some(t) = self.give_targets.last_mut() {
                        t.exits_mut().push(pc);
                    }
                } else if let Some((ret_dst, _)) = self.inline_ret {
                    let r = self.compile_expr_i(e);
                    if r != ret_dst {
                        self.emit(Op::MoveI { dst: ret_dst, src: r }, e.span);
                    }
                    let pc = self.emit(Op::Jmp { target: 0 }, Span::default());
                    if let Some((_, jumps)) = &mut self.inline_ret {
                        jumps.push(pc);
                    }
                } else if self.cur_fn.is_some() {
                    if ScalarKind::of_ty(&e.ty, self.program).is_some() {
                        let r = self.compile_expr_i(e);
                        self.emit(Op::RetI { src: r }, e.span);
                    } else {
                        let r = self.compile_expr_v(e);
                        self.emit(Op::RetV { src: r }, e.span);
                    }
                } else {
                    self.emit_fallback_stmt(stmt);
                }
            }
            Stmt::Return(val) => {
                if self.feed_acc.is_some() {
                    self.emit(Op::FeedEnd, Span::default());
                } else if let Some((ret_dst, _)) = self.inline_ret {
                    if let Some(e) = val {
                        let r = self.compile_expr_i(e);
                        if r != ret_dst {
                            self.emit(Op::MoveI { dst: ret_dst, src: r }, e.span);
                        }
                    }
                    let pc = self.emit(Op::Jmp { target: 0 }, Span::default());
                    if let Some((_, jumps)) = &mut self.inline_ret {
                        jumps.push(pc);
                    }
                } else if let Some(e) = val {
                    if ScalarKind::of_ty(&e.ty, self.program).is_some() {
                        let r = self.compile_expr_i(e);
                        self.emit(Op::RetI { src: r }, e.span);
                    } else {
                        let r = self.compile_expr_v(e);
                        self.emit(Op::RetV { src: r }, e.span);
                    }
                } else {
                    self.emit(Op::RetUnit, Span::default());
                }
            }
            _ => self.emit_fallback_stmt(stmt),
        }
    }

    fn block_has_outer_loop_jump(block: &Block, fed_id: LoopId) -> bool {
        fn check_block(b: &Block, fed_id: LoopId, inner: &mut Vec<LoopId>) -> bool {
            for s in &b.stmts {
                match s {
                    Stmt::Break(Some(id)) | Stmt::Continue(Some(id)) => {
                        if *id != fed_id && !inner.contains(id) {
                            return true;
                        }
                    }
                    Stmt::If(_, t, e) => {
                        if check_block(t, fed_id, inner) || check_block(e, fed_id, inner) {
                            return true;
                        }
                    }
                    Stmt::While(lid, _, body) | Stmt::For(lid, _, _, body) => {
                        inner.push(*lid);
                        let bad = check_block(body, fed_id, inner);
                        inner.pop();
                        if bad {
                            return true;
                        }
                    }
                    Stmt::Expr(e) if matches!(e.kind, ExprKind::Do(_)) => {
                        if let ExprKind::Do(b2) = &e.kind {
                            if check_block(b2, fed_id, inner) {
                                return true;
                            }
                        }
                    }
                    Stmt::Give(_) => return true,
                    _ => {}
                }
            }
            false
        }
        check_block(block, fed_id, &mut Vec::new())
    }

    fn try_compile_for_fed(
        &mut self,
        id: LoopId,
        slot: Slot,
        fn_id: FnId,
        seq_expr: &'p Expr,
        body: &'p Block,
    ) -> bool {
        let ExprKind::Call(cid, args) = &seq_expr.kind else {
            return false;
        };
        let f = &self.program.fns[fn_id];
        if *cid != fn_id
            || !self.fed_ok[fn_id]
            || matches!(&f.ret, Ty::List(elem) if matches!(**elem, Ty::Struct(_)))
            || f.locals.len() > 512
            || self.program.judge_fns.contains(&fn_id)
            || args.len() != f.params.len()
            || f.params.iter().any(|p| p.mutable)
            || !self.give_targets.is_empty()
            || self.inline_ret.is_some()
            || Self::block_has_outer_loop_jump(body, id)
        {
            return false;
        }
        if !args.iter().all(|a| matches!(a, Arg::Value(_))) {
            return false;
        }
        let mut args_i = Vec::new();
        let mut args_v = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            let Arg::Value(e) = arg else { unreachable!() };
            let pty = &f.locals[i].ty;
            if ScalarKind::of_ty(pty, self.program).is_some() {
                args_i.push(self.compile_expr_i(e));
            } else if ScalarKind::of_opt_ty(pty, self.program).is_some() {
                let (rv, rs) = self.compile_expr_opt_i(e);
                args_i.push(rv);
                args_i.push(rs);
            } else {
                args_v.push(self.compile_expr_v(e));
            }
        }
        let var_loc = self.slot_loc(slot);
        let spec_idx = self.feed_calls.len() as u16;
        self.feed_calls.push(FeedCallSpec {
            fn_id,
            args_i,
            args_v,
            body_pc: 0,
            var_loc,
        });
        self.emit(Op::CallVmFeed { spec: spec_idx }, seq_expr.span);
        let j_after = self.emit(Op::Jmp { target: 0 }, seq_expr.span);

        let body_pc = self.ops.len();
        self.feed_calls[spec_idx as usize].body_pc = body_pc;
        self.loops.push(LoopTargets {
            id,
            breaks: Vec::new(),
            continues: Vec::new(),
        });
        self.compile_stmts(&body.stmts);
        let lt = self.loops.pop().unwrap();
        let next_pc = self.ops.len() as u32;
        for c in lt.continues {
            self.patch(c, next_pc);
        }
        self.emit(Op::FeedBodyNext, Span::default());
        let break_pc = self.ops.len() as u32;
        for b in lt.breaks {
            self.patch(b, break_pc);
        }
        self.emit(Op::FeedBodyBreak, Span::default());
        let after_pc = self.ops.len() as u32;
        self.patch(j_after, after_pc);
        true
    }

    fn try_compile_for(
        &mut self,
        id: LoopId,
        slot: Slot,
        iter: &'p Iter,
        body: &'p Block,
    ) -> bool {
        if let Iter::Seq(seq_expr, rev) = iter {
            if !*rev && self.allow_fed_loops {
                if let Some((fn_id, _acc_slot)) = self.feeds.feeds(id, seq_expr, body) {
                    return self.try_compile_for_fed(id, slot, fn_id, seq_expr, body);
                }
            }
            if !*rev && self.feeds.feeds(id, seq_expr, body).is_some() {
                return false;
            }
            return self.try_compile_for_seq(id, slot, seq_expr, *rev, body);
        }
        let SlotLoc::I(r_var, _) = self.slot_loc(slot) else {
            return false;
        };
        match iter {
            Iter::Range {
                lo,
                hi,
                step: None,
                rev: false,
                closed,
            } => {
                let r_lo = self.compile_expr_i(lo);
                let r_hi = self.compile_expr_i(hi);
                let j_skip = if *closed {
                    self.emit(Op::JmpGtI { a: r_lo, b: r_hi, target: 0 }, lo.span)
                } else {
                    self.emit(Op::JmpGeI { a: r_lo, b: r_hi, target: 0 }, lo.span)
                };
                if r_var != r_lo {
                    self.emit(Op::MoveI { dst: r_var, src: r_lo }, lo.span);
                }
                let body_pc = self.ops.len() as u32;
                self.loops.push(LoopTargets {
                    id,
                    breaks: Vec::new(),
                    continues: Vec::new(),
                });
                self.compile_stmts(&body.stmts);
                let lt = self.loops.pop().unwrap();
                let step_pc = self.ops.len() as u32;
                for c in lt.continues {
                    self.patch(c, step_pc);
                }
                if *closed {
                    self.emit(
                        Op::ForIncClosed { r_i: r_var, r_hi, target: body_pc },
                        hi.span,
                    );
                } else {
                    self.emit(
                        Op::ForInc { r_i: r_var, r_hi, target: body_pc },
                        hi.span,
                    );
                }
                let end_pc = self.ops.len() as u32;
                self.patch(j_skip, end_pc);
                for b in lt.breaks {
                    self.patch(b, end_pc);
                }
                true
            }
            Iter::Range {
                lo,
                hi,
                step: None,
                rev: true,
                closed,
            } => {
                let r_lo = self.compile_expr_i(lo);
                let r_hi = self.compile_expr_i(hi);
                let j_skip = if *closed {
                    self.emit(Op::JmpGtI { a: r_lo, b: r_hi, target: 0 }, lo.span)
                } else {
                    self.emit(Op::JmpGeI { a: r_lo, b: r_hi, target: 0 }, lo.span)
                };
                if *closed {
                    if r_var != r_hi {
                        self.emit(Op::MoveI { dst: r_var, src: r_hi }, hi.span);
                    }
                } else {
                    self.emit(Op::AddImmI { dst: r_var, a: r_hi, imm: -1 }, hi.span);
                }
                let body_pc = self.ops.len() as u32;
                self.loops.push(LoopTargets {
                    id,
                    breaks: Vec::new(),
                    continues: Vec::new(),
                });
                self.compile_stmts(&body.stmts);
                let lt = self.loops.pop().unwrap();
                let step_pc = self.ops.len() as u32;
                for c in lt.continues {
                    self.patch(c, step_pc);
                }
                self.emit(Op::ForDec { r_i: r_var, r_lo, target: body_pc }, lo.span);
                let end_pc = self.ops.len() as u32;
                self.patch(j_skip, end_pc);
                for b in lt.breaks {
                    self.patch(b, end_pc);
                }
                true
            }
            _ => false,
        }
    }

    fn try_compile_for_seq(
        &mut self,
        id: LoopId,
        slot: Slot,
        seq_expr: &'p Expr,
        rev: bool,
        body: &'p Block,
    ) -> bool {
        #[derive(Clone, Copy)]
        enum SeqKind {
            Array,
            List,
            Str,
        }
        let seq_kind = match &seq_expr.ty {
            Ty::Array { dims, .. } if dims.len() == 1 => SeqKind::Array,
            Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) => SeqKind::List,
            Ty::Str | Ty::StrView => SeqKind::Str,
            _ => return false,
        };
        let slot_loc = self.slot_loc(slot);
        match (seq_kind, slot_loc) {
            (SeqKind::Str, SlotLoc::I(..)) => {}
            (SeqKind::Str, _) => return false,
            (_, SlotLoc::I(..) | SlotLoc::V(_) | SlotLoc::SmallTuple { .. }) => {}
            (_, SlotLoc::OptI { .. }) => return false,
        }
        let r_idx = self.alloc_i();
        let r_len = self.alloc_i();
        let global_id = match &seq_expr.kind {
            ExprKind::Global(gid) if matches!(seq_expr.ty, Ty::Array { .. } | Ty::List(_)) => {
                Some(*gid as u16)
            }
            _ => None,
        };
        let local_v = if global_id.is_none() {
            Some(self.compile_expr_v(seq_expr))
        } else {
            None
        };
        match (seq_kind, global_id) {
            (SeqKind::Array, Some(gid)) => self.emit(Op::GArrLen { dst: r_len, gid }, seq_expr.span),
            (SeqKind::Array, None) => self.emit(
                Op::LArrLen { dst: r_len, arr_v: local_v.unwrap() },
                seq_expr.span,
            ),
            (SeqKind::List, Some(gid)) => self.emit(Op::GListLen { dst: r_len, gid }, seq_expr.span),
            (SeqKind::List, None) => self.emit(
                Op::LListLen { dst: r_len, list_v: local_v.unwrap() },
                seq_expr.span,
            ),
            (SeqKind::Str, _) => self.emit(
                Op::LStrLen { dst: r_len, str_v: local_v.unwrap() },
                seq_expr.span,
            ),
        };
        let r_zero = self.alloc_i();
        let j_skip = if rev {
            let r_one = self.alloc_i();
            self.emit(Op::ConstImmI { dst: r_zero, val: 0 }, seq_expr.span);
            self.emit(Op::ConstImmI { dst: r_one, val: 1 }, seq_expr.span);
            self.emit(Op::SubI { dst: r_idx, a: r_len, b: r_one }, seq_expr.span);
            self.emit(Op::JmpLtI { a: r_idx, b: r_zero, target: 0 }, seq_expr.span)
        } else {
            self.emit(Op::ConstImmI { dst: r_idx, val: 0 }, seq_expr.span);
            self.emit(Op::JmpGeI { a: r_idx, b: r_len, target: 0 }, seq_expr.span)
        };
        let body_pc = self.ops.len() as u32;
        match slot_loc {
            SlotLoc::I(r_var, _) => match (seq_kind, global_id) {
                (SeqKind::Array, Some(gid)) => {
                    self.emit(Op::LoadGArr1I { dst: r_var, gid, r_i: r_idx }, seq_expr.span);
                }
                (SeqKind::Array, None) => {
                    self.emit(
                        Op::LoadLArr1I { dst: r_var, arr_v: local_v.unwrap(), r_i: r_idx },
                        seq_expr.span,
                    );
                }
                (SeqKind::List, Some(gid)) => {
                    self.emit(Op::LoadGList1I { dst: r_var, gid, r_i: r_idx }, seq_expr.span);
                }
                (SeqKind::List, None) => {
                    self.emit(
                        Op::LoadLList1I { dst: r_var, list_v: local_v.unwrap(), r_i: r_idx },
                        seq_expr.span,
                    );
                }
                (SeqKind::Str, Some(gid)) => {
                    self.emit(Op::GStrGetI { dst: r_var, gid, r_i: r_idx }, seq_expr.span);
                }
                (SeqKind::Str, None) => {
                    self.emit(
                        Op::StrGetI { dst: r_var, str_v: local_v.unwrap(), r_i: r_idx },
                        seq_expr.span,
                    );
                }
            },
            SlotLoc::V(r_var) => match (seq_kind, global_id) {
                (SeqKind::Array, Some(gid)) => {
                    self.emit(Op::LoadGArr1V { dst: r_var, gid, r_i: r_idx }, seq_expr.span);
                }
                (SeqKind::Array, None) => {
                    self.emit(
                        Op::LoadLArr1V { dst: r_var, arr_v: local_v.unwrap(), r_i: r_idx },
                        seq_expr.span,
                    );
                }
                (SeqKind::List, Some(gid)) => {
                    self.emit(Op::LoadGList1V { dst: r_var, gid, r_i: r_idx }, seq_expr.span);
                }
                (SeqKind::List, None) => {
                    self.emit(
                        Op::LoadLList1V { dst: r_var, list_v: local_v.unwrap(), r_i: r_idx },
                        seq_expr.span,
                    );
                }
                (SeqKind::Str, _) => unreachable!(),
            },
            SlotLoc::SmallTuple { base, len, .. } => match (seq_kind, global_id) {
                (SeqKind::Array, Some(gid)) => {
                    self.emit(
                        Op::LoadGArr1SmallTuple { dst_base: base, len, gid, r_i: r_idx },
                        seq_expr.span,
                    );
                }
                (SeqKind::Array, None) => {
                    self.emit(
                        Op::LoadLArr1SmallTuple {
                            dst_base: base,
                            len,
                            arr_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::List, Some(gid)) => {
                    self.emit(
                        Op::LoadList1SmallTuple {
                            dst_base: base,
                            len,
                            root: gid,
                            is_global: true,
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::List, None) => {
                    self.emit(
                        Op::LoadList1SmallTuple {
                            dst_base: base,
                            len,
                            root: local_v.unwrap(),
                            is_global: false,
                            r_i: r_idx,
                        },
                        seq_expr.span,
                    );
                }
                (SeqKind::Str, _) => unreachable!(),
            },
            SlotLoc::OptI { .. } => unreachable!(),
        }
        self.loops.push(LoopTargets {
            id,
            breaks: Vec::new(),
            continues: Vec::new(),
        });
        let saved_pinned = self.pinned_v;
        if let Some(lv) = local_v {
            self.pinned_v = self.pinned_v.max(lv + 1);
        }
        self.compile_stmts(&body.stmts);
        self.pinned_v = saved_pinned;
        let lt = self.loops.pop().unwrap();
        let step_pc = self.ops.len() as u32;
        for c in lt.continues {
            self.patch(c, step_pc);
        }
        if rev {
            self.emit(
                Op::ForDec { r_i: r_idx, r_lo: r_zero, target: body_pc },
                seq_expr.span,
            );
        } else {
            self.emit(
                Op::ForInc { r_i: r_idx, r_hi: r_len, target: body_pc },
                seq_expr.span,
            );
        }
        let end_pc = self.ops.len() as u32;
        self.patch(j_skip, end_pc);
        for b in lt.breaks {
            self.patch(b, end_pc);
        }
        if let Some(lv) = local_v {
            if lv >= self.num_local_v {
                self.emit(Op::ClearV { reg: lv }, seq_expr.span);
            }
        }
        true
    }

    fn try_compile_assign(&mut self, place: &'p Place, op: AssignOp, value: &'p Expr) -> bool {
        if self.try_compile_assign_fast(place, op, value) {
            return true;
        }
        self.try_compile_assign_place_path(place, op, value)
    }

    fn try_compile_assign_fast(&mut self, place: &'p Place, op: AssignOp, value: &'p Expr) -> bool {
        let val_kind = ScalarKind::of_ty(&place.ty, self.program);
        let opt_kind = ScalarKind::of_opt_ty(&place.ty, self.program);
        match (&place.kind, op, val_kind, opt_kind) {
            (PlaceKind::Local(slot), AssignOp::Set, Some(_), _) => {
                let SlotLoc::I(dst, _) = self.slot_loc(*slot) else {
                    return false;
                };
                let r = self.compile_expr_i(value);
                if r != dst {
                    self.emit(Op::MoveI { dst, src: r }, place.span);
                }
                true
            }
            (PlaceKind::Local(slot), AssignOp::Set, None, Some(_)) => {
                let SlotLoc::OptI { val, some, .. } = self.slot_loc(*slot) else {
                    return false;
                };
                let (rv, rs) = self.compile_expr_opt_i(value);
                if rv != val {
                    self.emit(Op::MoveI { dst: val, src: rv }, place.span);
                }
                if rs != some {
                    self.emit(Op::MoveI { dst: some, src: rs }, place.span);
                }
                true
            }
            (PlaceKind::Local(slot), AssignOp::Set, None, None) => match self.slot_loc(*slot) {
                SlotLoc::SmallTuple { base, len, .. } => {
                    self.compile_expr_small_tuple(value, base, len);
                    true
                }
                SlotLoc::V(dst) => {
                    let r = self.compile_expr_v(value);
                    if r != dst {
                        self.emit(Op::MoveV { dst, src: r }, place.span);
                    }
                    true
                }
                _ => false,
            },
            (PlaceKind::Local(slot), op, Some(ScalarKind::Int), _) => {
                let SlotLoc::I(dst, _) = self.slot_loc(*slot) else {
                    return false;
                };
                let r = self.compile_expr_i(value);
                self.emit_assign_op_i(dst, dst, r, op, place.span)
            }
            (PlaceKind::Global(gid), AssignOp::Set, Some(kind), _) => {
                let r = self.compile_expr_i(value);
                self.emit(
                    Op::StoreGlobalI { gid: *gid as u16, src: r, kind },
                    place.span,
                );
                true
            }
            (PlaceKind::Global(gid), AssignOp::Set, None, Some(kind)) => {
                let (val, some) = self.compile_expr_opt_i(value);
                self.emit(
                    Op::StoreGlobalOptI { gid: *gid as u16, val, some, kind },
                    place.span,
                );
                true
            }
            (PlaceKind::Global(gid), AssignOp::Set, None, None) => {
                let r = self.compile_expr_v(value);
                self.emit(Op::StoreGlobalV { gid: *gid as u16, src: r }, place.span);
                true
            }
            (PlaceKind::Global(gid), op, Some(ScalarKind::Int), _) => {
                let cur = self.alloc_i();
                self.emit(Op::LoadGlobalI { dst: cur, gid: *gid as u16 }, place.span);
                let r = self.compile_expr_i(value);
                if !self.emit_assign_op_i(cur, cur, r, op, place.span) {
                    return false;
                }
                self.emit(
                    Op::StoreGlobalI { gid: *gid as u16, src: cur, kind: ScalarKind::Int },
                    place.span,
                );
                true
            }
            // Array[i] = val or Array[i, j] = val (scalar)
            (PlaceKind::Index(base, indices), AssignOp::Set, Some(kind), _)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let r_val = self.compile_expr_i(value);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreGArr2I { gid: *gid as u16, r_i, r_j, r_val },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::StoreGArr1I { gid: *gid as u16, r_i, r_val },
                                place.span,
                            );
                        }
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let r_val = self.compile_expr_i(value);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreLArr2I { arr_v, r_i, r_j, r_val },
                                place.span,
                            );
                        } else {
                            self.emit(Op::StoreLArr1I { arr_v, r_i, r_val }, place.span);
                        }
                        true
                    }
                    PlaceKind::Field(obj_place, field) if indices.len() == 1 => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_i(value);
                        self.emit(
                            Op::StoreLFieldIdx1I { obj_v, field: *field as u16, r_i, r_val, kind },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            // Array[i] = opt_val or Array[i, j] = opt_val (unboxed ?Int / ?Scalar)
            (PlaceKind::Index(base, indices), AssignOp::Set, None, Some(kind))
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let (r_val, r_some) = self.compile_expr_opt_i(value);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreGArr2OptI {
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                    r_val,
                                    r_some,
                                    kind,
                                },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::StoreGArr1OptI {
                                    gid: *gid as u16,
                                    r_i,
                                    r_val,
                                    r_some,
                                    kind,
                                },
                                place.span,
                            );
                        }
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let (r_val, r_some) = self.compile_expr_opt_i(value);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreLArr2OptI { arr_v, r_i, r_j, r_val, r_some, kind },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::StoreLArr1OptI { arr_v, r_i, r_val, r_some, kind },
                                place.span,
                            );
                        }
                        true
                    }
                    PlaceKind::Field(obj_place, field) if indices.len() == 1 => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let (val, some) = self.compile_expr_opt_i(value);
                        self.emit(
                            Op::StoreLFieldIdx1OptI {
                                obj_v,
                                field: *field as u16,
                                r_i,
                                val,
                                some,
                                kind,
                            },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            // List[i] = val (scalar)
            (PlaceKind::Index(base, indices), AssignOp::Set, Some(kind), _)
                if matches!(base.ty, Ty::List(_)) && indices.len() == 1 =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_i(value);
                        self.emit(
                            Op::StoreGList1I { gid: *gid as u16, r_i, r_val, kind },
                            place.span,
                        );
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(list_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_i(value);
                        self.emit(
                            Op::StoreLList1I { list_v, r_i, r_val, kind },
                            place.span,
                        );
                        true
                    }
                    PlaceKind::Field(obj_place, field) => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_i(value);
                        self.emit(
                            Op::StoreLFieldIdx1I { obj_v, field: *field as u16, r_i, r_val, kind },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            // Array[i] op= val (Int)
            (PlaceKind::Index(base, indices), op, Some(ScalarKind::Int), _)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let r_rhs = self.compile_expr_i(value);
                        let cur = self.alloc_i();
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::LoadGArr2I { dst: cur, gid: *gid as u16, r_i, r_j },
                                place.span,
                            );
                            if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                                return false;
                            }
                            self.emit(
                                Op::StoreGArr2I { gid: *gid as u16, r_i, r_j, r_val: cur },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::LoadGArr1I { dst: cur, gid: *gid as u16, r_i },
                                place.span,
                            );
                            if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                                return false;
                            }
                            self.emit(
                                Op::StoreGArr1I { gid: *gid as u16, r_i, r_val: cur },
                                place.span,
                            );
                        }
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let r_rhs = self.compile_expr_i(value);
                        let cur = self.alloc_i();
                        if let Some(r_j) = r_j {
                            self.emit(Op::LoadLArr2I { dst: cur, arr_v, r_i, r_j }, place.span);
                            if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                                return false;
                            }
                            self.emit(Op::StoreLArr2I { arr_v, r_i, r_j, r_val: cur }, place.span);
                        } else {
                            self.emit(Op::LoadLArr1I { dst: cur, arr_v, r_i }, place.span);
                            if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                                return false;
                            }
                            self.emit(Op::StoreLArr1I { arr_v, r_i, r_val: cur }, place.span);
                        }
                        true
                    }
                    PlaceKind::Field(obj_place, field) if indices.len() == 1 => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_rhs = self.compile_expr_i(value);
                        let cur = self.alloc_i();
                        self.emit(
                            Op::LoadFieldArr1I { dst: cur, obj_v, field: *field as u16, r_i },
                            place.span,
                        );
                        if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                            return false;
                        }
                        self.emit(
                            Op::StoreLFieldIdx1I {
                                obj_v,
                                field: *field as u16,
                                r_i,
                                r_val: cur,
                                kind: ScalarKind::Int,
                            },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            // List[i] op= val (Int)
            (PlaceKind::Index(base, indices), op, Some(ScalarKind::Int), _)
                if matches!(base.ty, Ty::List(_)) && indices.len() == 1 =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_rhs = self.compile_expr_i(value);
                        let cur = self.alloc_i();
                        self.emit(Op::LoadGList1I { dst: cur, gid: *gid as u16, r_i }, place.span);
                        if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                            return false;
                        }
                        self.emit(
                            Op::StoreGList1I {
                                gid: *gid as u16,
                                r_i,
                                r_val: cur,
                                kind: ScalarKind::Int,
                            },
                            place.span,
                        );
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(list_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_rhs = self.compile_expr_i(value);
                        let cur = self.alloc_i();
                        self.emit(Op::LoadLList1I { dst: cur, list_v, r_i }, place.span);
                        if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                            return false;
                        }
                        self.emit(
                            Op::StoreLList1I {
                                list_v,
                                r_i,
                                r_val: cur,
                                kind: ScalarKind::Int,
                            },
                            place.span,
                        );
                        true
                    }
                    PlaceKind::Field(obj_place, field) => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_rhs = self.compile_expr_i(value);
                        let cur = self.alloc_i();
                        self.emit(
                            Op::LoadFieldList1I { dst: cur, obj_v, field: *field as u16, r_i },
                            place.span,
                        );
                        if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                            return false;
                        }
                        self.emit(
                            Op::StoreLFieldIdx1I {
                                obj_v,
                                field: *field as u16,
                                r_i,
                                r_val: cur,
                                kind: ScalarKind::Int,
                            },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            // seq[i].field = val or seq[i].field op= val (scalar field on Array/List of Struct)
            (PlaceKind::Field(obj_place, field), op, Some(kind), _)
                if matches!(obj_place.kind, PlaceKind::Index(..)) =>
            {
                let PlaceKind::Index(base, indices) = &obj_place.kind else {
                    return false;
                };
                if !matches!(base.ty, Ty::Array { .. } | Ty::List(_)) || indices.len() != 1 {
                    return false;
                }
                if op != AssignOp::Set && kind != ScalarKind::Int {
                    return false;
                }
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = if op == AssignOp::Set {
                            self.compile_expr_i(value)
                        } else {
                            let r_rhs = self.compile_expr_i(value);
                            let cur = self.alloc_i();
                            self.emit(
                                Op::LoadGIdxFieldI {
                                    dst: cur,
                                    gid: *gid as u16,
                                    r_i,
                                    field: *field as u16,
                                },
                                place.span,
                            );
                            if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                                return false;
                            }
                            cur
                        };
                        self.emit(
                            Op::StoreGIdxFieldI {
                                gid: *gid as u16,
                                r_i,
                                field: *field as u16,
                                r_val,
                                kind,
                            },
                            place.span,
                        );
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(seq_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = if op == AssignOp::Set {
                            self.compile_expr_i(value)
                        } else {
                            let r_rhs = self.compile_expr_i(value);
                            let cur = self.alloc_i();
                            self.emit(
                                Op::LoadLIdxFieldI {
                                    dst: cur,
                                    seq_v,
                                    r_i,
                                    field: *field as u16,
                                },
                                place.span,
                            );
                            if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                                return false;
                            }
                            cur
                        };
                        self.emit(
                            Op::StoreLIdxFieldI {
                                seq_v,
                                r_i,
                                field: *field as u16,
                                r_val,
                                kind,
                            },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            (PlaceKind::Index(base, indices), AssignOp::Min | AssignOp::Max, None, None)
                if matches!(base.ty, Ty::Array { .. })
                    && indices.len() == 1
                    && small_tuple_shape(&place.ty, self.program).is_some() =>
            {
                let (len, _) = small_tuple_shape(&place.ty, self.program).unwrap();
                let is_max = op == AssignOp::Max;
                let src_base = self.next_i;
                self.next_i += len as u16;
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        self.compile_expr_small_tuple(value, src_base, len);
                        self.emit(
                            Op::MinMaxGArr1SmallTuple {
                                gid: *gid as u16,
                                r_i,
                                src_base,
                                len,
                                is_max,
                            },
                            place.span,
                        );
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        self.compile_expr_small_tuple(value, src_base, len);
                        self.emit(
                            Op::MinMaxLArr1SmallTuple { arr_v, r_i, src_base, len, is_max },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            (PlaceKind::Index(base, indices), AssignOp::Set, None, None)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2)
                    && small_tuple_shape(&place.ty, self.program).is_some() =>
            {
                let (len, kinds) = small_tuple_shape(&place.ty, self.program).unwrap();
                let src_base = self.next_i;
                self.next_i += len as u16;
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        self.compile_expr_small_tuple(value, src_base, len);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreGArr2SmallTuple {
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                    src_base,
                                    len,
                                    kinds,
                                },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::StoreGArr1SmallTuple {
                                    gid: *gid as u16,
                                    r_i,
                                    src_base,
                                    len,
                                    kinds,
                                },
                                place.span,
                            );
                        }
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        self.compile_expr_small_tuple(value, src_base, len);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreLArr2SmallTuple {
                                    arr_v,
                                    r_i,
                                    r_j,
                                    src_base,
                                    len,
                                    kinds,
                                },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::StoreLArr1SmallTuple { arr_v, r_i, src_base, len, kinds },
                                place.span,
                            );
                        }
                        true
                    }
                    _ => false,
                }
            }
            (PlaceKind::Index(base, indices), AssignOp::Set, None, None)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let r_val = self.compile_expr_v(value);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreGArr2V { gid: *gid as u16, r_i, r_j, r_val },
                                place.span,
                            );
                        } else {
                            self.emit(
                                Op::StoreGArr1V { gid: *gid as u16, r_i, r_val },
                                place.span,
                            );
                        }
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let r_val = self.compile_expr_v(value);
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::StoreLArr2V { arr_v, r_i, r_j, r_val },
                                place.span,
                            );
                        } else {
                            self.emit(Op::StoreLArr1V { arr_v, r_i, r_val }, place.span);
                        }
                        true
                    }
                    PlaceKind::Field(obj_place, field) => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_v(value);
                        self.emit(
                            Op::StoreLFieldIdx1V { obj_v, field: *field as u16, r_i, r_val },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            (PlaceKind::Index(base, indices), AssignOp::Set, None, None)
                if matches!(base.ty, Ty::List(_)) && indices.len() == 1 =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_v(value);
                        self.emit(Op::StoreGList1V { gid: *gid as u16, r_i, r_val }, place.span);
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(list_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_v(value);
                        self.emit(Op::StoreLList1V { list_v, r_i, r_val }, place.span);
                        true
                    }
                    PlaceKind::Field(obj_place, field) => {
                        let PlaceKind::Local(slot) = obj_place.kind else {
                            return false;
                        };
                        let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_v(value);
                        self.emit(
                            Op::StoreLFieldIdx1V { obj_v, field: *field as u16, r_i, r_val },
                            place.span,
                        );
                        true
                    }
                    _ => false,
                }
            }
            (PlaceKind::Index(base, indices), AssignOp::Set, None, Some(kind))
                if matches!(base.ty, Ty::List(_)) && indices.len() == 1 =>
            {
                let PlaceKind::Field(obj_place, field) = &base.kind else {
                    return false;
                };
                let PlaceKind::Local(slot) = obj_place.kind else {
                    return false;
                };
                let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                    return false;
                };
                let r_i = self.compile_expr_i(&indices[0]);
                let (val, some) = self.compile_expr_opt_i(value);
                self.emit(
                    Op::StoreLFieldIdx1OptI {
                        obj_v,
                        field: *field as u16,
                        r_i,
                        val,
                        some,
                        kind,
                    },
                    place.span,
                );
                true
            }
            (PlaceKind::Field(obj_place, field), op, Some(kind), _) => {
                if op != AssignOp::Set && kind != ScalarKind::Int {
                    return false;
                }
                let PlaceKind::Local(slot) = obj_place.kind else {
                    return false;
                };
                let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                    return false;
                };
                let src_i = if op == AssignOp::Set {
                    self.compile_expr_i(value)
                } else {
                    let r_rhs = self.compile_expr_i(value);
                    let cur = self.alloc_i();
                    self.emit(Op::FieldI { dst: cur, obj_v, field: *field as u16 }, place.span);
                    if !self.emit_assign_op_i(cur, cur, r_rhs, op, place.span) {
                        return false;
                    }
                    cur
                };
                self.emit(
                    Op::StoreLFieldI { obj_v, field: *field as u16, src_i, kind },
                    place.span,
                );
                true
            }
            (PlaceKind::Field(obj_place, field), AssignOp::Set, None, Some(kind)) => {
                let PlaceKind::Local(slot) = obj_place.kind else {
                    return false;
                };
                let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                    return false;
                };
                let (val, some) = self.compile_expr_opt_i(value);
                self.emit(
                    Op::StoreLFieldOptI { obj_v, field: *field as u16, val, some, kind },
                    place.span,
                );
                true
            }
            (PlaceKind::Field(obj_place, field), AssignOp::Set, None, None) => {
                let PlaceKind::Local(slot) = obj_place.kind else {
                    return false;
                };
                let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                    return false;
                };
                let src_v = self.compile_expr_v(value);
                self.emit(
                    Op::StoreLFieldV { obj_v, field: *field as u16, src_v },
                    place.span,
                );
                true
            }
            _ => false,
        }
    }

    fn compile_place_path(&mut self, place: &'p Place) -> Option<u16> {
        let mut chain: Vec<&'p Place> = Vec::new();
        let mut cur = place;
        let root = loop {
            match &cur.kind {
                PlaceKind::Local(slot) => match self.slot_loc(*slot) {
                    SlotLoc::V(v) => break PlaceRoot::LocalV(v),
                    _ => return None,
                },
                PlaceKind::Global(gid) => break PlaceRoot::Global(*gid as u16),
                PlaceKind::Field(base, _) => {
                    chain.push(cur);
                    cur = base;
                }
                PlaceKind::Index(base, indices)
                    if matches!(base.ty, Ty::Array { .. } | Ty::List(_))
                        && (indices.len() == 1 || indices.len() == 2) =>
                {
                    if matches!(base.ty, Ty::List(_)) && indices.len() != 1 {
                        return None;
                    }
                    chain.push(cur);
                    cur = base;
                }
                _ => return None,
            }
        };
        if chain.is_empty()
            && !matches!(
                place.ty,
                Ty::List(_) | Ty::Array { .. } | Ty::Str | Ty::Set(_) | Ty::Map(_, _)
            )
        {
            return None;
        }
        let mut steps = Vec::with_capacity(chain.len());
        for p in chain.into_iter().rev() {
            match &p.kind {
                PlaceKind::Field(_, field) => steps.push(PlaceStep::Field(*field as u16)),
                PlaceKind::Index(_, indices) if indices.len() == 1 => {
                    let r_i = self.compile_expr_i(&indices[0]);
                    steps.push(PlaceStep::Index1(r_i));
                }
                PlaceKind::Index(_, indices) if indices.len() == 2 => {
                    let r_i = self.compile_expr_i(&indices[0]);
                    let r_j = self.compile_expr_i(&indices[1]);
                    steps.push(PlaceStep::Index2(r_i, r_j));
                }
                _ => return None,
            }
        }
        let idx = u16::try_from(self.places.len()).ok()?;
        self.places.push(PlacePath { root, steps });
        Some(idx)
    }

    fn try_compile_assign_place_path(
        &mut self,
        place: &'p Place,
        op: AssignOp,
        value: &'p Expr,
    ) -> bool {
        if let PlaceKind::Index(map_place, indices) = &place.kind {
            if matches!(&map_place.ty, Ty::Map(kty, _) if !crate::value::ty_has_cached(kty, self.program))
                && indices.len() == 1
            {
                let saved_ops = self.ops.len();
                let saved_spans = self.spans.len();
                let saved_places = self.places.len();
                let saved_fill_tys = self.fill_tys.len();
                if let Some(path) = self.compile_place_path(map_place) {
                    let key_v = self.compile_expr_v(&indices[0]);
                    let val_v = self.compile_expr_v(value);
                    let val_ty = self.fill_tys.len() as u16;
                    self.fill_tys.push(&place.ty);
                    self.emit(
                        Op::MapPlaceAssign { path, key_v, val_v, op, val_ty },
                        place.span,
                    );
                    if val_v >= self.num_local_v {
                        self.emit(Op::ClearV { reg: val_v }, place.span);
                    }
                    if key_v >= self.num_local_v {
                        self.emit(Op::ClearV { reg: key_v }, place.span);
                    }
                    return true;
                }
                self.ops.truncate(saved_ops);
                self.spans.truncate(saved_spans);
                self.places.truncate(saved_places);
                self.fill_tys.truncate(saved_fill_tys);
            }
        }
        let val_kind = ScalarKind::of_ty(&place.ty, self.program);
        let opt_kind = ScalarKind::of_opt_ty(&place.ty, self.program);
        if val_kind == Some(ScalarKind::U64) && op != AssignOp::Set {
            match &place.kind {
                PlaceKind::Local(slot) => {
                    let SlotLoc::I(dst, _) = self.slot_loc(*slot) else {
                        return false;
                    };
                    let r_rhs = self.compile_expr_i(value);
                    return self.emit_assign_op_u64(dst, dst, r_rhs, op, place.span);
                }
                PlaceKind::Global(gid) => {
                    let r_rhs = self.compile_expr_i(value);
                    let cur = self.alloc_i();
                    self.emit(Op::LoadGlobalI { dst: cur, gid: *gid as u16 }, place.span);
                    if !self.emit_assign_op_u64(cur, cur, r_rhs, op, place.span) {
                        return false;
                    }
                    self.emit(
                        Op::StoreGlobalI { gid: *gid as u16, src: cur, kind: ScalarKind::U64 },
                        place.span,
                    );
                    return true;
                }
                _ => {}
            }
        }
        if matches!(place.kind, PlaceKind::Local(_) | PlaceKind::Global(_)) {
            return false;
        }
        let saved_ops = self.ops.len();
        let saved_spans = self.spans.len();
        let saved_places = self.places.len();
        let Some(path) = self.compile_place_path(place) else {
            self.ops.truncate(saved_ops);
            self.spans.truncate(saved_spans);
            self.places.truncate(saved_places);
            return false;
        };
        match (op, val_kind, opt_kind) {
            (AssignOp::Set, Some(kind), _) => {
                let r_val = self.compile_expr_i(value);
                self.emit(Op::StorePlaceI { path, r_val, kind }, place.span);
                true
            }
            (AssignOp::Set, None, Some(kind)) => {
                let (r_val, r_some) = self.compile_expr_opt_i(value);
                self.emit(Op::StorePlaceOptI { path, r_val, r_some, kind }, place.span);
                true
            }
            (AssignOp::Set, None, None) => {
                let r_val = self.compile_expr_v(value);
                self.emit(Op::StorePlaceV { path, r_val }, place.span);
                true
            }
            (op, Some(kind @ (ScalarKind::Int | ScalarKind::U64)), _)
                if matches!(
                    op,
                    AssignOp::Binary(
                        BinOp::Add
                            | BinOp::Sub
                            | BinOp::Mul
                            | BinOp::Div
                            | BinOp::Rem
                            | BinOp::BitAnd
                            | BinOp::BitOr
                            | BinOp::BitXor
                            | BinOp::Shl
                            | BinOp::Shr
                    ) | AssignOp::Min
                        | AssignOp::Max
                ) =>
            {
                let r_rhs = self.compile_expr_i(value);
                self.emit(Op::AssignPlaceI { path, r_rhs, op, kind }, place.span);
                true
            }
            _ => {
                self.ops.truncate(saved_ops);
                self.spans.truncate(saved_spans);
                self.places.truncate(saved_places);
                false
            }
        }
    }

    fn emit_assign_op_u64(
        &mut self,
        dst: u16,
        a: u16,
        b: u16,
        op: AssignOp,
        span: Span,
    ) -> bool {
        let inst = match op {
            AssignOp::Binary(
                bop @ (BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Div
                | BinOp::Rem
                | BinOp::Shl
                | BinOp::Shr),
            ) => Op::BinU64 { dst, a, b, op: bop },
            AssignOp::Binary(BinOp::BitAnd) => Op::BitAndU64 { dst, a, b },
            AssignOp::Binary(BinOp::BitOr) => Op::BitOrU64 { dst, a, b },
            AssignOp::Binary(BinOp::BitXor) => Op::BitXorU64 { dst, a, b },
            AssignOp::Set => Op::MoveI { dst, src: b },
            _ => return false,
        };
        self.emit(inst, span);
        true
    }

    fn try_compile_sort_stmt(
        &mut self,
        place: &'p Place,
        desc: Option<&'p Expr>,
        span: Span,
    ) -> bool {
        let elem_ty_ok = match &place.ty {
            Ty::List(elem) => ty_uses_structural_cmp(elem, self.program),
            Ty::Array { elem, dims } => dims.len() == 1 && ty_uses_structural_cmp(elem, self.program),
            Ty::Str => true,
            _ => false,
        };
        if !elem_ty_ok {
            return false;
        }
        let saved_ops = self.ops.len();
        let saved_spans = self.spans.len();
        let saved_places = self.places.len();
        let Some(path) = self.compile_place_path(place) else {
            self.ops.truncate(saved_ops);
            self.spans.truncate(saved_spans);
            self.places.truncate(saved_places);
            return false;
        };
        let desc_r = match desc {
            Some(e) => self.compile_expr_i(e),
            None => u16::MAX,
        };
        self.emit(Op::SortPlace { path, desc_r }, span);
        true
    }

    fn emit_assign_op_i(
        &mut self,
        dst: u16,
        a: u16,
        b: u16,
        op: AssignOp,
        span: Span,
    ) -> bool {
        let inst = match op {
            AssignOp::Binary(BinOp::Add) => Op::AddI { dst, a, b },
            AssignOp::Binary(BinOp::Sub) => Op::SubI { dst, a, b },
            AssignOp::Binary(BinOp::Mul) => Op::MulI { dst, a, b },
            AssignOp::Binary(BinOp::Div) => Op::DivI { dst, a, b },
            AssignOp::Binary(BinOp::Rem) => Op::RemI { dst, a, b },
            AssignOp::Binary(BinOp::BitAnd) => Op::BitAndI { dst, a, b },
            AssignOp::Binary(BinOp::BitOr) => Op::BitOrI { dst, a, b },
            AssignOp::Binary(BinOp::BitXor) => Op::BitXorI { dst, a, b },
            AssignOp::Binary(BinOp::Shl) => Op::ShlI { dst, a, b },
            AssignOp::Binary(BinOp::Shr) => Op::ShrI { dst, a, b },
            AssignOp::Min => Op::MinI { dst, a, b },
            AssignOp::Max => Op::MaxI { dst, a, b },
            AssignOp::Set => Op::MoveI { dst, src: b },
            _ => return false,
        };
        self.emit(inst, span);
        true
    }

    fn emit_branch_if_false(&mut self, cond: &'p Expr) -> usize {
        if let ExprKind::IsSome(inner) = &cond.kind {
            if ScalarKind::of_opt_ty(&inner.ty, self.program).is_some() {
                let (_, r_some) = self.compile_expr_opt_i(inner);
                return self.emit(Op::JmpIfZero { reg: r_some, target: 0 }, cond.span);
            } else {
                let src_v = self.compile_expr_v(inner);
                let r_some = self.alloc_i();
                self.emit(Op::IsSomeV { dst_i: r_some, src_v }, cond.span);
                return self.emit(Op::JmpIfZero { reg: r_some, target: 0 }, cond.span);
            }
        }
        if let ExprKind::Cmp(op, lhs, rhs) = &cond.kind {
            if ScalarKind::of_ty(&lhs.ty, self.program).is_some()
                && ScalarKind::of_ty(&rhs.ty, self.program).is_some()
                && lhs.ty != Ty::U64
                && rhs.ty != Ty::U64
                && ty_uses_structural_cmp(&lhs.ty, self.program)
                && matches!(
                    op,
                    CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge | CmpOp::Eq | CmpOp::Ne
                )
            {
                let a = self.compile_expr_i(lhs);
                let b = self.compile_expr_i(rhs);
                let inv = match op {
                    CmpOp::Lt => Op::JmpGeI { a, b, target: 0 },
                    CmpOp::Le => Op::JmpGtI { a, b, target: 0 },
                    CmpOp::Gt => Op::JmpLeI { a, b, target: 0 },
                    CmpOp::Ge => Op::JmpLtI { a, b, target: 0 },
                    CmpOp::Eq => Op::JmpNeI { a, b, target: 0 },
                    CmpOp::Ne => Op::JmpEqI { a, b, target: 0 },
                    _ => unreachable!(),
                };
                return self.emit(inv, cond.span);
            }
        }
        if let ExprKind::Unary(UnOp::Not, inner) = &cond.kind {
            let r = self.compile_expr_i(inner);
            return self.emit(Op::JmpIfNonZero { reg: r, target: 0 }, cond.span);
        }
        let r = self.compile_expr_i(cond);
        self.emit(Op::JmpIfZero { reg: r, target: 0 }, cond.span)
    }

    /// Compiles an optional-scalar expression into `(r_val, r_some)` unboxed `i64` registers.
    fn compile_expr_opt_i(&mut self, expr: &'p Expr) -> (u16, u16) {
        let span = expr.span;
        match &expr.kind {
            ExprKind::None => {
                let v = self.alloc_i();
                let s = self.alloc_i();
                self.emit(Op::ConstImmI { dst: s, val: 0 }, span);
                (v, s)
            }
            ExprKind::Some(inner) => {
                let v = self.compile_expr_i(inner);
                let s = self.alloc_i();
                self.emit(Op::ConstImmI { dst: s, val: 1 }, span);
                (v, s)
            }
            ExprKind::Local(slot) => match self.slot_loc(*slot) {
                SlotLoc::OptI { val, some, .. } => (val, some),
                SlotLoc::V(src_v) => {
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(Op::UnboxOptI { dst_val, dst_some, src_v }, span);
                    (dst_val, dst_some)
                }
                SlotLoc::I(v, _) => {
                    let s = self.alloc_i();
                    self.emit(Op::ConstImmI { dst: s, val: 1 }, span);
                    (v, s)
                }
                SlotLoc::SmallTuple { base, .. } => {
                    let s = self.alloc_i();
                    self.emit(Op::ConstImmI { dst: s, val: 0 }, span);
                    (base, s)
                }
            },
            ExprKind::Global(gid) => {
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::LoadGlobalOptI { dst_val, dst_some, gid: *gid as u16 },
                    span,
                );
                (dst_val, dst_some)
            }
            ExprKind::Builtin(Builtin::PosNext, args) if args.len() == 2 => {
                let src = self.compile_expr_i(&args[0]);
                let len = self.compile_expr_i(&args[1]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(Op::PosNext { dst_val, dst_some, src, len }, span);
                (dst_val, dst_some)
            }
            ExprKind::Builtin(Builtin::PosPrev, args) if args.len() == 1 => {
                let src = self.compile_expr_i(&args[0]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(Op::PosPrev { dst_val, dst_some, src }, span);
                (dst_val, dst_some)
            }
            ExprKind::Builtin(Builtin::PosOffset(d), args) if args.len() == 2 => {
                let src = self.compile_expr_i(&args[0]);
                let delta = self.compile_expr_i(&args[1]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::PosOffset { dst_val, dst_some, src, delta, dom: *d as u16 },
                    span,
                );
                (dst_val, dst_some)
            }
            ExprKind::Builtin(Builtin::PosTryOrd(d), args) if args.len() == 1 => {
                let src = self.compile_expr_i(&args[0]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::PosTryOrd { dst_val, dst_some, src, dom: *d as u16 },
                    span,
                );
                (dst_val, dst_some)
            }
            ExprKind::Builtin(b @ (Builtin::Last | Builtin::First), args) if args.len() == 1 => {
                let is_last = matches!(b, Builtin::Last);
                if let ExprKind::Index(base, indices) = &args[0].kind {
                    if indices.len() == 1 && matches!(base.ty, Ty::Array { .. } | Ty::List(_)) {
                        let root_info = match &base.kind {
                            ExprKind::Global(gid) => Some((*gid as u16, true)),
                            ExprKind::Local(slot) => match self.slot_loc(*slot) {
                                SlotLoc::V(v) => Some((v, false)),
                                _ => None,
                            },
                            _ => None,
                        };
                        if let Some((root, is_global)) = root_info {
                            let r_i = self.compile_expr_i(&indices[0]);
                            let dst_val = self.alloc_i();
                            let dst_some = self.alloc_i();
                            self.emit(
                                Op::SeqIdxEndOptI {
                                    dst_val,
                                    dst_some,
                                    root,
                                    is_global,
                                    r_i,
                                    is_last,
                                },
                                span,
                            );
                            return (dst_val, dst_some);
                        }
                    }
                }
                let seq_v = self.compile_expr_v(&args[0]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                if is_last {
                    self.emit(Op::SeqLastOptI { dst_val, dst_some, seq_v }, span);
                } else {
                    self.emit(Op::SeqFirstOptI { dst_val, dst_some, seq_v }, span);
                }
                if seq_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: seq_v }, span);
                }
                (dst_val, dst_some)
            }
            ExprKind::Builtin(Builtin::MapGet, args)
                if args.len() == 2
                    && matches!(&args[0].ty, Ty::Map(kty, _) if !crate::value::ty_has_cached(kty, self.program)) =>
            {
                let map_v = self.compile_expr_v(&args[0]);
                let key_v = self.compile_expr_v(&args[1]);
                let tmp_v = self.alloc_v();
                self.emit(Op::MapGetV { dst: tmp_v, map_v, key_v }, span);
                if key_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: key_v }, span);
                }
                if map_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: map_v }, span);
                }
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(Op::UnboxOptI { dst_val, dst_some, src_v: tmp_v }, span);
                self.emit(Op::ClearV { reg: tmp_v }, span);
                (dst_val, dst_some)
            }
            ExprKind::Builtin(
                b @ (Builtin::FirstGe | Builtin::FirstGt | Builtin::LastLe | Builtin::LastLt),
                args,
            ) if args.len() == 2
                && matches!(args[0].ty, Ty::Set(_) | Ty::Map(_, _))
                && !crate::value::ty_has_cached(&args[1].ty, self.program) => {
                let mode = match b {
                    Builtin::FirstGe => 0,
                    Builtin::FirstGt => 1,
                    Builtin::LastLe => 2,
                    Builtin::LastLt => 3,
                    _ => unreachable!(),
                };
                let src_v = self.compile_expr_v(&args[0]);
                let key_v = self.compile_expr_v(&args[1]);
                let tmp_v = self.alloc_v();
                self.emit(Op::SetMapBoundV { dst: tmp_v, src_v, key_v, mode }, span);
                if key_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: key_v }, span);
                }
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(Op::UnboxOptI { dst_val, dst_some, src_v: tmp_v }, span);
                self.emit(Op::ClearV { reg: tmp_v }, span);
                (dst_val, dst_some)
            }
            ExprKind::Mutate(m @ (Mutation::PopFirst | Mutation::PopLast), place, args)
                if args.is_empty() && matches!(place.ty, Ty::Set(_) | Ty::Map(_, _)) =>
            {
                let saved_ops = self.ops.len();
                let saved_spans = self.spans.len();
                let saved_places = self.places.len();
                if let Some(path) = self.compile_place_path(place) {
                    let tmp_v = self.alloc_v();
                    self.emit(
                        Op::SetMapPopEndPlace {
                            dst: tmp_v,
                            path,
                            is_last: matches!(m, Mutation::PopLast),
                        },
                        span,
                    );
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(Op::UnboxOptI { dst_val, dst_some, src_v: tmp_v }, span);
                    self.emit(Op::ClearV { reg: tmp_v }, span);
                    (dst_val, dst_some)
                } else {
                    self.ops.truncate(saved_ops);
                    self.spans.truncate(saved_spans);
                    self.places.truncate(saved_places);
                    let src_v = self.emit_fallback_expr_v(expr);
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(Op::UnboxOptI { dst_val, dst_some, src_v }, span);
                    (dst_val, dst_some)
                }
            }
            ExprKind::Mutate(Mutation::Pop, place, args)
                if args.is_empty() && matches!(place.ty, Ty::List(_)) =>
            match &place.kind {
                PlaceKind::Local(slot) => {
                    if let SlotLoc::V(list_v) = self.slot_loc(*slot) {
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        self.emit(Op::ListPopOptIL { dst_val, dst_some, list_v }, span);
                        (dst_val, dst_some)
                    } else {
                        let src_v = self.emit_fallback_expr_v(expr);
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        self.emit(Op::UnboxOptI { dst_val, dst_some, src_v }, span);
                        (dst_val, dst_some)
                    }
                }
                PlaceKind::Global(gid) => {
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(Op::ListPopOptIG { dst_val, dst_some, gid: *gid as u16 }, span);
                    (dst_val, dst_some)
                }
                PlaceKind::Field(obj_place, field) => {
                    if let PlaceKind::Local(slot) = obj_place.kind {
                        if let SlotLoc::V(obj_v) = self.slot_loc(slot) {
                            let dst_val = self.alloc_i();
                            let dst_some = self.alloc_i();
                            self.emit(
                                Op::ListPopFieldOptIL {
                                    dst_val,
                                    dst_some,
                                    obj_v,
                                    field: *field as u16,
                                },
                                span,
                            );
                            return (dst_val, dst_some);
                        }
                    }
                    let src_v = self.emit_fallback_expr_v(expr);
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(Op::UnboxOptI { dst_val, dst_some, src_v }, span);
                    (dst_val, dst_some)
                }
                _ => {
                    let src_v = self.emit_fallback_expr_v(expr);
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(Op::UnboxOptI { dst_val, dst_some, src_v }, span);
                    (dst_val, dst_some)
                }
            },
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    ExprKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        if indices.len() == 2 {
                            let r_j = self.compile_expr_i(&indices[1]);
                            self.emit(
                                Op::LoadGArr2OptI {
                                    dst_val,
                                    dst_some,
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                },
                                span,
                            );
                        } else {
                            self.emit(
                                Op::LoadGArr1OptI {
                                    dst_val,
                                    dst_some,
                                    gid: *gid as u16,
                                    r_i,
                                },
                                span,
                            );
                        }
                        (dst_val, dst_some)
                    }
                    ExprKind::Local(slot) => {
                        if let SlotLoc::V(arr_v) = self.slot_loc(*slot) {
                            let r_i = self.compile_expr_i(&indices[0]);
                            let dst_val = self.alloc_i();
                            let dst_some = self.alloc_i();
                            if indices.len() == 2 {
                                let r_j = self.compile_expr_i(&indices[1]);
                                self.emit(
                                    Op::LoadLArr2OptI { dst_val, dst_some, arr_v, r_i, r_j },
                                    span,
                                );
                            } else {
                                self.emit(
                                    Op::LoadLArr1OptI { dst_val, dst_some, arr_v, r_i },
                                    span,
                                );
                            }
                            (dst_val, dst_some)
                        } else {
                            let src_v = self.emit_fallback_expr_v(expr);
                            let dst_val = self.alloc_i();
                            let dst_some = self.alloc_i();
                            self.emit(Op::UnboxOptI { dst_val, dst_some, src_v }, span);
                            (dst_val, dst_some)
                        }
                    }
                    ExprKind::Index(outer, outer_indices)
                        if indices.len() == 1
                            && outer_indices.len() == 1
                            && matches!(outer.ty, Ty::Array { .. } | Ty::List(_)) =>
                    {
                        let root_info = match &outer.kind {
                            ExprKind::Global(gid) => Some((*gid as u16, true)),
                            ExprKind::Local(slot) => match self.slot_loc(*slot) {
                                SlotLoc::V(v) => Some((v, false)),
                                _ => None,
                            },
                            _ => None,
                        };
                        if let Some((root, is_global)) = root_info {
                            let r_i = self.compile_expr_i(&outer_indices[0]);
                            let r_j = self.compile_expr_i(&indices[0]);
                            let dst_val = self.alloc_i();
                            let dst_some = self.alloc_i();
                            self.emit(
                                Op::LoadNestedArrOptI {
                                    dst_val,
                                    dst_some,
                                    root,
                                    is_global,
                                    r_i,
                                    r_j,
                                },
                                span,
                            );
                            return (dst_val, dst_some);
                        }
                        let arr_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        self.emit(Op::LoadLArr1OptI { dst_val, dst_some, arr_v, r_i }, span);
                        if arr_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: arr_v }, span);
                        }
                        (dst_val, dst_some)
                    }
                    ExprKind::Field(inner_base, field) if indices.len() == 1 => {
                        if let ExprKind::Index(seq, inner_indices) = &inner_base.kind {
                            if inner_indices.len() == 1
                                && matches!(seq.ty, Ty::List(_) | Ty::Array { .. })
                            {
                                let root_info = match &seq.kind {
                                    ExprKind::Global(gid) => Some((*gid as u16, true)),
                                    ExprKind::Local(slot) => match self.slot_loc(*slot) {
                                        SlotLoc::V(v) => Some((v, false)),
                                        _ => None,
                                    },
                                    _ => None,
                                };
                                if let Some((root, is_global)) = root_info {
                                    let r_i = self.compile_expr_i(&inner_indices[0]);
                                    let r_j = self.compile_expr_i(&indices[0]);
                                    let dst_val = self.alloc_i();
                                    let dst_some = self.alloc_i();
                                    self.emit(
                                        Op::LoadIdxFieldIdxOptI {
                                            dst_val,
                                            dst_some,
                                            root,
                                            is_global,
                                            r_i,
                                            field: *field as u16,
                                            r_j,
                                        },
                                        span,
                                    );
                                    return (dst_val, dst_some);
                                }
                            }
                        }
                        let arr_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        self.emit(Op::LoadLArr1OptI { dst_val, dst_some, arr_v, r_i }, span);
                        if arr_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: arr_v }, span);
                        }
                        (dst_val, dst_some)
                    }
                    _ => {
                        let arr_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        if indices.len() == 2 {
                            let r_j = self.compile_expr_i(&indices[1]);
                            self.emit(
                                Op::LoadLArr2OptI { dst_val, dst_some, arr_v, r_i, r_j },
                                span,
                            );
                        } else {
                            self.emit(
                                Op::LoadLArr1OptI { dst_val, dst_some, arr_v, r_i },
                                span,
                            );
                        }
                        if arr_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: arr_v }, span);
                        }
                        (dst_val, dst_some)
                    }
                }
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _))
                    && indices.len() == 1 =>
            {
                let list_v = self.compile_expr_v(base);
                let r_i = self.compile_expr_i(&indices[0]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(Op::LoadLList1OptI { dst_val, dst_some, list_v, r_i }, span);
                if list_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: list_v }, span);
                }
                (dst_val, dst_some)
            }
            ExprKind::Field(base, idx) => {
                let obj_v = self.compile_expr_v(base);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::FieldOptI { dst_val, dst_some, obj_v, field: *idx as u16 },
                    span,
                );
                if obj_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: obj_v }, span);
                }
                (dst_val, dst_some)
            }
            ExprKind::Builtin(b @ (Builtin::SetMin | Builtin::SetMax), args) if args.len() == 1 => {
                let src_v = self.compile_expr_v(&args[0]);
                let tmp_v = self.alloc_v();
                self.emit(
                    Op::SetMinMaxV { dst: tmp_v, src_v, is_max: matches!(b, Builtin::SetMax) },
                    span,
                );
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(Op::UnboxOptI { dst_val, dst_some, src_v: tmp_v }, span);
                self.emit(Op::ClearV { reg: tmp_v }, span);
                (dst_val, dst_some)
            }
            ExprKind::Call(fn_id, args) => {
                if let Some((dst_val, _, dst_some)) =
                    self.try_compile_call_mix(*fn_id, args, CallRet::OptI, span)
                {
                    (dst_val, dst_some)
                } else {
                    let src_v = self.emit_fallback_expr_v(expr);
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(Op::UnboxOptI { dst_val, dst_some, src_v }, span);
                    (dst_val, dst_some)
                }
            }
            ExprKind::CallExpr(callee, args) => {
                if let Some((dst_val, _, dst_some)) =
                    self.try_compile_call_closure(callee, args, CallRet::OptI, span)
                {
                    (dst_val, dst_some)
                } else {
                    let src_v = self.emit_fallback_expr_v(expr);
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(Op::UnboxOptI { dst_val, dst_some, src_v }, span);
                    (dst_val, dst_some)
                }
            }
            ExprKind::If(cond, then_e, else_e) => {
                if let Some(b) = Self::const_bool(cond) {
                    return if b {
                        self.compile_expr_opt_i(then_e)
                    } else {
                        self.compile_expr_opt_i(else_e)
                    };
                }
                if let (ExprKind::Some(inner), ExprKind::None) = (&then_e.kind, &else_e.kind) {
                    let dst_val = self.alloc_i();
                    let dst_some = self.alloc_i();
                    self.emit(Op::ConstImmI { dst: dst_some, val: 0 }, span);
                    let j_else = self.emit_branch_if_false(cond);
                    let tv = self.compile_expr_i(inner);
                    if dst_val != tv {
                        self.emit(Op::MoveI { dst: dst_val, src: tv }, inner.span);
                    }
                    self.emit(Op::ConstImmI { dst: dst_some, val: 1 }, then_e.span);
                    let end_pc = self.ops.len() as u32;
                    self.patch(j_else, end_pc);
                    return (dst_val, dst_some);
                }
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                let j_else = self.emit_branch_if_false(cond);
                let (tv, ts) = self.compile_expr_opt_i(then_e);
                self.emit(Op::MoveI { dst: dst_val, src: tv }, then_e.span);
                self.emit(Op::MoveI { dst: dst_some, src: ts }, then_e.span);
                let j_end = self.emit(Op::Jmp { target: 0 }, span);
                let else_pc = self.ops.len() as u32;
                self.patch(j_else, else_pc);
                let (ev, es) = self.compile_expr_opt_i(else_e);
                self.emit(Op::MoveI { dst: dst_val, src: ev }, else_e.span);
                self.emit(Op::MoveI { dst: dst_some, src: es }, else_e.span);
                let end_pc = self.ops.len() as u32;
                self.patch(j_end, end_pc);
                (dst_val, dst_some)
            }
            ExprKind::Do(block) => {
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                let prev_pinned = self.pinned_v;
                self.pinned_v = self.next_v;
                self.give_targets.push(GiveTarget::OptI { dst_val, dst_some, exits: Vec::new() });
                self.compile_stmts(&block.stmts);
                let mut target = self.give_targets.pop().unwrap();
                self.pinned_v = prev_pinned;
                let end_pc = self.ops.len() as u32;
                for j in target.exits_mut().drain(..) {
                    self.patch(j, end_pc);
                }
                (dst_val, dst_some)
            }
            _ => {
                let src_v = self.emit_fallback_expr_v(expr);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(Op::UnboxOptI { dst_val, dst_some, src_v }, span);
                (dst_val, dst_some)
            }
        }
    }

    fn compile_expr_i(&mut self, expr: &'p Expr) -> u16 {
        let span = expr.span;
        match &expr.kind {
            ExprKind::Int(v) => {
                let dst = self.alloc_i();
                self.emit_const_i(dst, *v, span);
                dst
            }
            ExprKind::Bool(b) => {
                let dst = self.alloc_i();
                self.emit(Op::ConstImmI { dst, val: i32::from(*b) }, span);
                dst
            }
            ExprKind::Char(c) => {
                let dst = self.alloc_i();
                self.emit(Op::ConstImmI { dst, val: i32::from(*c) }, span);
                dst
            }
            ExprKind::Local(slot) => match self.slot_loc(*slot) {
                SlotLoc::I(r, _) => r,
                SlotLoc::V(v) => {
                    let dst = self.alloc_i();
                    self.emit(Op::UnboxI { dst_i: dst, src_v: v }, span);
                    dst
                }
                SlotLoc::OptI { val, .. } => val,
                SlotLoc::SmallTuple { base, .. } => base,
            },
            ExprKind::Global(gid) => {
                let dst = self.alloc_i();
                self.emit(Op::LoadGlobalI { dst, gid: *gid as u16 }, span);
                dst
            }
            ExprKind::DomainLen(d) => {
                let dst = self.alloc_i();
                self.emit(Op::DomainLen { dst, dom: *d as u16 }, span);
                dst
            }
            ExprKind::DomainLo(d) => {
                let dst = self.alloc_i();
                self.emit(Op::DomainLo { dst, dom: *d as u16 }, span);
                dst
            }
            ExprKind::IsSome(inner) => {
                if ScalarKind::of_opt_ty(&inner.ty, self.program).is_some() {
                    let (_, s) = self.compile_expr_opt_i(inner);
                    s
                } else {
                    let src_v = self.compile_expr_v(inner);
                    let dst_i = self.alloc_i();
                    self.emit(Op::IsSomeV { dst_i, src_v }, span);
                    dst_i
                }
            }
            ExprKind::Unwrap(inner, _) => {
                if ScalarKind::of_opt_ty(&inner.ty, self.program).is_some() {
                    let (val, some) = self.compile_expr_opt_i(inner);
                    let dst = self.alloc_i();
                    self.emit(Op::UnwrapOptI { dst, val, some }, span);
                    dst
                } else {
                    let src_v = self.compile_expr_v(inner);
                    let dst_i = self.alloc_i();
                    self.emit(Op::UnwrapVI { dst_i, src_v }, span);
                    dst_i
                }
            }
            ExprKind::Binary(BinOp::Coalesce, lhs, rhs)
                if ScalarKind::of_opt_ty(&lhs.ty, self.program).is_some() =>
            {
                let (val, some) = self.compile_expr_opt_i(lhs);
                let dst = self.alloc_i();
                self.emit(Op::MoveI { dst, src: val }, lhs.span);
                let j_done = self.emit(Op::JmpIfNonZero { reg: some, target: 0 }, span);
                let rv = self.compile_expr_i(rhs);
                if rv != dst {
                    self.emit(Op::MoveI { dst, src: rv }, rhs.span);
                }
                let end_pc = self.ops.len() as u32;
                self.patch(j_done, end_pc);
                dst
            }
            ExprKind::Tuple(fields) if fields.len() == 1 => self.compile_expr_i(&fields[0]),
            ExprKind::Field(base, 0)
                if matches!(
                    ScalarKind::of_ty(&base.ty, self.program),
                    Some(ScalarKind::OneIntStruct)
                ) =>
            {
                self.compile_expr_i(base)
            }
            ExprKind::Field(base, idx) => {
                if let ExprKind::Local(slot) = &base.kind {
                    if let SlotLoc::SmallTuple { base: b, len, .. } = self.slot_loc(*slot) {
                        if *idx < len as usize {
                            return b + *idx as u16;
                        }
                    }
                }
                if let ExprKind::Tuple(fs) = &base.kind {
                    if *idx < fs.len() {
                        return self.compile_expr_i(&fs[*idx]);
                    }
                }
                let obj_v = self.compile_expr_v(base);
                let dst = self.alloc_i();
                self.emit(Op::FieldI { dst, obj_v, field: *idx as u16 }, span);
                if obj_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: obj_v }, span);
                }
                dst
            }
            ExprKind::RangeCheck { value, lo, hi, text } => {
                let val = self.compile_expr_i(value);
                let r_lo = match lo {
                    Some(lo) => self.compile_expr_i(lo),
                    None => {
                        let r = self.alloc_i();
                        self.emit_const_i(r, cpl_ir::INT_MIN, span);
                        r
                    }
                };
                let r_hi = match hi {
                    Some(hi) => self.compile_expr_i(hi),
                    None => {
                        let r = self.alloc_i();
                        self.emit_const_i(r, i64::MAX, span);
                        r
                    }
                };
                let text_idx = self.range_texts.len() as u16;
                self.range_texts.push(text.as_str());
                let dst = self.alloc_i();
                self.emit(Op::RangeCheckI { dst, val, lo: r_lo, hi: r_hi, text_idx }, span);
                dst
            }
            ExprKind::Unary(UnOp::Not, inner) => {
                let a = self.compile_expr_i(inner);
                let dst = self.alloc_i();
                self.emit(Op::NotBool { dst, a }, span);
                dst
            }
            ExprKind::Unary(UnOp::Neg, inner) if matches!(expr.ty, Ty::Int) => {
                let a = self.compile_expr_i(inner);
                let dst = self.alloc_i();
                self.emit(Op::NegI { dst, a }, span);
                dst
            }
            ExprKind::Unary(UnOp::Neg, inner) if expr.ty == Ty::U64 => {
                let a = self.compile_expr_i(inner);
                let dst = self.alloc_i();
                self.emit(Op::NegU64 { dst, a }, span);
                dst
            }
            ExprKind::Unary(UnOp::BitNot, inner) if matches!(expr.ty, Ty::Int | Ty::U64) => {
                let a = self.compile_expr_i(inner);
                let dst = self.alloc_i();
                self.emit(Op::BitNotI { dst, a }, span);
                dst
            }
            ExprKind::Unary(UnOp::Pos, inner) => self.compile_expr_i(inner),
            ExprKind::Binary(op, lhs, rhs)
                if expr.ty == Ty::U64 && lhs.ty == Ty::U64 && rhs.ty == Ty::U64 =>
            {
                let a = self.compile_expr_i(lhs);
                let b = self.compile_expr_i(rhs);
                let dst = self.alloc_i();
                let inst = match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem | BinOp::Pow => {
                        Op::BinU64 { dst, a, b, op: *op }
                    }
                    BinOp::BitAnd => Op::BitAndU64 { dst, a, b },
                    BinOp::BitOr => Op::BitOrU64 { dst, a, b },
                    BinOp::BitXor => Op::BitXorU64 { dst, a, b },
                    _ => return self.emit_fallback_expr_i(expr),
                };
                self.emit(inst, span);
                dst
            }
            ExprKind::Binary(op, lhs, rhs)
                if expr.ty == Ty::U64
                    && lhs.ty == Ty::U64
                    && matches!(op, BinOp::Shl | BinOp::Shr) =>
            {
                let a = self.compile_expr_i(lhs);
                let b = self.compile_expr_i(rhs);
                let dst = self.alloc_i();
                self.emit(Op::BinU64 { dst, a, b, op: *op }, span);
                dst
            }
            ExprKind::Binary(op, lhs, rhs)
                if matches!(expr.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_))
                    && matches!(lhs.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_))
                    && matches!(rhs.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_)) =>
            {
                let a = self.compile_expr_i(lhs);
                if *op == BinOp::Add {
                    if let ExprKind::Int(v) = rhs.kind {
                        if let Ok(imm) = i32::try_from(v) {
                            let dst = self.alloc_i();
                            self.emit(Op::AddImmI { dst, a, imm }, span);
                            return dst;
                        }
                    }
                }
                if *op == BinOp::Sub {
                    if let ExprKind::Int(v) = rhs.kind {
                        if let Some(neg) = v.checked_neg() {
                            if let Ok(imm) = i32::try_from(neg) {
                                let dst = self.alloc_i();
                                self.emit(Op::AddImmI { dst, a, imm }, span);
                                return dst;
                            }
                        }
                    }
                }
                let b = self.compile_expr_i(rhs);
                let dst = self.alloc_i();
                let inst = match op {
                    BinOp::Add => Op::AddI { dst, a, b },
                    BinOp::Sub => Op::SubI { dst, a, b },
                    BinOp::Mul => Op::MulI { dst, a, b },
                    BinOp::Div => Op::DivI { dst, a, b },
                    BinOp::Rem => Op::RemI { dst, a, b },
                    BinOp::BitAnd => Op::BitAndI { dst, a, b },
                    BinOp::BitOr => Op::BitOrI { dst, a, b },
                    BinOp::BitXor => Op::BitXorI { dst, a, b },
                    BinOp::Shl => Op::ShlI { dst, a, b },
                    BinOp::Shr => Op::ShrI { dst, a, b },
                    _ => return self.emit_fallback_expr_i(expr),
                };
                self.emit(inst, span);
                dst
            }
            ExprKind::Binary(BinOp::And, lhs, rhs) => {
                let dst = self.alloc_i();
                let j_false = self.emit_branch_if_false(lhs);
                let r = self.compile_expr_i(rhs);
                self.emit(Op::MoveI { dst, src: r }, rhs.span);
                let j_end = self.emit(Op::Jmp { target: 0 }, span);
                let false_pc = self.ops.len() as u32;
                self.patch(j_false, false_pc);
                self.emit(Op::ConstImmI { dst, val: 0 }, span);
                let end_pc = self.ops.len() as u32;
                self.patch(j_end, end_pc);
                dst
            }
            ExprKind::Binary(BinOp::Or, lhs, rhs) => {
                let dst = self.alloc_i();
                let l = self.compile_expr_i(lhs);
                let j_true = self.emit(Op::JmpIfNonZero { reg: l, target: 0 }, lhs.span);
                let r = self.compile_expr_i(rhs);
                self.emit(Op::MoveI { dst, src: r }, rhs.span);
                let j_end = self.emit(Op::Jmp { target: 0 }, span);
                let true_pc = self.ops.len() as u32;
                self.patch(j_true, true_pc);
                self.emit(Op::ConstImmI { dst, val: 1 }, span);
                let end_pc = self.ops.len() as u32;
                self.patch(j_end, end_pc);
                dst
            }
            ExprKind::Cmp(op, lhs, rhs)
                if lhs.ty == Ty::U64
                    && rhs.ty == Ty::U64
                    && matches!(
                        op,
                        CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge | CmpOp::Eq | CmpOp::Ne
                    ) =>
            {
                let a = self.compile_expr_i(lhs);
                let b = self.compile_expr_i(rhs);
                let dst = self.alloc_i();
                self.emit(Op::CmpU64 { dst, a, b, op: *op }, span);
                dst
            }
            ExprKind::Cmp(op, lhs, rhs)
                if ScalarKind::of_ty(&lhs.ty, self.program).is_some()
                    && ScalarKind::of_ty(&rhs.ty, self.program).is_some()
                    && lhs.ty != Ty::U64
                    && rhs.ty != Ty::U64
                    && ty_uses_structural_cmp(&lhs.ty, self.program)
                    && matches!(
                        op,
                        CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge | CmpOp::Eq | CmpOp::Ne
                    ) =>
            {
                let a = self.compile_expr_i(lhs);
                let b = self.compile_expr_i(rhs);
                let dst = self.alloc_i();
                self.emit(Op::CmpI { dst, a, b, op: *op }, span);
                dst
            }
            ExprKind::Cmp(op, lhs, rhs)
                if matches!(
                    ScalarKind::of_opt_ty(&lhs.ty, self.program),
                    Some(k) if k != ScalarKind::U64
                )
                    && matches!(
                        ScalarKind::of_opt_ty(&rhs.ty, self.program),
                        Some(k) if k != ScalarKind::U64
                    )
                    && ty_uses_structural_cmp(&lhs.ty, self.program)
                    && matches!(
                        op,
                        CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge | CmpOp::Eq | CmpOp::Ne
                    ) =>
            {
                let (a_val, a_some) = self.compile_expr_opt_i(lhs);
                let (b_val, b_some) = self.compile_expr_opt_i(rhs);
                let dst = self.alloc_i();
                self.emit(Op::CmpOptI { dst, a_val, a_some, b_val, b_some, op: *op }, span);
                dst
            }
            ExprKind::Cmp(op, lhs, rhs)
                if matches!(
                    op,
                    CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge | CmpOp::Eq | CmpOp::Ne
                ) && ty_uses_structural_cmp(&lhs.ty, self.program) =>
            {
                let a_v = self.compile_expr_v(lhs);
                let b_v = self.compile_expr_v(rhs);
                let dst = self.alloc_i();
                self.emit(Op::CmpV { dst, a_v, b_v, op: *op }, span);
                if a_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: a_v }, span);
                }
                if b_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: b_v }, span);
                }
                dst
            }
            ExprKind::Do(block) => {
                let dst = self.alloc_i();
                let prev_pinned = self.pinned_v;
                self.pinned_v = self.next_v;
                self.give_targets.push(GiveTarget::I { dst, exits: Vec::new() });
                self.compile_stmts(&block.stmts);
                let mut target = self.give_targets.pop().unwrap();
                self.pinned_v = prev_pinned;
                let end_pc = self.ops.len() as u32;
                for j in target.exits_mut().drain(..) {
                    self.patch(j, end_pc);
                }
                dst
            }
            ExprKind::Cast(
                Cast::Ord | Cast::Trust | Cast::Raw | Cast::CharToInt | Cast::IntToU64,
                inner,
            ) => self.compile_expr_i(inner),
            ExprKind::Cast(Cast::U64ToInt, inner) => {
                let src = self.compile_expr_i(inner);
                let dst = self.alloc_i();
                self.emit(Op::U64ToInt { dst, src }, span);
                dst
            }
            ExprKind::Cast(Cast::Label(d), inner) => {
                let a = self.compile_expr_i(inner);
                let lo = self.alloc_i();
                self.emit(Op::DomainLo { dst: lo, dom: *d as u16 }, span);
                let dst = self.alloc_i();
                self.emit(Op::AddI { dst, a, b: lo }, span);
                dst
            }
            ExprKind::Cast(Cast::FromLabel(d), inner) => {
                let src = self.compile_expr_i(inner);
                let dst = self.alloc_i();
                self.emit(Op::FromLabel { dst, src, dom: *d as u16 }, span);
                dst
            }
            ExprKind::Cast(Cast::FromOrd(d), inner) => {
                let src = self.compile_expr_i(inner);
                let dst = self.alloc_i();
                self.emit(Op::FromOrd { dst, src, dom: *d as u16 }, span);
                dst
            }
            ExprKind::Builtin(Builtin::BoundStepL(d), args) if args.len() == 2 => {
                let ord = self.compile_expr_i(&args[0]);
                let k = self.compile_expr_i(&args[1]);
                let dst = self.alloc_i();
                self.emit(Op::BoundStepL { dst, ord, k, dom: *d as u16 }, span);
                dst
            }
            ExprKind::Builtin(Builtin::BoundStepR(d), args) if args.len() == 2 => {
                let ord = self.compile_expr_i(&args[0]);
                let k = self.compile_expr_i(&args[1]);
                let dst = self.alloc_i();
                self.emit(Op::BoundStepR { dst, ord, k, dom: *d as u16 }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Min, args)
                if args.len() == 2 && matches!(expr.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_)) =>
            {
                let a = self.compile_expr_i(&args[0]);
                let b = self.compile_expr_i(&args[1]);
                let dst = self.alloc_i();
                self.emit(Op::MinI { dst, a, b }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Max, args)
                if args.len() == 2 && matches!(expr.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_)) =>
            {
                let a = self.compile_expr_i(&args[0]);
                let b = self.compile_expr_i(&args[1]);
                let dst = self.alloc_i();
                self.emit(Op::MaxI { dst, a, b }, span);
                dst
            }
            ExprKind::Builtin(op @ (Builtin::WrapAdd | Builtin::WrapSub | Builtin::WrapMul), args)
                if args.len() == 2 =>
            {
                let a = self.compile_expr_i(&args[0]);
                let b = self.compile_expr_i(&args[1]);
                let dst = self.alloc_i();
                let op = match op {
                    Builtin::WrapAdd => Op::WrapAddI { dst, a, b },
                    Builtin::WrapSub => Op::WrapSubI { dst, a, b },
                    _ => Op::WrapMulI { dst, a, b },
                };
                self.emit(op, span);
                dst
            }
            ExprKind::Builtin(Builtin::Abs, args)
                if args.len() == 1 && matches!(expr.ty, Ty::Int) =>
            {
                let a = self.compile_expr_i(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::AbsI { dst, a }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Len, args) if args.len() == 1 => match &args[0].ty {
                Ty::Array { .. } => {
                    let dst = self.alloc_i();
                    if let ExprKind::Global(gid) = args[0].kind {
                        self.emit(Op::GArrLen { dst, gid: gid as u16 }, span);
                    } else if let ExprKind::Field(obj, fidx) = &args[0].kind {
                        let obj_v = self.compile_expr_v(obj);
                        self.emit(Op::FieldArrLen { dst, obj_v, field: *fidx as u16 }, span);
                    } else {
                        let arr_v = self.compile_expr_v(&args[0]);
                        self.emit(Op::LArrLen { dst, arr_v }, span);
                    }
                    dst
                }
                Ty::List(_) => {
                    let dst = self.alloc_i();
                    if let ExprKind::Global(gid) = args[0].kind {
                        self.emit(Op::GListLen { dst, gid: gid as u16 }, span);
                    } else if let ExprKind::Field(obj, fidx) = &args[0].kind {
                        let obj_v = self.compile_expr_v(obj);
                        self.emit(Op::FieldListLen { dst, obj_v, field: *fidx as u16 }, span);
                    } else {
                        let list_v = self.compile_expr_v(&args[0]);
                        self.emit(Op::LListLen { dst, list_v }, span);
                    }
                    dst
                }
                _ => {
                    let seq_v = self.compile_expr_v(&args[0]);
                    let dst = self.alloc_i();
                    self.emit(Op::SeqLenI { dst, seq_v }, span);
                    dst
                }
            },
            ExprKind::Builtin(Builtin::RangeLo, args) if args.len() == 1 && matches!(expr.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_)) => {
                let src_v = self.compile_expr_v(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::RangeLoI { dst, src_v }, span);
                dst
            }
            ExprKind::Builtin(Builtin::RangeHi, args) if args.len() == 1 && matches!(expr.ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_)) => {
                let src_v = self.compile_expr_v(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::RangeHiI { dst, src_v }, span);
                dst
            }
            ExprKind::Builtin(Builtin::BitCeil, args) if args.len() == 1 && matches!(expr.ty, Ty::Int) => {
                let src = self.compile_expr_i(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::BitCeilI { dst, src }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Popcount, args)
                if args.len() == 1 && ScalarKind::of_ty(&args[0].ty, self.program).is_some() =>
            {
                let src = self.compile_expr_i(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::PopcountI { dst, src }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Ctz, args)
                if args.len() == 1 && ScalarKind::of_ty(&args[0].ty, self.program).is_some() =>
            {
                let src = self.compile_expr_i(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::CtzI { dst, src }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Clz, args)
                if args.len() == 1 && ScalarKind::of_ty(&args[0].ty, self.program).is_some() =>
            {
                let src = self.compile_expr_i(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::ClzI { dst, src }, span);
                dst
            }
            ExprKind::Builtin(Builtin::BitWidth, args)
                if args.len() == 1 && ScalarKind::of_ty(&args[0].ty, self.program).is_some() =>
            {
                let src = self.compile_expr_i(&args[0]);
                let dst = self.alloc_i();
                self.emit(Op::BitWidthI { dst, src }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Contains, args)
                if args.len() == 2
                    && matches!(
                        args[0].ty,
                        Ty::Set(_) | Ty::Map(_, _) | Ty::List(_) | Ty::Array { .. }
                    )
                    && !crate::value::ty_has_cached(&args[1].ty, self.program) =>
            {
                let src_v = self.compile_expr_v(&args[0]);
                let key_v = self.compile_expr_v(&args[1]);
                let dst = self.alloc_i();
                self.emit(Op::ContainsV { dst, src_v, key_v }, span);
                if key_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: key_v }, span);
                }
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                dst
            }
            ExprKind::Mutate(m @ (Mutation::Insert | Mutation::Remove), place, args)
                if args.len() == 1
                    && !crate::value::ty_has_cached(&args[0].ty, self.program)
                    && ((matches!(m, Mutation::Insert) && matches!(place.ty, Ty::Set(_)))
                        || (matches!(m, Mutation::Remove)
                            && matches!(place.ty, Ty::Set(_) | Ty::Map(_, _)))) =>
            {
                let saved_ops = self.ops.len();
                let saved_spans = self.spans.len();
                let saved_places = self.places.len();
                if let Some(path) = self.compile_place_path(place) {
                    let val_v = self.compile_expr_v(&args[0]);
                    let dst = self.alloc_i();
                    self.emit(
                        Op::SetMapMutPlace {
                            dst,
                            path,
                            val_v,
                            is_remove: matches!(m, Mutation::Remove),
                        },
                        span,
                    );
                    if val_v >= self.num_local_v {
                        self.emit(Op::ClearV { reg: val_v }, span);
                    }
                    dst
                } else {
                    self.ops.truncate(saved_ops);
                    self.spans.truncate(saved_spans);
                    self.places.truncate(saved_places);
                    self.emit_fallback_expr_i(expr)
                }
            }
            ExprKind::Index(base, indices)
                if matches!(&base.ty, Ty::Map(kty, _) if !crate::value::ty_has_cached(kty, self.program))
                    && indices.len() == 1 =>
            {
                let Ty::Map(kty, _) = &base.ty else { unreachable!() };
                let map_v = self.compile_expr_v(base);
                let key_v = self.compile_expr_v(&indices[0]);
                let key_ty = self.fill_tys.len() as u16;
                self.fill_tys.push(kty);
                let tmp_v = self.alloc_v();
                self.emit(Op::MapIndexV { dst: tmp_v, map_v, key_v, key_ty }, span);
                if key_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: key_v }, span);
                }
                if map_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: map_v }, span);
                }
                let dst = self.alloc_i();
                self.emit(Op::UnboxI { dst_i: dst, src_v: tmp_v }, span);
                self.emit(Op::ClearV { reg: tmp_v }, span);
                dst
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    ExprKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        if indices.len() == 2 {
                            let r_j = self.compile_expr_i(&indices[1]);
                            self.emit(
                                Op::LoadGArr2I { dst, gid: *gid as u16, r_i, r_j },
                                span,
                            );
                        } else {
                            self.emit(Op::LoadGArr1I { dst, gid: *gid as u16, r_i }, span);
                        }
                        dst
                    }
                    ExprKind::Local(slot) => {
                        if let SlotLoc::V(arr_v) = self.slot_loc(*slot) {
                            let r_i = self.compile_expr_i(&indices[0]);
                            let dst = self.alloc_i();
                            if indices.len() == 2 {
                                let r_j = self.compile_expr_i(&indices[1]);
                                self.emit(Op::LoadLArr2I { dst, arr_v, r_i, r_j }, span);
                            } else {
                                self.emit(Op::LoadLArr1I { dst, arr_v, r_i }, span);
                            }
                            dst
                        } else {
                            self.emit_fallback_expr_i(expr)
                        }
                    }
                    ExprKind::Index(outer, outer_indices)
                        if indices.len() == 1
                            && outer_indices.len() == 1
                            && matches!(outer.ty, Ty::Array { .. } | Ty::List(_)) =>
                    {
                        let root_info = match &outer.kind {
                            ExprKind::Global(gid) => Some((*gid as u16, true)),
                            ExprKind::Local(slot) => match self.slot_loc(*slot) {
                                SlotLoc::V(v) => Some((v, false)),
                                _ => None,
                            },
                            _ => None,
                        };
                        if let Some((root, is_global)) = root_info {
                            let r_i = self.compile_expr_i(&outer_indices[0]);
                            let r_j = self.compile_expr_i(&indices[0]);
                            let dst = self.alloc_i();
                            self.emit(
                                Op::LoadNestedArrI { dst, root, is_global, r_i, r_j },
                                span,
                            );
                            return dst;
                        }
                        let arr_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        self.emit(Op::LoadLArr1I { dst, arr_v, r_i }, span);
                        if arr_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: arr_v }, span);
                        }
                        dst
                    }
                    ExprKind::Field(obj, fidx) if indices.len() == 1 => {
                        let obj_v = self.compile_expr_v(obj);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        self.emit(
                            Op::LoadFieldArr1I { dst, obj_v, field: *fidx as u16, r_i },
                            span,
                        );
                        if obj_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: obj_v }, span);
                        }
                        dst
                    }
                    _ => {
                        let arr_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        if indices.len() == 2 {
                            let r_j = self.compile_expr_i(&indices[1]);
                            self.emit(Op::LoadLArr2I { dst, arr_v, r_i, r_j }, span);
                        } else {
                            self.emit(Op::LoadLArr1I { dst, arr_v, r_i }, span);
                        }
                        if arr_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: arr_v }, span);
                        }
                        dst
                    }
                }
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _))
                    && indices.len() == 1 =>
            {
                match &base.kind {
                    ExprKind::Global(gid) if matches!(base.ty, Ty::List(_)) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        self.emit(Op::LoadGList1I { dst, gid: *gid as u16, r_i }, span);
                        dst
                    }
                    ExprKind::Local(slot) => {
                        if let SlotLoc::V(list_v) = self.slot_loc(*slot) {
                            let r_i = self.compile_expr_i(&indices[0]);
                            let dst = self.alloc_i();
                            self.emit(Op::LoadLList1I { dst, list_v, r_i }, span);
                            dst
                        } else {
                            self.emit_fallback_expr_i(expr)
                        }
                    }
                    ExprKind::Field(obj, fidx) if matches!(base.ty, Ty::List(_)) => {
                        let obj_v = self.compile_expr_v(obj);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        self.emit(
                            Op::LoadFieldList1I { dst, obj_v, field: *fidx as u16, r_i },
                            span,
                        );
                        dst
                    }
                    _ => {
                        let list_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        self.emit(Op::LoadLList1I { dst, list_v, r_i }, span);
                        if list_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: list_v }, span);
                        }
                        dst
                    }
                }
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::Str | Ty::StrView) && indices.len() == 1 =>
            {
                let r_i = self.compile_expr_i(&indices[0]);
                let dst = self.alloc_i();
                match (&base.kind, &base.ty) {
                    (ExprKind::Global(gid), Ty::Str) => {
                        self.emit(Op::GStrGetI { dst, gid: *gid as u16, r_i }, span);
                    }
                    _ => {
                        let str_v = self.compile_expr_v(base);
                        self.emit(Op::StrGetI { dst, str_v, r_i }, span);
                        if str_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: str_v }, span);
                        }
                    }
                }
                dst
            }
            ExprKind::Mutate(Mutation::Take, place, args) if args.is_empty() => {
                match &place.kind {
                    PlaceKind::Local(slot) => {
                        if let SlotLoc::I(src, _) = self.slot_loc(*slot) {
                            let dst = self.alloc_i();
                            self.emit(Op::MoveI { dst, src }, span);
                            return dst;
                        }
                    }
                    PlaceKind::Global(gid) => {
                        let dst = self.alloc_i();
                        self.emit(Op::LoadGlobalI { dst, gid: *gid as u16 }, span);
                        return dst;
                    }
                    _ => {}
                }
                self.emit_fallback_expr_i(expr)
            }
            ExprKind::Binary(BinOp::Coalesce, lhs, rhs)
                if ScalarKind::of_opt_ty(&lhs.ty, self.program).is_some() =>
            {
                let (lv, ls) = self.compile_expr_opt_i(lhs);
                let dst = self.alloc_i();
                self.emit(Op::MoveI { dst, src: lv }, span);
                let j_done = self.emit(Op::JmpIfNonZero { reg: ls, target: 0 }, span);
                let rv = self.compile_expr_i(rhs);
                if rv != dst {
                    self.emit(Op::MoveI { dst, src: rv }, span);
                }
                let end = self.ops.len() as u32;
                self.patch(j_done, end);
                dst
            }
            ExprKind::If(cond, then_e, else_e) => {
                if let Some(b) = Self::const_bool(cond) {
                    return if b {
                        self.compile_expr_i(then_e)
                    } else {
                        self.compile_expr_i(else_e)
                    };
                }
                let dst = self.alloc_i();
                let j_else = self.emit_branch_if_false(cond);
                let rt = self.compile_expr_i(then_e);
                if rt != dst {
                    self.emit(Op::MoveI { dst, src: rt }, then_e.span);
                }
                let j_end = self.emit(Op::Jmp { target: 0 }, span);
                let else_pc = self.ops.len() as u32;
                self.patch(j_else, else_pc);
                let re = self.compile_expr_i(else_e);
                if re != dst {
                    self.emit(Op::MoveI { dst, src: re }, else_e.span);
                }
                let end_pc = self.ops.len() as u32;
                self.patch(j_end, end_pc);
                dst
            }
            ExprKind::Call(fn_id, args) => {
                if let Some(mop) = Self::recognize_mod_op(self.program, *fn_id) {
                    if args.iter().all(|a| matches!(a, Arg::Value(_))) {
                        let dst = self.alloc_i();
                        match (mop, args.as_slice()) {
                            (ModSuperOp::Add(modulus), [Arg::Value(a), Arg::Value(b)]) => {
                                let ra = self.compile_expr_i(a);
                                let rb = self.compile_expr_i(b);
                                self.emit(Op::ModAddI { dst, a: ra, b: rb, modulus }, span);
                                return dst;
                            }
                            (ModSuperOp::Sub(modulus), [Arg::Value(a), Arg::Value(b)]) => {
                                let ra = self.compile_expr_i(a);
                                let rb = self.compile_expr_i(b);
                                self.emit(Op::ModSubI { dst, a: ra, b: rb, modulus }, span);
                                return dst;
                            }
                            (ModSuperOp::Mul(modulus), [Arg::Value(a), Arg::Value(b)]) => {
                                let ra = self.compile_expr_i(a);
                                let rb = self.compile_expr_i(b);
                                self.emit(Op::ModMulI { dst, a: ra, b: rb, modulus }, span);
                                return dst;
                            }
                            (ModSuperOp::Neg(modulus), [Arg::Value(a)]) => {
                                let ra = self.compile_expr_i(a);
                                self.emit(Op::ModNegI { dst, a: ra, modulus }, span);
                                return dst;
                            }
                            (ModSuperOp::New(modulus), [Arg::Value(a)]) => {
                                if let ExprKind::Int(v) = a.kind {
                                    self.emit_const_i(dst, v.rem_euclid(modulus as i64), span);
                                    return dst;
                                }
                                let ra = self.compile_expr_i(a);
                                let rm = self.alloc_i();
                                self.emit_const_i(rm, modulus as i64, span);
                                self.emit(Op::RemI { dst, a: ra, b: rm }, span);
                                return dst;
                            }
                            _ => {}
                        }
                    }
                }
                let can_inline_pure_leaf = self.is_pure_leaf_fn(*fn_id) && {
                    let f = &self.program.fns[*fn_id];
                    args.iter().enumerate().all(|(i, a)| {
                        let Arg::Value(ae) = a else { return false; };
                        if ScalarKind::of_ty(&f.locals[i].ty, self.program).is_some() {
                            true
                        } else if let ExprKind::Local(slot) = &ae.kind {
                            matches!(self.slot_loc(*slot), SlotLoc::V(r) if r < self.num_local_v)
                        } else {
                            false
                        }
                    })
                };
                if can_inline_pure_leaf {
                    let f = &self.program.fns[*fn_id];
                    let mut map = Vec::with_capacity(f.locals.len());
                    for (i, a) in args.iter().enumerate() {
                        let Arg::Value(ae) = a else { unreachable!() };
                        if let Some(kind) = ScalarKind::of_ty(&f.locals[i].ty, self.program) {
                            let src = self.compile_expr_i(ae);
                            let reg = self.alloc_i();
                            self.emit(Op::MoveI { dst: reg, src }, span);
                            map.push(SlotLoc::I(reg, kind));
                        } else if let ExprKind::Local(slot) = &ae.kind {
                            let SlotLoc::V(r) = self.slot_loc(*slot) else { unreachable!() };
                            map.push(SlotLoc::V(r));
                        } else {
                            unreachable!()
                        }
                    }
                    for l in f.locals.iter().skip(args.len()) {
                        let kind = ScalarKind::of_ty(&l.ty, self.program).unwrap();
                        let reg = self.alloc_i();
                        self.emit(Op::ConstImmI { dst: reg, val: 0 }, span);
                        map.push(SlotLoc::I(reg, kind));
                    }
                    let dst = self.alloc_i();
                    let prev_slots = self.inline_slots.replace(map);
                    let prev_ret = self.inline_ret.replace((dst, Vec::new()));
                    for s in &f.body.stmts {
                        self.compile_stmt(s);
                    }
                    let end_pc = self.ops.len() as u32;
                    let (_, jumps) = self.inline_ret.take().unwrap();
                    self.inline_ret = prev_ret;
                    self.inline_slots = prev_slots;
                    for j in jumps {
                        self.patch(j, end_pc);
                    }
                    dst
                } else {
                    let f = &self.program.fns[*fn_id];
                    let can_vm = !self.program.judge_fns.contains(fn_id)
                        && !f.params.iter().any(|p| p.mutable)
                        && f.locals.len() <= 512
                        && self.program.fn_guarded_domains(&f.ret, &f.body).is_empty()
                        && (0..f.params.len()).all(|i| {
                            ScalarKind::of_ty(&f.locals[i].ty, self.program).is_some()
                        })
                        && ScalarKind::of_ty(&f.ret, self.program).is_some()
                        && args.len() <= 8
                        && args.iter().all(|a| matches!(a, Arg::Value(_)));
                    if can_vm {
                        let mut evaluated = Vec::with_capacity(args.len());
                        for a in args {
                            let Arg::Value(ae) = a else { unreachable!() };
                            evaluated.push(self.compile_expr_i(ae));
                        }
                        let arg_base = self.next_i;
                        for r in evaluated {
                            let slot = self.alloc_i();
                            if slot != r {
                                self.emit(Op::MoveI { dst: slot, src: r }, span);
                            }
                        }
                        let dst = self.alloc_i();
                        self.emit(
                            Op::CallVmI {
                                fn_id: *fn_id as u16,
                                arg_base,
                                argc: args.len() as u8,
                                dst,
                            },
                            span,
                        );
                        dst
                    } else if let Some((dst, _, _)) =
                        self.try_compile_call_mix(*fn_id, args, CallRet::I, span)
                    {
                        dst
                    } else {
                        self.emit_fallback_expr_i(expr)
                    }
                }
            }
            ExprKind::CallExpr(callee, args) => {
                if let Some((dst, _, _)) =
                    self.try_compile_call_closure(callee, args, CallRet::I, span)
                {
                    dst
                } else {
                    self.emit_fallback_expr_i(expr)
                }
            }
            _ => self.emit_fallback_expr_i(expr),
        }
    }

    fn compile_expr_v(&mut self, expr: &'p Expr) -> u16 {
        let span = expr.span;
        if let Some(kind) = ScalarKind::of_ty(&expr.ty, self.program) {
            let src_i = self.compile_expr_i(expr);
            let dst_v = self.alloc_v();
            self.emit(Op::BoxI { dst_v, src_i, kind }, span);
            return dst_v;
        }
        if let Some(kind) = ScalarKind::of_opt_ty(&expr.ty, self.program) {
            let (val, some) = self.compile_expr_opt_i(expr);
            let dst_v = self.alloc_v();
            self.emit(Op::BoxOptI { dst_v, val, some, kind }, span);
            return dst_v;
        }
        match &expr.kind {
            ExprKind::Format(pieces) => self.compile_format(pieces, span),
            ExprKind::Str(bytes) => {
                let idx = self.consts_v.len() as u32;
                self.consts_v.push(Value::Str(Rc::new(bytes.clone())));
                let dst = self.alloc_v();
                self.emit(Op::ConstV { dst, idx }, span);
                dst
            }
            ExprKind::Local(slot) => match self.slot_loc(*slot) {
                SlotLoc::V(r) => r,
                SlotLoc::I(src_i, kind) => {
                    let dst_v = self.alloc_v();
                    self.emit(Op::BoxI { dst_v, src_i, kind }, span);
                    dst_v
                }
                SlotLoc::OptI { val, some, kind } => {
                    let dst_v = self.alloc_v();
                    self.emit(Op::BoxOptI { dst_v, val, some, kind }, span);
                    dst_v
                }
                SlotLoc::SmallTuple { base, len, kinds } => {
                    let dst_v = self.alloc_v();
                    self.emit(Op::BoxSmallTuple { dst_v, src_base: base, len, kinds }, span);
                    dst_v
                }
            },
            ExprKind::Global(gid) => {
                let dst = self.alloc_v();
                self.emit(Op::LoadGlobalV { dst, gid: *gid as u16 }, span);
                dst
            }
            ExprKind::Field(base, idx) => {
                let obj_v = self.compile_expr_v(base);
                let dst = self.alloc_v();
                self.emit(Op::FieldV { dst, obj_v, field: *idx as u16 }, span);
                if obj_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: obj_v }, span);
                }
                dst
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    ExprKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let dst = self.alloc_v();
                        if let Some(r_j) = r_j {
                            self.emit(
                                Op::LoadGArr2V { dst, gid: *gid as u16, r_i, r_j },
                                span,
                            );
                        } else {
                            self.emit(Op::LoadGArr1V { dst, gid: *gid as u16, r_i }, span);
                        }
                        dst
                    }
                    ExprKind::Local(slot) => {
                        if let SlotLoc::V(arr_v) = self.slot_loc(*slot) {
                            let r_i = self.compile_expr_i(&indices[0]);
                            let r_j =
                                (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                            let dst = self.alloc_v();
                            if let Some(r_j) = r_j {
                                self.emit(Op::LoadLArr2V { dst, arr_v, r_i, r_j }, span);
                            } else {
                                self.emit(Op::LoadLArr1V { dst, arr_v, r_i }, span);
                            }
                            dst
                        } else {
                            self.emit_fallback_expr_v(expr)
                        }
                    }
                    _ => {
                        let arr_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_j = (indices.len() == 2).then(|| self.compile_expr_i(&indices[1]));
                        let dst = self.alloc_v();
                        if let Some(r_j) = r_j {
                            self.emit(Op::LoadLArr2V { dst, arr_v, r_i, r_j }, span);
                        } else {
                            self.emit(Op::LoadLArr1V { dst, arr_v, r_i }, span);
                        }
                        if arr_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: arr_v }, span);
                        }
                        dst
                    }
                }
            }
            ExprKind::Float(f) => {
                let idx = self.consts_i.len() as u16;
                self.consts_i.push(f.to_bits() as i64);
                let dst = self.alloc_v();
                self.emit(Op::ConstF64 { dst, idx }, span);
                dst
            }
            ExprKind::Binary(op, a, b)
                if matches!(a.ty, Ty::Float)
                    && matches!(b.ty, Ty::Float)
                    && matches!(
                        op,
                        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Pow
                    ) =>
            {
                let ra = self.compile_expr_v(a);
                let rb = self.compile_expr_v(b);
                let dst = self.alloc_v();
                self.emit(Op::FloatBin { dst, a: ra, b: rb, op: *op }, span);
                dst
            }
            ExprKind::Builtin(b @ (Builtin::Min | Builtin::Max), args)
                if args.len() == 2 && matches!(expr.ty, Ty::Float) =>
            {
                let ra = self.compile_expr_v(&args[0]);
                let rb = self.compile_expr_v(&args[1]);
                let dst = self.alloc_v();
                self.emit(
                    Op::FloatMinMax { dst, a: ra, b: rb, is_max: matches!(b, Builtin::Max) },
                    span,
                );
                dst
            }
            ExprKind::If(cond, then_e, else_e) => {
                let dst = self.alloc_v();
                let rc = self.compile_expr_i(cond);
                let j_else = self.emit(Op::JmpIfZero { reg: rc, target: 0 }, span);
                let rt = self.compile_expr_v(then_e);
                if rt != dst {
                    if rt >= self.num_local_v {
                        self.emit(Op::MoveVConsume { dst, src: rt }, span);
                    } else {
                        self.emit(Op::MoveV { dst, src: rt }, span);
                    }
                }
                let j_end = self.emit(Op::Jmp { target: 0 }, span);
                let else_pc = self.ops.len() as u32;
                self.patch(j_else, else_pc);
                let re = self.compile_expr_v(else_e);
                if re != dst {
                    if re >= self.num_local_v {
                        self.emit(Op::MoveVConsume { dst, src: re }, span);
                    } else {
                        self.emit(Op::MoveV { dst, src: re }, span);
                    }
                }
                let end_pc = self.ops.len() as u32;
                self.patch(j_end, end_pc);
                dst
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _))
                    && indices.len() == 1 =>
            {
                match &base.kind {
                    ExprKind::Global(gid) if matches!(base.ty, Ty::List(_)) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_v();
                        self.emit(Op::LoadGList1V { dst, gid: *gid as u16, r_i }, span);
                        dst
                    }
                    ExprKind::Local(slot) => {
                        if let SlotLoc::V(list_v) = self.slot_loc(*slot) {
                            let r_i = self.compile_expr_i(&indices[0]);
                            let dst = self.alloc_v();
                            self.emit(Op::LoadLList1V { dst, list_v, r_i }, span);
                            dst
                        } else {
                            self.emit_fallback_expr_v(expr)
                        }
                    }
                    _ => {
                        let list_v = self.compile_expr_v(base);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_v();
                        self.emit(Op::LoadLList1V { dst, list_v, r_i }, span);
                        if list_v >= self.num_local_v {
                            self.emit(Op::ClearV { reg: list_v }, span);
                        }
                        dst
                    }
                }
            }
            ExprKind::Tuple(fields)
                if fields.len() <= 16
                    && matches!(expr.ty, Ty::Tuple(_) | Ty::Record(_) | Ty::Struct(_)) => {
                let mut regs = Vec::with_capacity(fields.len());
                for f in fields {
                    regs.push(self.compile_expr_v(f));
                }
                let base_v = self.next_v;
                for r in regs {
                    let slot = self.alloc_v();
                    if slot != r {
                        self.emit(Op::MoveV { dst: slot, src: r }, span);
                    }
                }
                let dst = self.alloc_v();
                self.emit(Op::MakeTuple { dst, base_v, len: fields.len() as u8 }, span);
                dst
            }
            ExprKind::Tuple(fields) if fields.len() <= 16 && matches!(expr.ty, Ty::List(_)) => {
                let mut regs = Vec::with_capacity(fields.len());
                for f in fields {
                    regs.push(self.compile_expr_v(f));
                }
                let base_v = self.next_v;
                for r in regs {
                    let slot = self.alloc_v();
                    if slot != r {
                        self.emit(Op::MoveV { dst: slot, src: r }, span);
                    }
                }
                let dst = self.alloc_v();
                self.emit(Op::MakeList { dst, base_v, len: fields.len() as u8 }, span);
                dst
            }
            ExprKind::Builtin(Builtin::ListFilled, args) if args.len() == 2 => {
                let r_n = self.compile_expr_i(&args[0]);
                let val_v = self.compile_expr_v(&args[1]);
                let dst = self.alloc_v();
                self.emit(Op::ListFilled { dst, r_n, val_v }, span);
                if val_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: val_v }, span);
                }
                dst
            }
            ExprKind::Builtin(Builtin::ArrayToList | Builtin::ToList, args) if args.len() == 1 => {
                let src_v = self.compile_expr_v(&args[0]);
                let dst = self.alloc_v();
                self.emit(Op::ToList { dst, src_v }, span);
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                dst
            }
            ExprKind::Builtin(Builtin::ListSlice, args) if args.len() == 3 => {
                let src_v = self.compile_expr_v(&args[0]);
                let r_lo = self.compile_expr_i(&args[1]);
                let r_hi = self.compile_expr_i(&args[2]);
                let dst = self.alloc_v();
                self.emit(Op::ListSlice { dst, src_v, r_lo, r_hi }, span);
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                dst
            }
            ExprKind::Builtin(Builtin::StrSlice, args) if args.len() == 3 => {
                let src_v = self.compile_expr_v(&args[0]);
                let r_lo = self.compile_expr_i(&args[1]);
                let r_hi = self.compile_expr_i(&args[2]);
                let dst = self.alloc_v();
                self.emit(Op::StrSlice { dst, src_v, r_lo, r_hi }, span);
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                dst
            }
            ExprKind::Fill(dims, value) if dims.len() <= 4 => {
                let mut dim_regs: Vec<u16> = Vec::with_capacity(dims.len());
                for dim in dims {
                    let r = match dim {
                        cpl_ir::DimSpec::Pos(d) => {
                            let r = self.alloc_i();
                            self.emit(Op::DomainLen { dst: r, dom: *d as u16 }, span);
                            r
                        }
                        cpl_ir::DimSpec::Bound(d) => {
                            let r = self.alloc_i();
                            self.emit(Op::DomainLen { dst: r, dom: *d as u16 }, span);
                            let one = self.alloc_i();
                            self.emit_const_i(one, 1, span);
                            let out = self.alloc_i();
                            self.emit(Op::AddI { dst: out, a: r, b: one }, span);
                            out
                        }
                        cpl_ir::DimSpec::Sym(d) => {
                            let n = self.alloc_i();
                            self.emit(Op::DomainLen { dst: n, dom: *d as u16 }, span);
                            let one = self.alloc_i();
                            self.emit_const_i(one, 1, span);
                            let np1 = self.alloc_i();
                            self.emit(Op::AddI { dst: np1, a: n, b: one }, span);
                            let prod = self.alloc_i();
                            self.emit(Op::MulI { dst: prod, a: n, b: np1 }, span);
                            let two = self.alloc_i();
                            self.emit_const_i(two, 2, span);
                            let out = self.alloc_i();
                            self.emit(Op::DivI { dst: out, a: prod, b: two }, span);
                            out
                        }
                        cpl_ir::DimSpec::Enum(e) => {
                            let r = self.alloc_i();
                            self.emit_const_i(
                                r,
                                self.program.enums[*e].variants.len() as i64,
                                span,
                            );
                            r
                        }
                        cpl_ir::DimSpec::Int(len) | cpl_ir::DimSpec::IntFrom(_, len) => {
                            self.compile_expr_i(len)
                        }
                    };
                    dim_regs.push(r);
                }
                let dims_base = self.next_i;
                for r in dim_regs {
                    let slot = self.alloc_i();
                    if slot != r {
                        self.emit(Op::MoveI { dst: slot, src: r }, span);
                    }
                }
                let (fill_v, fill_ty) = match value {
                    Some(v) => (self.compile_expr_v(v), u16::MAX),
                    None => match &expr.ty {
                        Ty::Array { elem, .. } => {
                            let idx = self.fill_tys.len() as u16;
                            self.fill_tys.push(elem.as_ref());
                            (u16::MAX, idx)
                        }
                        _ => (u16::MAX, u16::MAX),
                    },
                };
                let dst = self.alloc_v();
                self.emit(
                    Op::MakeFill {
                        dst,
                        dims_base,
                        ndims: dims.len() as u8,
                        fill_v,
                        fill_ty,
                    },
                    span,
                );
                if fill_v != u16::MAX && fill_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: fill_v }, span);
                }
                dst
            }
            ExprKind::Builtin(Builtin::Copy, args) if args.len() == 1 => {
                self.compile_expr_v(&args[0])
            }
            ExprKind::None => {
                let dst_v = self.alloc_v();
                self.emit(Op::ConstNoneV { dst_v }, span);
                dst_v
            }
            ExprKind::Some(inner) => {
                let src_v = self.compile_expr_v(inner);
                let dst_v = self.alloc_v();
                self.emit(Op::WrapSomeV { dst_v, src_v }, span);
                dst_v
            }
            ExprKind::Unwrap(inner, _) => {
                let src_v = self.compile_expr_v(inner);
                let dst_v = self.alloc_v();
                self.emit(Op::UnwrapV { dst_v, src_v }, span);
                dst_v
            }
            ExprKind::Binary(BinOp::Coalesce, lhs, rhs) => {
                let lv = self.compile_expr_v(lhs);
                let is_some = self.alloc_i();
                self.emit(Op::IsSomeV { dst_i: is_some, src_v: lv }, lhs.span);
                let dst = self.alloc_v();
                let j_none = self.emit(Op::JmpIfZero { reg: is_some, target: 0 }, span);
                self.emit(Op::UnwrapV { dst_v: dst, src_v: lv }, lhs.span);
                let j_done = self.emit(Op::Jmp { target: 0 }, span);
                let none_pc = self.ops.len() as u32;
                self.patch(j_none, none_pc);
                let rv = self.compile_expr_v(rhs);
                if rv != dst {
                    self.emit(Op::MoveV { dst, src: rv }, rhs.span);
                }
                let end_pc = self.ops.len() as u32;
                self.patch(j_done, end_pc);
                dst
            }
            ExprKind::Builtin(Builtin::Last, args) if args.len() == 1 => {
                let seq_v = self.compile_expr_v(&args[0]);
                let dst_v = self.alloc_v();
                self.emit(Op::SeqLastV { dst_v, seq_v }, span);
                dst_v
            }
            ExprKind::Builtin(Builtin::First, args) if args.len() == 1 => {
                let seq_v = self.compile_expr_v(&args[0]);
                let dst_v = self.alloc_v();
                self.emit(Op::SeqFirstV { dst_v, seq_v }, span);
                dst_v
            }
            ExprKind::Mutate(Mutation::Pop, place, args)
                if args.is_empty() && matches!(place.ty, Ty::List(_)) =>
            match &place.kind {
                PlaceKind::Local(slot) => {
                    if let SlotLoc::V(list_v) = self.slot_loc(*slot) {
                        let dst_v = self.alloc_v();
                        self.emit(Op::ListPopLV { dst_v, list_v }, span);
                        dst_v
                    } else {
                        self.emit_fallback_expr_v(expr)
                    }
                }
                PlaceKind::Global(gid) => {
                    let dst_v = self.alloc_v();
                    self.emit(Op::ListPopGV { dst_v, gid: *gid as u16 }, span);
                    dst_v
                }
                PlaceKind::Field(obj_place, field) => {
                    if let PlaceKind::Local(slot) = obj_place.kind {
                        if let SlotLoc::V(obj_v) = self.slot_loc(slot) {
                            let dst_v = self.alloc_v();
                            self.emit(
                                Op::ListPopFieldLV { dst_v, obj_v, field: *field as u16 },
                                span,
                            );
                            return dst_v;
                        }
                    }
                    self.emit_fallback_expr_v(expr)
                }
                _ => self.emit_fallback_expr_v(expr),
            },
            ExprKind::Mutate(Mutation::Take, place, args) if args.is_empty() => {
                if let PlaceKind::Local(slot) = place.kind {
                    if let SlotLoc::V(src) = self.slot_loc(slot) {
                        let dst = self.alloc_v();
                        self.emit(Op::TakeLocalV { dst, src }, span);
                        return dst;
                    }
                }
                if let Some(path) = self.compile_place_path(place) {
                    let dst = self.alloc_v();
                    self.emit(Op::TakePlaceV { dst, path }, span);
                    return dst;
                }
                self.emit_fallback_expr_v(expr)
            }
            ExprKind::Closure(cid) => {
                let def = &self.program.closures[*cid];
                if def.captures.iter().all(|c| c.is_global || !c.by_ref) {
                    let mut caps = Vec::with_capacity(def.captures.len());
                    for cap in &def.captures {
                        if cap.is_global {
                            if cap.by_ref {
                                caps.push(CapSrc::GlobalRef(cap.outer as u16));
                            } else {
                                caps.push(CapSrc::GlobalVal(cap.outer as u16));
                            }
                        } else {
                            match self.slot_loc(cap.outer) {
                                SlotLoc::I(r, kind) => caps.push(CapSrc::LocalI(r, kind)),
                                SlotLoc::OptI { val, some, kind } => {
                                    caps.push(CapSrc::LocalOptI { val, some, kind });
                                }
                                SlotLoc::SmallTuple { base, len, kinds } => {
                                    caps.push(CapSrc::LocalSmallTuple { base, len, kinds });
                                }
                                SlotLoc::V(r) => caps.push(CapSrc::LocalV(r)),
                            }
                        }
                    }
                    let spec = self.closure_specs.len() as u16;
                    self.closure_specs.push(ClosureMakeSpec { cid: *cid as u32, caps });
                    let dst = self.alloc_v();
                    self.emit(Op::MakeClosure { dst, spec }, span);
                    dst
                } else {
                    self.emit_fallback_expr_v(expr)
                }
            }
            ExprKind::Range { lo, hi }
                if ScalarKind::of_ty(&lo.ty, self.program).is_some()
                    && ScalarKind::of_ty(&hi.ty, self.program).is_some() =>
            {
                let r_lo = self.compile_expr_i(lo);
                let r_hi = self.compile_expr_i(hi);
                let dst = self.alloc_v();
                self.emit(Op::MakeRange { dst, r_lo, r_hi }, span);
                dst
            }
            ExprKind::Do(block) => {
                let dst = self.alloc_v();
                let prev_pinned = self.pinned_v;
                self.pinned_v = self.next_v;
                self.give_targets.push(GiveTarget::V { dst, exits: Vec::new() });
                self.compile_stmts(&block.stmts);
                let mut target = self.give_targets.pop().unwrap();
                self.pinned_v = prev_pinned;
                let end_pc = self.ops.len() as u32;
                for j in target.exits_mut().drain(..) {
                    self.patch(j, end_pc);
                }
                dst
            }
            ExprKind::Call(fn_id, args) => {
                if let Some((_, dst_v, _)) =
                    self.try_compile_call_mix(*fn_id, args, CallRet::V, span)
                {
                    dst_v
                } else {
                    self.emit_fallback_expr_v(expr)
                }
            }
            ExprKind::Cast(Cast::IntToFloat, inner) => {
                let src = self.compile_expr_i(inner);
                let dst = self.alloc_v();
                self.emit(Op::IntToFloat { dst, src }, span);
                dst
            }
            ExprKind::Builtin(Builtin::Sqrt, args) if args.len() == 1 => {
                let src = self.compile_expr_v(&args[0]);
                let dst = self.alloc_v();
                self.emit(Op::FloatSqrt { dst, src }, span);
                if src >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src }, span);
                }
                dst
            }
            ExprKind::Unit => {
                let dst = self.alloc_v();
                self.emit(Op::ConstUnit { dst }, span);
                dst
            }
            ExprKind::Builtin(b @ (Builtin::SetMin | Builtin::SetMax), args) if args.len() == 1 => {
                let src_v = self.compile_expr_v(&args[0]);
                let dst = self.alloc_v();
                self.emit(
                    Op::SetMinMaxV { dst, src_v, is_max: matches!(b, Builtin::SetMax) },
                    span,
                );
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                dst
            }
            ExprKind::Builtin(Builtin::MapGet, args)
                if args.len() == 2
                    && matches!(&args[0].ty, Ty::Map(kty, _) if !crate::value::ty_has_cached(kty, self.program)) =>
            {
                let map_v = self.compile_expr_v(&args[0]);
                let key_v = self.compile_expr_v(&args[1]);
                let dst = self.alloc_v();
                self.emit(Op::MapGetV { dst, map_v, key_v }, span);
                if key_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: key_v }, span);
                }
                if map_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: map_v }, span);
                }
                dst
            }
            ExprKind::Builtin(
                b @ (Builtin::FirstGe | Builtin::FirstGt | Builtin::LastLe | Builtin::LastLt),
                args,
            ) if args.len() == 2
                && matches!(args[0].ty, Ty::Set(_) | Ty::Map(_, _))
                && !crate::value::ty_has_cached(&args[1].ty, self.program) => {
                let mode = match b {
                    Builtin::FirstGe => 0,
                    Builtin::FirstGt => 1,
                    Builtin::LastLe => 2,
                    Builtin::LastLt => 3,
                    _ => unreachable!(),
                };
                let src_v = self.compile_expr_v(&args[0]);
                let key_v = self.compile_expr_v(&args[1]);
                let dst = self.alloc_v();
                self.emit(Op::SetMapBoundV { dst, src_v, key_v, mode }, span);
                if key_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: key_v }, span);
                }
                if src_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: src_v }, span);
                }
                dst
            }
            ExprKind::Mutate(m @ (Mutation::PopFirst | Mutation::PopLast), place, args)
                if args.is_empty() && matches!(place.ty, Ty::Set(_) | Ty::Map(_, _)) =>
            {
                let saved_ops = self.ops.len();
                let saved_spans = self.spans.len();
                let saved_places = self.places.len();
                if let Some(path) = self.compile_place_path(place) {
                    let dst = self.alloc_v();
                    self.emit(
                        Op::SetMapPopEndPlace {
                            dst,
                            path,
                            is_last: matches!(m, Mutation::PopLast),
                        },
                        span,
                    );
                    dst
                } else {
                    self.ops.truncate(saved_ops);
                    self.spans.truncate(saved_spans);
                    self.places.truncate(saved_places);
                    self.emit_fallback_expr_v(expr)
                }
            }
            ExprKind::Index(base, indices)
                if matches!(&base.ty, Ty::Map(kty, _) if !crate::value::ty_has_cached(kty, self.program))
                    && indices.len() == 1 =>
            {
                let Ty::Map(kty, _) = &base.ty else { unreachable!() };
                let map_v = self.compile_expr_v(base);
                let key_v = self.compile_expr_v(&indices[0]);
                let key_ty = self.fill_tys.len() as u16;
                self.fill_tys.push(kty);
                let dst = self.alloc_v();
                self.emit(Op::MapIndexV { dst, map_v, key_v, key_ty }, span);
                if key_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: key_v }, span);
                }
                if map_v >= self.num_local_v {
                    self.emit(Op::ClearV { reg: map_v }, span);
                }
                dst
            }
            ExprKind::CallExpr(callee, args) => {
                if let Some((_, dst_v, _)) =
                    self.try_compile_call_closure(callee, args, CallRet::V, span)
                {
                    dst_v
                } else {
                    self.emit_fallback_expr_v(expr)
                }
            }
            _ => self.emit_fallback_expr_v(expr),
        }
    }

    fn try_compile_mutate_stmt(
        &mut self,
        op: Mutation,
        place: &'p Place,
        args: &'p [Expr],
        span: Span,
    ) -> bool {
        if op == Mutation::Reverse
            && args.is_empty()
            && matches!(place.ty, Ty::List(_) | Ty::Array { .. } | Ty::Str)
        {
            let saved_ops = self.ops.len();
            let saved_spans = self.spans.len();
            let saved_places = self.places.len();
            if let Some(path) = self.compile_place_path(place) {
                self.emit(Op::ReversePlace { path }, span);
                return true;
            }
            self.ops.truncate(saved_ops);
            self.spans.truncate(saved_spans);
            self.places.truncate(saved_places);
        }
        if matches!(op, Mutation::PopFirst | Mutation::PopLast)
            && args.is_empty()
            && matches!(place.ty, Ty::Set(_) | Ty::Map(_, _))
        {
            let saved_ops = self.ops.len();
            let saved_spans = self.spans.len();
            let saved_places = self.places.len();
            if let Some(path) = self.compile_place_path(place) {
                let dst = self.alloc_v();
                self.emit(
                    Op::SetMapPopEndPlace {
                        dst,
                        path,
                        is_last: matches!(op, Mutation::PopLast),
                    },
                    span,
                );
                self.emit(Op::ClearV { reg: dst }, span);
                return true;
            }
            self.ops.truncate(saved_ops);
            self.spans.truncate(saved_spans);
            self.places.truncate(saved_places);
        }
        if !matches!(place.ty, Ty::List(_)) {
            return false;
        }
        match (op, &place.kind) {
            (Mutation::Push, PlaceKind::Local(slot)) if args.len() == 1 => {
                let SlotLoc::V(list_v) = self.slot_loc(*slot) else {
                    return false;
                };
                if let Some(kind) = ScalarKind::of_ty(&args[0].ty, self.program) {
                    let val_i = self.compile_expr_i(&args[0]);
                    self.emit(Op::ListPushLI { list_v, val_i, kind }, span);
                } else {
                    let val_v = self.compile_expr_v(&args[0]);
                    self.emit(Op::ListPushLV { list_v, val_v }, span);
                }
                true
            }
            (Mutation::Push, PlaceKind::Global(gid)) if args.len() == 1 => {
                if let Some(kind) = ScalarKind::of_ty(&args[0].ty, self.program) {
                    let val_i = self.compile_expr_i(&args[0]);
                    self.emit(Op::ListPushGI { gid: *gid as u16, val_i, kind }, span);
                } else {
                    let val_v = self.compile_expr_v(&args[0]);
                    self.emit(Op::ListPushGV { gid: *gid as u16, val_v }, span);
                }
                true
            }
            (Mutation::Pop, PlaceKind::Local(slot)) if args.is_empty() => {
                let SlotLoc::V(list_v) = self.slot_loc(*slot) else {
                    return false;
                };
                self.emit(Op::ListPopDiscardL { list_v }, span);
                true
            }
            (Mutation::Pop, PlaceKind::Global(gid)) if args.is_empty() => {
                self.emit(Op::ListPopDiscardG { gid: *gid as u16 }, span);
                true
            }
            (Mutation::Push, PlaceKind::Field(obj_place, field)) if args.len() == 1 => {
                let PlaceKind::Local(slot) = obj_place.kind else {
                    return false;
                };
                let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                    return false;
                };
                if let Some(kind) = ScalarKind::of_ty(&args[0].ty, self.program) {
                    let val_i = self.compile_expr_i(&args[0]);
                    self.emit(
                        Op::ListPushFieldLI { obj_v, field: *field as u16, val_i, kind },
                        span,
                    );
                } else {
                    let val_v = self.compile_expr_v(&args[0]);
                    self.emit(
                        Op::ListPushFieldLV { obj_v, field: *field as u16, val_v },
                        span,
                    );
                }
                true
            }
            (Mutation::Pop, PlaceKind::Field(obj_place, field)) if args.is_empty() => {
                let PlaceKind::Local(slot) = obj_place.kind else {
                    return false;
                };
                let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                    return false;
                };
                self.emit(Op::ListPopDiscardFieldL { obj_v, field: *field as u16 }, span);
                true
            }
            (Mutation::Push, PlaceKind::Index(base, indices))
                if matches!(base.ty, Ty::Array { .. }) && indices.len() == 1 && args.len() == 1 =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        if let Some(kind) = ScalarKind::of_ty(&args[0].ty, self.program) {
                            let val_i = self.compile_expr_i(&args[0]);
                            self.emit(
                                Op::ListPushGArr1I { gid: *gid as u16, r_i, val_i, kind },
                                span,
                            );
                        } else {
                            let val_v = self.compile_expr_v(&args[0]);
                            self.emit(Op::ListPushGArr1V { gid: *gid as u16, r_i, val_v }, span);
                        }
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        if let Some(kind) = ScalarKind::of_ty(&args[0].ty, self.program) {
                            let val_i = self.compile_expr_i(&args[0]);
                            self.emit(Op::ListPushLArr1I { arr_v, r_i, val_i, kind }, span);
                        } else {
                            let val_v = self.compile_expr_v(&args[0]);
                            self.emit(Op::ListPushLArr1V { arr_v, r_i, val_v }, span);
                        }
                        true
                    }
                    _ => false,
                }
            }
            _ => match op {
                Mutation::Pop if args.is_empty() => {
                    let saved_ops = self.ops.len();
                    let saved_spans = self.spans.len();
                    let saved_places = self.places.len();
                    let Some(path) = self.compile_place_path(place) else {
                        self.ops.truncate(saved_ops);
                        self.spans.truncate(saved_spans);
                        self.places.truncate(saved_places);
                        return false;
                    };
                    self.emit(Op::ListPopDiscardPlace { path }, span);
                    true
                }
                Mutation::Push if args.len() == 1 => {
                    let saved_ops = self.ops.len();
                    let saved_spans = self.spans.len();
                    let saved_places = self.places.len();
                    let Some(path) = self.compile_place_path(place) else {
                        self.ops.truncate(saved_ops);
                        self.spans.truncate(saved_spans);
                        self.places.truncate(saved_places);
                        return false;
                    };
                    if let Some(kind) = ScalarKind::of_ty(&args[0].ty, self.program) {
                        let val_i = self.compile_expr_i(&args[0]);
                        self.emit(Op::ListPushPlaceI { path, val_i, kind }, span);
                    } else {
                        let val_v = self.compile_expr_v(&args[0]);
                        self.emit(Op::ListPushPlaceV { path, val_v }, span);
                    }
                    true
                }
                _ => false,
            },
        }
    }

    fn compile_print(&mut self, args: &'p [FmtArg], end: Option<&'p Expr>, span: Span) {
        let regs: Vec<u16> = args.iter().map(|a| self.compile_expr_v(&a.value)).collect();
        let end = end.map(|e| self.compile_expr_v(e));
        let idx = self.prints.len() as u32;
        self.prints.push(PrintSpec { args, regs: regs.clone(), end });
        self.emit(Op::Print { idx }, span);
        for reg in regs.into_iter().chain(end) {
            if reg >= self.num_local_v {
                self.emit(Op::ClearV { reg }, span);
            }
        }
    }

    fn compile_format(&mut self, pieces: &'p [FmtPiece], span: Span) -> u16 {
        let regs: Vec<u16> = pieces
            .iter()
            .filter_map(|p| match p {
                FmtPiece::Text(_) => None,
                FmtPiece::Value(a) => Some(&a.value),
                FmtPiece::Number(e, _) => Some(e),
            })
            .map(|e| self.compile_expr_v(e))
            .collect();
        let dst = self.alloc_v();
        let idx = self.formats.len() as u32;
        self.formats.push(FormatSpec { pieces, regs: regs.clone() });
        self.emit(Op::Format { idx, dst }, span);
        for reg in regs {
            if reg >= self.num_local_v {
                self.emit(Op::ClearV { reg }, span);
            }
        }
        dst
    }

    fn emit_fallback_stmt(&mut self, stmt: &'p Stmt) {
        let idx = self.fallback_stmts.len() as u32;
        self.fallback_stmts.push(stmt);
        let in_loop = !self.loops.is_empty();
        self.emit(
            Op::FallbackStmt { idx, clear_from_v: self.pinned_v, in_loop },
            stmt.span().unwrap_or_default(),
        );
    }

    fn emit_fallback_expr_i(&mut self, expr: &'p Expr) -> u16 {
        let idx = self.fallback_exprs.len() as u32;
        self.fallback_exprs.push(expr);
        let dst = self.alloc_i();
        let in_loop = !self.loops.is_empty();
        self.emit(Op::FallbackExprI { idx, dst, in_loop }, expr.span);
        dst
    }

    fn compile_expr_small_tuple(&mut self, expr: &'p Expr, dst_base: u16, len: u8) {
        let span = expr.span;
        match &expr.kind {
            ExprKind::Tuple(fs) if fs.len() == len as usize => {
                let mut regs = [0u16; 4];
                for (k, f) in fs.iter().enumerate() {
                    regs[k] = self.compile_expr_i(f);
                }
                for (k, &r) in regs.iter().enumerate().take(len as usize) {
                    let d = dst_base + k as u16;
                    if r != d {
                        self.emit(Op::MoveI { dst: d, src: r }, span);
                    }
                }
            }
            ExprKind::If(cond, then_e, else_e) => {
                let j_else = self.emit_branch_if_false(cond);
                self.compile_expr_small_tuple(then_e, dst_base, len);
                let j_end = self.emit(Op::Jmp { target: 0 }, span);
                let else_pc = self.ops.len() as u32;
                self.patch(j_else, else_pc);
                self.compile_expr_small_tuple(else_e, dst_base, len);
                let end_pc = self.ops.len() as u32;
                self.patch(j_end, end_pc);
            }
            ExprKind::Local(slot) => match self.slot_loc(*slot) {
                SlotLoc::SmallTuple { base, len: slen, .. } if slen == len => {
                    for k in 0..len as u16 {
                        if dst_base + k != base + k {
                            self.emit(Op::MoveI { dst: dst_base + k, src: base + k }, span);
                        }
                    }
                }
                SlotLoc::V(src_v) => {
                    self.emit(Op::UnboxSmallTuple { dst_base, len, src_v }, span);
                }
                _ => {
                    let src_v = self.emit_fallback_expr_v(expr);
                    self.emit(Op::UnboxSmallTuple { dst_base, len, src_v }, span);
                }
            },
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::Array { .. })
                    && (indices.len() == 1 || indices.len() == 2) =>
            {
                match &base.kind {
                    ExprKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        if indices.len() == 2 {
                            let r_j = self.compile_expr_i(&indices[1]);
                            self.emit(
                                Op::LoadGArr2SmallTuple {
                                    dst_base,
                                    len,
                                    gid: *gid as u16,
                                    r_i,
                                    r_j,
                                },
                                span,
                            );
                        } else {
                            self.emit(
                                Op::LoadGArr1SmallTuple {
                                    dst_base,
                                    len,
                                    gid: *gid as u16,
                                    r_i,
                                },
                                span,
                            );
                        }
                    }
                    ExprKind::Local(slot) => {
                        if let SlotLoc::V(arr_v) = self.slot_loc(*slot) {
                            let r_i = self.compile_expr_i(&indices[0]);
                            if indices.len() == 2 {
                                let r_j = self.compile_expr_i(&indices[1]);
                                self.emit(
                                    Op::LoadLArr2SmallTuple { dst_base, len, arr_v, r_i, r_j },
                                    span,
                                );
                            } else {
                                self.emit(
                                    Op::LoadLArr1SmallTuple { dst_base, len, arr_v, r_i },
                                    span,
                                );
                            }
                        } else {
                            let src_v = self.emit_fallback_expr_v(expr);
                            self.emit(Op::UnboxSmallTuple { dst_base, len, src_v }, span);
                        }
                    }
                    _ => {
                        let src_v = self.emit_fallback_expr_v(expr);
                        self.emit(Op::UnboxSmallTuple { dst_base, len, src_v }, span);
                    }
                }
            }
            _ => {
                let src_v = self.compile_expr_v(expr);
                self.emit(Op::UnboxSmallTuple { dst_base, len, src_v }, span);
            }
        }
    }

    fn emit_fallback_expr_v(&mut self, expr: &'p Expr) -> u16 {
        let idx = self.fallback_exprs.len() as u32;
        self.fallback_exprs.push(expr);
        let dst = self.alloc_v();
        let in_loop = !self.loops.is_empty();
        self.emit(Op::FallbackExprV { idx, dst, in_loop }, expr.span);
        dst
    }
}

fn block_has_by_ref_closure(block: &Block, _program: &Program) -> bool {
    Compiler::stmts_have_closure(&block.stmts)
}

fn ty_uses_structural_cmp(ty: &Ty, program: &Program) -> bool {
    match ty {
        Ty::Struct(id) => {
            let s = &program.structs[*id];
            s.eq_fn.is_none()
                && s.lt_fn.is_none()
                && s.cached_fields.is_empty()
                && s.fields.iter().all(|(_, t)| ty_uses_structural_cmp(t, program))
        }
        Ty::Tuple(ts) => ts.iter().all(|t| ty_uses_structural_cmp(t, program)),
        Ty::Record(fs) => fs.iter().all(|(_, t)| ty_uses_structural_cmp(t, program)),
        Ty::List(t) | Ty::Opt(t) => ty_uses_structural_cmp(t, program),
        Ty::Array { elem, .. } => ty_uses_structural_cmp(elem, program),
        Ty::Int | Ty::Float | Ty::Bool | Ty::Char | Ty::Str | Ty::Unit | Ty::Pos(_) | Ty::Bound(_) => true,
        _ => false,
    }
}

/// In a traced run a local the VM keeps in a register is left out of the
/// stack, which the tracer shows, until it is declared, as in the tree-walker.
#[inline(always)]
fn undeclared(traced: bool, loc: &SlotLoc, held: &Value) -> bool {
    traced && !matches!(loc, SlotLoc::V(_)) && matches!(held, Value::Unit)
}

fn sync_to_stack(
    slot_map: &[SlotLoc],
    regs_i: &[i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
    traced: bool,
) {
    for (slot, loc) in slot_map.iter().enumerate() {
        if undeclared(traced, loc, &stack[base + slot]) {
            continue;
        }
        // The tree-walker, which a fallback hands the value to, may have
        // stopped with a trap before handing it back.
        if traced && matches!(*loc, SlotLoc::V(r) if matches!(regs_v[r as usize], Value::Unit)) {
            continue;
        }
        stack[base + slot] = match *loc {
            SlotLoc::I(r, kind) => kind.box_val(regs_i[r as usize]),
            SlotLoc::OptI { val, some, kind } => {
                kind.box_opt(regs_i[val as usize], regs_i[some as usize])
            }
            SlotLoc::SmallTuple { base: b, len, kinds } => {
                let n = len as usize;
                let mut fs = Vec::with_capacity(n);
                for k in 0..n {
                    fs.push(kinds[k].box_val(regs_i[b as usize + k]));
                }
                Value::Tuple(TupleVal::Boxed(Rc::new(fs)))
            }
            SlotLoc::V(r) => std::mem::replace(&mut regs_v[r as usize], Value::Unit),
        };
    }
}

/// Hands the moved values back to their registers after a trace point,
/// which reads the stack but writes none of the scalars kept in registers.
fn take_back_values(slot_map: &[SlotLoc], regs_v: &mut [Value], stack: &mut [Value], base: usize) {
    for (slot, loc) in slot_map.iter().enumerate() {
        if let SlotLoc::V(r) = *loc {
            regs_v[r as usize] = std::mem::replace(&mut stack[base + slot], Value::Unit);
        }
    }
}

fn sync_from_stack(
    slot_map: &[SlotLoc],
    regs_i: &mut [i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
    traced: bool,
) {
    for (slot, loc) in slot_map.iter().enumerate() {
        if undeclared(traced, loc, &stack[base + slot]) {
            continue;
        }
        match *loc {
            SlotLoc::I(r, _) => regs_i[r as usize] = ScalarKind::unbox_val(&stack[base + slot]),
            SlotLoc::OptI { val, some, .. } => {
                let (v, s) = ScalarKind::unbox_opt(&stack[base + slot]);
                regs_i[val as usize] = v;
                regs_i[some as usize] = s;
            }
            SlotLoc::SmallTuple { base: b, len, .. } => {
                if let Value::Tuple(TupleVal::Boxed(fs)) = &stack[base + slot] {
                    for (k, f) in fs.iter().take(len as usize).enumerate() {
                        regs_i[b as usize + k] = ScalarKind::unbox_val(f);
                    }
                }
            }
            SlotLoc::V(r) => {
                regs_v[r as usize] = std::mem::replace(&mut stack[base + slot], Value::Unit);
            }
        }
    }
}

#[inline(always)]
fn read_array_i(arr: &Array, at: usize) -> i64 {
    unsafe {
        match &arr.data {
            ArrayData::Int(v) | ArrayData::OneIntStruct(v) => *v.get_unchecked(at),
            ArrayData::Bool(v) | ArrayData::Char(v) => i64::from(*v.get_unchecked(at)),
            ArrayData::Values(v) => ScalarKind::unbox_val(v.get_unchecked(at)),
            ArrayData::OptInt(v) => v.get_unchecked(at).unwrap_or(0),
            ArrayData::Float(_) | ArrayData::SmallIntTuple { .. } => 0,
        }
    }
}

#[inline(always)]
fn write_array_i(arr: &mut Array, at: usize, val: i64) {
    unsafe {
        match &mut arr.data {
            ArrayData::Int(v) | ArrayData::OneIntStruct(v) => *v.get_unchecked_mut(at) = val,
            ArrayData::Bool(v) => *v.get_unchecked_mut(at) = u8::from(val != 0),
            ArrayData::Char(v) => *v.get_unchecked_mut(at) = val as u8,
            ArrayData::Values(v) => *v.get_unchecked_mut(at) = Value::Int(val),
            ArrayData::OptInt(v) => *v.get_unchecked_mut(at) = Some(val),
            ArrayData::Float(_) | ArrayData::SmallIntTuple { .. } => {}
        }
    }
}

#[inline(always)]
fn read_array_opt_i(arr: &Array, at: usize) -> (i64, i64) {
    unsafe {
        match &arr.data {
            ArrayData::OptInt(v) => match *v.get_unchecked(at) {
                Some(x) => (x, 1),
                None => (0, 0),
            },
            ArrayData::Values(v) => ScalarKind::unbox_opt(v.get_unchecked(at)),
            _ => (0, 0),
        }
    }
}

#[inline(always)]
fn write_array_opt_i(arr: &mut Array, at: usize, val: i64, some: i64, kind: ScalarKind) {
    match &mut arr.data {
        ArrayData::OptInt(v) if kind == ScalarKind::Int => unsafe {
            *v.get_unchecked_mut(at) = if some != 0 { Some(val) } else { None };
        },
        ArrayData::Values(v) => unsafe {
            *v.get_unchecked_mut(at) = kind.box_opt(val, some);
        },
        _ => {
            arr.as_mut_values()[at] = kind.box_opt(val, some);
        }
    }
}

impl<'p, 'io> Interp<'p, 'io> {
    fn take_regs_v(&mut self, n: usize) -> Vec<Value> {
        if n == 0 {
            return Vec::new();
        }
        let mut v = self.vm_pool_v.pop().unwrap_or_default();
        v.clear();
        v.resize(n, Value::Unit);
        v
    }

    fn return_regs_v(&mut self, mut v: Vec<Value>) {
        if v.capacity() > 0 && self.vm_pool_v.len() < 64 {
            v.clear();
            self.vm_pool_v.push(v);
        }
    }

    fn trace_point(&mut self, point: &TracePoint<'p>) {
        match *point {
            TracePoint::Let(slot, init) => {
                self.trace_local(slot, init.span);
                self.trace_assigned_local(slot, init.span);
            }
            TracePoint::Assign(place) => {
                let mut indices = std::mem::take(&mut self.trace_indices);
                indices.clear();
                if self.indices(place, &mut indices).is_ok() {
                    self.trace_change(place, &indices);
                }
                indices.clear();
                self.trace_indices = indices;
                self.trace_assigned(place);
            }
            TracePoint::Loop(slot, span) => {
                self.trace_local(slot, span);
                self.trace_assigned_local(slot, span);
            }
            TracePoint::LoopEnd(_) => {}
        }
    }

    pub(crate) fn run_main_vm(&mut self, compiled: &CompiledProgram<'p>) -> Res<()> {
        let mut vm_regs_i = std::mem::take(&mut self.vm_regs_i);
        if vm_regs_i.capacity() < 4096 {
            vm_regs_i.reserve(4096);
        }
        let res = self.exec_chunk(compiled, &compiled.main, &[], &mut vm_regs_i);
        self.vm_regs_i = vm_regs_i;
        let _ = res?;
        Ok(())
    }

    pub(crate) fn run_fn_vm_from_stack(
        &mut self,
        compiled: &CompiledProgram<'p>,
        fn_id: FnId,
    ) -> Res<Value> {
        let chunk = compiled.fns[fn_id].as_ref().unwrap();
        let mut aliases: Vec<(usize, bool, usize)> = Vec::new();
        for k in 0..chunk.param_count {
            if let Value::Alias(is_global, at) = self.stack[self.base + k] {
                let val = if is_global {
                    std::mem::replace(&mut self.globals[at], Value::Moved)
                } else {
                    std::mem::replace(&mut self.stack[at], Value::Moved)
                };
                self.stack[self.base + k] = val;
                aliases.push((k, is_global, at));
            }
        }
        let mut vm_regs_i = std::mem::take(&mut self.vm_regs_i);
        let base_i = vm_regs_i.len();
        vm_regs_i.resize(base_i + chunk.num_i, 0);
        let mut regs_v = self.take_regs_v(chunk.num_v);
        sync_from_stack(
            &chunk.slot_map,
            &mut vm_regs_i[base_i..base_i + chunk.num_i],
            &mut regs_v,
            &mut self.stack,
            self.base, chunk.traced
        );
        let res = self.exec_chunk_inner(compiled, chunk, base_i, &mut vm_regs_i, &mut regs_v);
        for (k, is_global, at) in aliases {
            let val = match chunk.slot_map[k] {
                SlotLoc::I(r, kind) => kind.box_val(vm_regs_i[base_i + r as usize]),
                SlotLoc::OptI { val, some, kind } => {
                    kind.box_opt(vm_regs_i[base_i + val as usize], vm_regs_i[base_i + some as usize])
                }
                SlotLoc::V(r) => std::mem::replace(&mut regs_v[r as usize], Value::Unit),
                SlotLoc::SmallTuple { .. } => unreachable!("params are never SmallTuple"),
            };
            if is_global {
                self.globals[at] = val;
            } else {
                self.stack[at] = val;
            }
        }
        self.return_regs_v(regs_v);
        vm_regs_i.truncate(base_i);
        self.vm_regs_i = vm_regs_i;
        res
    }

    fn exec_chunk(
        &mut self,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        args_i: &[i64],
        vm_regs_i: &mut Vec<i64>,
    ) -> Res<Value> {
        let base_i = vm_regs_i.len();
        vm_regs_i.resize(base_i + chunk.num_i, 0);
        for (i, &v) in args_i.iter().enumerate() {
            if let Some(SlotLoc::I(r, _)) = chunk.slot_map.get(i) {
                unsafe {
                    *vm_regs_i.get_unchecked_mut(base_i + *r as usize) = v;
                }
            }
        }
        let mut regs_v = self.take_regs_v(chunk.num_v);
        let dom_mark = self.saved_domains.len();
        for &d in &chunk.guarded_domains {
            self.saved_domains.push((d, self.domains[d]));
        }
        let res = self.exec_chunk_inner(compiled, chunk, base_i, vm_regs_i, &mut regs_v);
        while self.saved_domains.len() > dom_mark {
            let (id, prev) = self.saved_domains.pop().unwrap();
            self.domains[id] = prev;
        }
        self.return_regs_v(regs_v);
        vm_regs_i.truncate(base_i);
        res
    }

    fn exec_chunk_mix(
        &mut self,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        args_i: &mut [i64],
        args_v: &mut [Value],
        params: &[cpl_ir::ParamDef],
        vm_regs_i: &mut Vec<i64>,
    ) -> Res<Value> {
        let base_i = vm_regs_i.len();
        vm_regs_i.resize(base_i + chunk.num_i, 0);
        let mut regs_v = self.take_regs_v(chunk.num_v);
        let mut ii = 0usize;
        let mut iv = 0usize;
        for loc in chunk.slot_map.iter().take(chunk.param_count) {
            match *loc {
                SlotLoc::I(r, _) => {
                    unsafe {
                        *vm_regs_i.get_unchecked_mut(base_i + r as usize) = args_i[ii];
                    }
                    ii += 1;
                }
                SlotLoc::OptI { val, some, .. } => {
                    unsafe {
                        *vm_regs_i.get_unchecked_mut(base_i + val as usize) = args_i[ii];
                        *vm_regs_i.get_unchecked_mut(base_i + some as usize) = args_i[ii + 1];
                    }
                    ii += 2;
                }
                SlotLoc::V(r) => {
                    regs_v[r as usize] = std::mem::replace(&mut args_v[iv], Value::Unit);
                    iv += 1;
                }
                SlotLoc::SmallTuple { .. } => unreachable!("params are never SmallTuple"),
            }
        }
        let dom_mark = self.saved_domains.len();
        for &d in &chunk.guarded_domains {
            self.saved_domains.push((d, self.domains[d]));
        }
        let res = self.exec_chunk_inner(compiled, chunk, base_i, vm_regs_i, &mut regs_v);
        while self.saved_domains.len() > dom_mark {
            let (id, prev) = self.saved_domains.pop().unwrap();
            self.domains[id] = prev;
        }
        if res.is_ok() && params.iter().any(|p| p.mutable) {
            let mut ii = 0usize;
            let mut iv = 0usize;
            for (k, loc) in chunk.slot_map.iter().take(chunk.param_count).enumerate() {
                let is_mut = params[k].mutable;
                match *loc {
                    SlotLoc::I(r, _) => {
                        if is_mut {
                            args_i[ii] = vm_regs_i[base_i + r as usize];
                        }
                        ii += 1;
                    }
                    SlotLoc::OptI { val, some, .. } => {
                        if is_mut {
                            args_i[ii] = vm_regs_i[base_i + val as usize];
                            args_i[ii + 1] = vm_regs_i[base_i + some as usize];
                        }
                        ii += 2;
                    }
                    SlotLoc::V(r) => {
                        if is_mut {
                            args_v[iv] = std::mem::replace(&mut regs_v[r as usize], Value::Unit);
                        }
                        iv += 1;
                    }
                    SlotLoc::SmallTuple { .. } => unreachable!("params are never SmallTuple"),
                }
            }
        }
        self.return_regs_v(regs_v);
        vm_regs_i.truncate(base_i);
        res
    }

    fn exec_chunk_inner(
        &mut self,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        base_i: usize,
        vm_regs_i: &mut Vec<i64>,
        regs_v: &mut [Value],
    ) -> Res<Value> {
        let exit = self.exec_chunk_core(compiled, chunk, base_i, vm_regs_i, regs_v, 0, None);
        if chunk.traced {
            // The end of the run and a trap show the locals as they were left.
            let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
            sync_to_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base, true);
        }
        match exit? {
            ExecExit::Return(v) => Ok(v),
            ExecExit::FeedNext | ExecExit::FeedBreak => Ok(Value::Unit),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn exec_chunk_core(
        &mut self,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        base_i: usize,
        vm_regs_i: &mut Vec<i64>,
        regs_v: &mut [Value],
        start_pc: usize,
        mut fed: Option<&mut FedCtx<'_, 'p>>,
    ) -> Res<ExecExit> {
        let ops = chunk.ops.as_slice();
        let spans = chunk.spans.as_slice();
        let mut pc: usize = start_pc;
        let mut regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
        let mut escaped: Option<(bool, Option<LoopId>)> = None;

        loop {
            let op = unsafe { *ops.get_unchecked(pc) };
            let cur_pc = pc;
            pc += 1;
            match op {
                Op::ConstImmI { dst, val } => unsafe {
                    *regs_i.get_unchecked_mut(dst as usize) = i64::from(val);
                },
                Op::ConstI { dst, idx } => unsafe {
                    *regs_i.get_unchecked_mut(dst as usize) = *chunk.consts_i.get_unchecked(idx as usize);
                },
                Op::MoveI { dst, src } => unsafe {
                    *regs_i.get_unchecked_mut(dst as usize) = *regs_i.get_unchecked(src as usize);
                },
                Op::MoveV { dst, src } => {
                    let v = if src as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[src as usize], Value::Unit)
                    } else {
                        regs_v[src as usize].clone()
                    };
                    regs_v[dst as usize] = v;
                }
                Op::MoveVConsume { dst, src } => {
                    regs_v[dst as usize] =
                        std::mem::replace(&mut regs_v[src as usize], Value::Unit);
                }
                Op::BoxI { dst_v, src_i, kind } => {
                    regs_v[dst_v as usize] = kind.box_val(regs_i[src_i as usize]);
                }
                Op::UnboxI { dst_i, src_v } => {
                    regs_i[dst_i as usize] = ScalarKind::unbox_val(&regs_v[src_v as usize]);
                }
                Op::BoxOptI { dst_v, val, some, kind } => {
                    regs_v[dst_v as usize] = kind.box_opt(regs_i[val as usize], regs_i[some as usize]);
                }
                Op::UnboxOptI { dst_val, dst_some, src_v } => {
                    let (v, s) = ScalarKind::unbox_opt(&regs_v[src_v as usize]);
                    regs_i[dst_val as usize] = v;
                    regs_i[dst_some as usize] = s;
                }
                Op::LoadGlobalI { dst, gid } => {
                    regs_i[dst as usize] = ScalarKind::unbox_val(&self.globals[gid as usize]);
                }
                Op::StoreGlobalI { gid, src, kind } => {
                    self.globals[gid as usize] = kind.box_val(regs_i[src as usize]);
                }
                Op::LoadGlobalOptI { dst_val, dst_some, gid } => {
                    let (v, s) = ScalarKind::unbox_opt(&self.globals[gid as usize]);
                    regs_i[dst_val as usize] = v;
                    regs_i[dst_some as usize] = s;
                }
                Op::StoreGlobalOptI { gid, val, some, kind } => {
                    self.globals[gid as usize] =
                        kind.box_opt(regs_i[val as usize], regs_i[some as usize]);
                }
                Op::LoadGlobalV { dst, gid } => {
                    regs_v[dst as usize] = self.globals[gid as usize].clone();
                }
                Op::StoreGlobalV { gid, src } => {
                    self.globals[gid as usize] = if src as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[src as usize], Value::Unit)
                    } else {
                        regs_v[src as usize].clone()
                    };
                }
                Op::StoreGlobalVMove { gid, src } => {
                    self.globals[gid as usize] =
                        std::mem::replace(&mut regs_v[src as usize], Value::Unit);
                }
                Op::AddI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    match av.checked_add(bv).filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} + {bv}` does not fit in an `Int` (64 bits)")),
                    }
                }
                Op::SubI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    match av.checked_sub(bv).filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} - {bv}` does not fit in an `Int` (64 bits)")),
                    }
                }
                Op::MulI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    match av.checked_mul(bv).filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} * {bv}` does not fit in an `Int` (64 bits)")),
                    }
                }
                Op::DivI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    if bv == 0 {
                        return trap(spans[cur_pc], format!("division by zero: `{av} / 0`"));
                    }
                    match av.checked_div_euclid(bv).filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} / {bv}` does not fit in an `Int` (64 bits)")),
                    }
                }
                Op::RemI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    if bv == 0 {
                        return trap(spans[cur_pc], format!("division by zero: `{av} % 0`"));
                    }
                    regs_i[dst as usize] = av.checked_rem_euclid(bv).unwrap_or(0);
                }
                Op::ModAddI { dst, a, b, modulus } => {
                    let s = regs_i[a as usize] + regs_i[b as usize];
                    let m = modulus as i64;
                    regs_i[dst as usize] = if s >= m { s - m } else { s };
                }
                Op::ModSubI { dst, a, b, modulus } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    let m = modulus as i64;
                    regs_i[dst as usize] = if av >= bv { av - bv } else { av - bv + m };
                }
                Op::ModMulI { dst, a, b, modulus } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    let m = modulus as i64;
                    let prod = av * bv;
                    regs_i[dst as usize] = if prod >= 0 { prod % m } else { prod.rem_euclid(m) };
                }
                Op::ModNegI { dst, a, modulus } => {
                    let av = regs_i[a as usize];
                    regs_i[dst as usize] = if av == 0 { 0 } else { modulus as i64 - av };
                }
                Op::AddImmI { dst, a, imm } => {
                    let av = regs_i[a as usize];
                    let bv = i64::from(imm);
                    match av.checked_add(bv).filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} + {bv}` does not fit in an `Int` (64 bits)")),
                    }
                }
                Op::NegI { dst, a } => {
                    let av = regs_i[a as usize];
                    match av.checked_neg().filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `-{av}` does not fit in an `Int` (64 bits)")),
                    }
                }
                Op::BitAndI { dst, a, b } => {
                    let (av, bv) = (regs_i[a as usize], regs_i[b as usize]);
                    let v = av & bv;
                    if v == i64::MIN {
                        return trap(spans[cur_pc], format!("integer overflow: `{av} & {bv}` does not fit in an `Int` (64 bits)"));
                    }
                    regs_i[dst as usize] = v;
                }
                Op::BitOrI { dst, a, b } => {
                    let (av, bv) = (regs_i[a as usize], regs_i[b as usize]);
                    let v = av | bv;
                    if v == i64::MIN {
                        return trap(spans[cur_pc], format!("integer overflow: `{av} | {bv}` does not fit in an `Int` (64 bits)"));
                    }
                    regs_i[dst as usize] = v;
                }
                Op::BitXorI { dst, a, b } => {
                    let (av, bv) = (regs_i[a as usize], regs_i[b as usize]);
                    let v = av ^ bv;
                    if v == i64::MIN {
                        return trap(spans[cur_pc], format!("integer overflow: `{av} ^ {bv}` does not fit in an `Int` (64 bits)"));
                    }
                    regs_i[dst as usize] = v;
                }
                Op::ShlI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    if !(0..64).contains(&bv) {
                        return trap(spans[cur_pc], format!("a shift by {bv} is outside 0..64"));
                    }
                    if av < 0 {
                        return trap(spans[cur_pc], format!("`<<` of the negative number {av}"));
                    }
                    if bv > 0 && av > (i64::MAX >> bv) {
                        return trap(spans[cur_pc], format!("integer overflow: `{av} << {bv}` does not fit in an `Int` (64 bits)"));
                    }
                    regs_i[dst as usize] = av << bv;
                }
                Op::ShrI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    if !(0..64).contains(&bv) {
                        return trap(spans[cur_pc], format!("a shift by {bv} is outside 0..64"));
                    }
                    regs_i[dst as usize] = av >> bv;
                }
                Op::NotBool { dst, a } => {
                    regs_i[dst as usize] = i64::from(regs_i[a as usize] == 0);
                }
                Op::MinI { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize].min(regs_i[b as usize]);
                }
                Op::MaxI { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize].max(regs_i[b as usize]);
                }
                Op::WrapAddI { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize].wrapping_add(regs_i[b as usize]);
                }
                Op::WrapSubI { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize].wrapping_sub(regs_i[b as usize]);
                }
                Op::WrapMulI { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize].wrapping_mul(regs_i[b as usize]);
                }
                Op::AbsI { dst, a } => {
                    let av = regs_i[a as usize];
                    match av.checked_abs().filter(|v| *v != i64::MIN) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `abs({av})` does not fit in an `Int` (64 bits)")),
                    }
                }
                Op::DomainLen { dst, dom } => {
                    regs_i[dst as usize] = self.domains[dom as usize].1;
                }
                Op::DomainLo { dst, dom } => {
                    regs_i[dst as usize] = self.domains[dom as usize].0;
                }
                Op::FromLabel { dst, src, dom } => {
                    let (lo, len) = self.domains[dom as usize];
                    let label = regs_i[src as usize];
                    if label < lo || label - lo >= len {
                        let name = &self.program.domains[dom as usize];
                        return trap(
                            spans[cur_pc],
                            format!("{label} is not in `{name}`, which is {}", describe(lo, len)),
                        );
                    }
                    regs_i[dst as usize] = label - lo;
                }
                Op::FromOrd { dst, src, dom } => {
                    let (lo, len) = self.domains[dom as usize];
                    let ord = regs_i[src as usize];
                    if ord < 0 || ord >= len {
                        let name = &self.program.domains[dom as usize];
                        return trap(
                            spans[cur_pc],
                            format!("`{name}` has no position {ord}: it is {}", describe(lo, len)),
                        );
                    }
                    regs_i[dst as usize] = ord;
                }
                Op::LoadGArr1SmallTuple { dst_base, len, gid, r_i } => {
                    let i = regs_i[r_i as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        let d0 = arr.dims[0];
                        if i < 0 || i >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array of length {d0}"),
                            );
                        }
                        let b = dst_base as usize;
                        let n = len as usize;
                        arr.get_small_tuple(i as usize, n, &mut regs_i[b..b + n]);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadLArr1SmallTuple { dst_base, len, arr_v, r_i } => {
                    let i = regs_i[r_i as usize];
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
                        let d0 = arr.dims[0];
                        if i < 0 || i >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array of length {d0}"),
                            );
                        }
                        let b = dst_base as usize;
                        let n = len as usize;
                        arr.get_small_tuple(i as usize, n, &mut regs_i[b..b + n]);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreGArr1SmallTuple { gid, r_i, src_base, len, kinds } => {
                    let i = regs_i[r_i as usize];
                    let b = src_base as usize;
                    let n = len as usize;
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let d0 = arr.dims[0];
                        if i < 0 || i >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array of length {d0}"),
                            );
                        }
                        Rc::make_mut(arr).set_small_tuple(i as usize, n, &regs_i[b..b + n], &kinds);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreLArr1SmallTuple { arr_v, r_i, src_base, len, kinds } => {
                    let i = regs_i[r_i as usize];
                    let b = src_base as usize;
                    let n = len as usize;
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let d0 = arr.dims[0];
                        if i < 0 || i >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array of length {d0}"),
                            );
                        }
                        Rc::make_mut(arr).set_small_tuple(i as usize, n, &regs_i[b..b + n], &kinds);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadGArr2SmallTuple { dst_base, len, gid, r_i, r_j } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        let d0 = arr.dims[0];
                        let d1 = arr.dims[1];
                        if i < 0 || i >= d0 || j < 0 || j >= d1 {
                            return trap(
                                spans[cur_pc],
                                format!("index ({i}, {j}) is outside an array of shape ({d0}, {d1})"),
                            );
                        }
                        let b = dst_base as usize;
                        let n = len as usize;
                        arr.get_small_tuple((i * d1 + j) as usize, n, &mut regs_i[b..b + n]);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadLArr2SmallTuple { dst_base, len, arr_v, r_i, r_j } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
                        let d0 = arr.dims[0];
                        let d1 = arr.dims[1];
                        if i < 0 || i >= d0 || j < 0 || j >= d1 {
                            return trap(
                                spans[cur_pc],
                                format!("index ({i}, {j}) is outside an array of shape ({d0}, {d1})"),
                            );
                        }
                        let b = dst_base as usize;
                        let n = len as usize;
                        arr.get_small_tuple((i * d1 + j) as usize, n, &mut regs_i[b..b + n]);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreGArr2SmallTuple { gid, r_i, r_j, src_base, len, kinds } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    let b = src_base as usize;
                    let n = len as usize;
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let d0 = arr.dims[0];
                        let d1 = arr.dims[1];
                        if i < 0 || i >= d0 || j < 0 || j >= d1 {
                            return trap(
                                spans[cur_pc],
                                format!("index ({i}, {j}) is outside an array of shape ({d0}, {d1})"),
                            );
                        }
                        Rc::make_mut(arr).set_small_tuple(
                            (i * d1 + j) as usize,
                            n,
                            &regs_i[b..b + n],
                            &kinds,
                        );
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreLArr2SmallTuple { arr_v, r_i, r_j, src_base, len, kinds } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    let b = src_base as usize;
                    let n = len as usize;
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let d0 = arr.dims[0];
                        let d1 = arr.dims[1];
                        if i < 0 || i >= d0 || j < 0 || j >= d1 {
                            return trap(
                                spans[cur_pc],
                                format!("index ({i}, {j}) is outside an array of shape ({d0}, {d1})"),
                            );
                        }
                        Rc::make_mut(arr).set_small_tuple(
                            (i * d1 + j) as usize,
                            n,
                            &regs_i[b..b + n],
                            &kinds,
                        );
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::BoxSmallTuple { dst_v, src_base, len, kinds } => {
                    let b = src_base as usize;
                    let n = len as usize;
                    let mut fs = Vec::with_capacity(n);
                    for k in 0..n {
                        fs.push(kinds[k].box_val(regs_i[b + k]));
                    }
                    regs_v[dst_v as usize] = Value::Tuple(TupleVal::Boxed(Rc::new(fs)));
                }
                Op::UnboxSmallTuple { dst_base, len, src_v } => {
                    let b = dst_base as usize;
                    let n = len as usize;
                    if let Value::Tuple(TupleVal::Boxed(fs)) = &regs_v[src_v as usize] {
                        for (k, f) in fs.iter().take(n).enumerate() {
                            regs_i[b + k] = ScalarKind::unbox_val(f);
                        }
                    }
                    if src_v as usize >= chunk.num_local_v {
                        regs_v[src_v as usize] = Value::Unit;
                    }
                }
                Op::PosNext { dst_val, dst_some, src, len } => {
                    let next = regs_i[src as usize] + 1;
                    let ok = next < regs_i[len as usize];
                    regs_i[dst_val as usize] = next;
                    regs_i[dst_some as usize] = i64::from(ok);
                }
                Op::PosPrev { dst_val, dst_some, src } => {
                    let ord = regs_i[src as usize];
                    let ok = ord > 0;
                    regs_i[dst_val as usize] = ord - 1;
                    regs_i[dst_some as usize] = i64::from(ok);
                }
                Op::PosOffset { dst_val, dst_some, src, delta, dom } => {
                    let len = self.domains[dom as usize].1;
                    let target = regs_i[src as usize].saturating_add(regs_i[delta as usize]);
                    let ok = target >= 0 && target < len;
                    regs_i[dst_val as usize] = target;
                    regs_i[dst_some as usize] = i64::from(ok);
                }
                Op::PosTryOrd { dst_val, dst_some, src, dom } => {
                    let len = self.domains[dom as usize].1;
                    let ord = regs_i[src as usize];
                    let ok = ord >= 0 && ord < len;
                    regs_i[dst_val as usize] = ord;
                    regs_i[dst_some as usize] = i64::from(ok);
                }
                Op::BoundStepL { dst, ord, k, dom } => {
                    let len = self.domains[dom as usize].1;
                    let kv = regs_i[k as usize];
                    let b = regs_i[ord as usize] + 1 - kv;
                    if b < 0 || b > len {
                        return trap(
                            spans[cur_pc],
                            format!("window of {kv} leaves domain (0..={len})"),
                        );
                    }
                    regs_i[dst as usize] = b;
                }
                Op::BoundStepR { dst, ord, k, dom } => {
                    let len = self.domains[dom as usize].1;
                    let kv = regs_i[k as usize];
                    let b = regs_i[ord as usize] + kv;
                    if b < 0 || b > len {
                        return trap(
                            spans[cur_pc],
                            format!("window of {kv} leaves domain (0..={len})"),
                        );
                    }
                    regs_i[dst as usize] = b;
                }
                Op::UnwrapOptI { dst, val, some } => {
                    if regs_i[some as usize] == 0 {
                        return trap(spans[cur_pc], "`!` on `none`: there is no value here");
                    }
                    regs_i[dst as usize] = regs_i[val as usize];
                }
                Op::CmpI { dst, a, b, op } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    let res = match op {
                        CmpOp::Lt => av < bv,
                        CmpOp::Le => av <= bv,
                        CmpOp::Gt => av > bv,
                        CmpOp::Ge => av >= bv,
                        CmpOp::Eq => av == bv,
                        CmpOp::Ne => av != bv,
                        _ => false,
                    };
                    regs_i[dst as usize] = i64::from(res);
                }
                Op::Jmp { target } => pc = target as usize,
                Op::JmpEscape { brk, id, innermost, target } => {
                    if let Some((was_brk, was)) = escaped {
                        if was_brk == brk && (was == Some(id) || (was.is_none() && innermost)) {
                            escaped = None;
                            pc = target as usize;
                        }
                    }
                }
                Op::JmpIfZero { reg, target } => {
                    if regs_i[reg as usize] == 0 {
                        pc = target as usize;
                    }
                }
                Op::JmpIfNonZero { reg, target } => {
                    if regs_i[reg as usize] != 0 {
                        pc = target as usize;
                    }
                }
                Op::JmpLtI { a, b, target } => {
                    if regs_i[a as usize] < regs_i[b as usize] {
                        pc = target as usize;
                    }
                }
                Op::JmpLeI { a, b, target } => {
                    if regs_i[a as usize] <= regs_i[b as usize] {
                        pc = target as usize;
                    }
                }
                Op::JmpGtI { a, b, target } => {
                    if regs_i[a as usize] > regs_i[b as usize] {
                        pc = target as usize;
                    }
                }
                Op::JmpGeI { a, b, target } => {
                    if regs_i[a as usize] >= regs_i[b as usize] {
                        pc = target as usize;
                    }
                }
                Op::JmpEqI { a, b, target } => {
                    if regs_i[a as usize] == regs_i[b as usize] {
                        pc = target as usize;
                    }
                }
                Op::JmpNeI { a, b, target } => {
                    if regs_i[a as usize] != regs_i[b as usize] {
                        pc = target as usize;
                    }
                }
                Op::ForInc { r_i, r_hi, target } => {
                    let next = regs_i[r_i as usize] + 1;
                    regs_i[r_i as usize] = next;
                    if next < regs_i[r_hi as usize] {
                        pc = target as usize;
                    }
                }
                Op::ForIncClosed { r_i, r_hi, target } => {
                    let cur = regs_i[r_i as usize];
                    if cur < regs_i[r_hi as usize] {
                        regs_i[r_i as usize] = cur + 1;
                        pc = target as usize;
                    }
                }
                Op::ForDec { r_i, r_lo, target } => {
                    let cur = regs_i[r_i as usize];
                    if cur > regs_i[r_lo as usize] {
                        regs_i[r_i as usize] = cur - 1;
                        pc = target as usize;
                    }
                }
                Op::LoadGArr1I { dst, gid, r_i } => {
                    let idx = regs_i[r_i as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {len}"),
                            );
                        }
                        regs_i[dst as usize] = read_array_i(arr, idx as usize);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreGArr1I { gid, r_i, r_val } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {len}"),
                            );
                        }
                        write_array_i(Rc::make_mut(arr), idx as usize, val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadGArr2I { dst, gid, r_i, r_j } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        regs_i[dst as usize] = read_array_i(arr, (i0 * l1 + i1) as usize);
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::StoreGArr2I { gid, r_i, r_j, r_val } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    let val = regs_i[r_val as usize];
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        write_array_i(Rc::make_mut(arr), (i0 * l1 + i1) as usize, val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::LoadLArr1I { dst, arr_v, r_i } => {
                    let idx = regs_i[r_i as usize];
                    match &regs_v[arr_v as usize] {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside an array dimension of length {len}"),
                                );
                            }
                            regs_i[dst as usize] = read_array_i(arr, idx as usize);
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            regs_i[dst as usize] = ScalarKind::unbox_val(&list[idx as usize]);
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array"),
                    }
                }
                Op::StoreLArr1I { arr_v, r_i, r_val } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    match &mut regs_v[arr_v as usize] {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside an array dimension of length {len}"),
                                );
                            }
                            write_array_i(Rc::make_mut(arr), idx as usize, val);
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            let slot = &mut Rc::make_mut(list)[idx as usize];
                            *slot = match slot {
                                Value::Bool(_) => Value::Bool(val != 0),
                                Value::Char(_) => Value::Char(val as u8),
                                Value::Float(_) => Value::Float(f64::from_bits(val as u64)),
                                Value::U64(_) => Value::U64(val as u64),
                                _ => Value::Int(val),
                            };
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array"),
                    }
                }
                Op::LoadLArr2I { dst, arr_v, r_i, r_j } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        regs_i[dst as usize] = read_array_i(arr, (i0 * l1 + i1) as usize);
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::StoreLArr2I { arr_v, r_i, r_j, r_val } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    let val = regs_i[r_val as usize];
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        write_array_i(Rc::make_mut(arr), (i0 * l1 + i1) as usize, val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::LoadGArr1OptI { dst_val, dst_some, gid, r_i } => {
                    let idx = regs_i[r_i as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {len}"),
                            );
                        }
                        let (v, s) = read_array_opt_i(arr, idx as usize);
                        regs_i[dst_val as usize] = v;
                        regs_i[dst_some as usize] = s;
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreGArr1OptI { gid, r_i, r_val, r_some, kind } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    let some = regs_i[r_some as usize];
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {len}"),
                            );
                        }
                        write_array_opt_i(Rc::make_mut(arr), idx as usize, val, some, kind);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadGArr2OptI { dst_val, dst_some, gid, r_i, r_j } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        let (v, s) = read_array_opt_i(arr, (i0 * l1 + i1) as usize);
                        regs_i[dst_val as usize] = v;
                        regs_i[dst_some as usize] = s;
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::StoreGArr2OptI { gid, r_i, r_j, r_val, r_some, kind } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    let val = regs_i[r_val as usize];
                    let some = regs_i[r_some as usize];
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        write_array_opt_i(Rc::make_mut(arr), (i0 * l1 + i1) as usize, val, some, kind);
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::LoadLArr1OptI { dst_val, dst_some, arr_v, r_i } => {
                    let idx = regs_i[r_i as usize];
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {len}"),
                            );
                        }
                        let (v, s) = read_array_opt_i(arr, idx as usize);
                        regs_i[dst_val as usize] = v;
                        regs_i[dst_some as usize] = s;
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreLArr1OptI { arr_v, r_i, r_val, r_some, kind } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    let some = regs_i[r_some as usize];
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {len}"),
                            );
                        }
                        write_array_opt_i(Rc::make_mut(arr), idx as usize, val, some, kind);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadLArr2OptI { dst_val, dst_some, arr_v, r_i, r_j } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        let (v, s) = read_array_opt_i(arr, (i0 * l1 + i1) as usize);
                        regs_i[dst_val as usize] = v;
                        regs_i[dst_some as usize] = s;
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::StoreLArr2OptI { arr_v, r_i, r_j, r_val, r_some, kind } => {
                    let i0 = regs_i[r_i as usize];
                    let i1 = regs_i[r_j as usize];
                    let val = regs_i[r_val as usize];
                    let some = regs_i[r_some as usize];
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let l0 = arr.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = arr.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        write_array_opt_i(Rc::make_mut(arr), (i0 * l1 + i1) as usize, val, some, kind);
                    } else {
                        return trap(spans[cur_pc], "internal: expected 2D array");
                    }
                }
                Op::LoadGList1I { dst, gid, r_i } => {
                    let idx = regs_i[r_i as usize];
                    match &self.globals[gid as usize] {
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            regs_i[dst as usize] = ScalarKind::unbox_val(&list[idx as usize]);
                        }
                        Value::ListView(view) => {
                            let len = view.len as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            regs_i[dst as usize] =
                                ScalarKind::unbox_val(view.get(idx as usize).unwrap());
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    }
                }
                Op::StoreGList1I { gid, r_i, r_val, kind } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    if let Value::List(list) = &mut self.globals[gid as usize] {
                        let len = list.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        Rc::make_mut(list)[idx as usize] = kind.box_val(val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected list");
                    }
                }
                Op::LoadLList1I { dst, list_v, r_i } => {
                    let idx = regs_i[r_i as usize];
                    match &regs_v[list_v as usize] {
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            regs_i[dst as usize] = ScalarKind::unbox_val(&list[idx as usize]);
                        }
                        Value::ListView(view) => {
                            let len = view.len as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            regs_i[dst as usize] =
                                ScalarKind::unbox_val(view.get(idx as usize).unwrap());
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    }
                }
                Op::StoreLList1I { list_v, r_i, r_val, kind } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    if let Value::List(list) = &mut regs_v[list_v as usize] {
                        let len = list.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        Rc::make_mut(list)[idx as usize] = kind.box_val(val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected list");
                    }
                }
                Op::GArrLen { dst, gid } => {
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        regs_i[dst as usize] = arr.len() as i64;
                    } else {
                        regs_i[dst as usize] = 0;
                    }
                }
                Op::LArrLen { dst, arr_v } => {
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
                        regs_i[dst as usize] = arr.len() as i64;
                    } else {
                        regs_i[dst as usize] = 0;
                    }
                }
                Op::GListLen { dst, gid } => {
                    regs_i[dst as usize] = match &self.globals[gid as usize] {
                        Value::List(list) => list.len() as i64,
                        Value::ListView(view) => view.len as i64,
                        _ => 0,
                    };
                }
                Op::LListLen { dst, list_v } => {
                    regs_i[dst as usize] = match &regs_v[list_v as usize] {
                        Value::List(list) => list.len() as i64,
                        Value::ListView(view) => view.len as i64,
                        _ => 0,
                    };
                }
                Op::LoadGArr1V { dst, gid, r_i } => {
                    let idx = regs_i[r_i as usize];
                    if let Value::Array(arr) = &self.globals[gid as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {len}"),
                            );
                        }
                        regs_v[dst as usize] = arr.get(idx as usize).unwrap_or(Value::Unit);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreGArr1V { gid, r_i, r_val } => {
                    let idx = regs_i[r_i as usize];
                    let val = if r_val as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                    } else {
                        regs_v[r_val as usize].clone()
                    };
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {len}"),
                            );
                        }
                        Rc::make_mut(arr).set(idx as usize, val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadLArr1V { dst, arr_v, r_i } => {
                    let idx = regs_i[r_i as usize];
                    match &regs_v[arr_v as usize] {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside an array dimension of length {len}"),
                                );
                            }
                            regs_v[dst as usize] = arr.get(idx as usize).unwrap_or(Value::Unit);
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            regs_v[dst as usize] = list[idx as usize].clone();
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array"),
                    }
                }
                Op::StoreLArr1V { arr_v, r_i, r_val } => {
                    let idx = regs_i[r_i as usize];
                    let val = if r_val as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                    } else {
                        regs_v[r_val as usize].clone()
                    };
                    match &mut regs_v[arr_v as usize] {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside an array dimension of length {len}"),
                                );
                            }
                            Rc::make_mut(arr).set(idx as usize, val);
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            Rc::make_mut(list)[idx as usize] = val;
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array"),
                    }
                }
                Op::FieldI { dst, obj_v, field } => {
                    regs_i[dst as usize] = match &regs_v[obj_v as usize] {
                        Value::Tuple(TupleVal::Boxed(fs)) => {
                            ScalarKind::unbox_val(&fs[field as usize])
                        }
                        Value::Tuple(TupleVal::OneInt(v)) => *v,
                        _ => 0,
                    };
                    if obj_v as usize >= chunk.num_local_v {
                        regs_v[obj_v as usize] = Value::Unit;
                    }
                }
                Op::FieldOptI { dst_val, dst_some, obj_v, field } => {
                    let (v, s) = match &regs_v[obj_v as usize] {
                        Value::Tuple(TupleVal::Boxed(fs)) => {
                            ScalarKind::unbox_opt(&fs[field as usize])
                        }
                        _ => (0, 0),
                    };
                    regs_i[dst_val as usize] = v;
                    regs_i[dst_some as usize] = s;
                    if obj_v as usize >= chunk.num_local_v {
                        regs_v[obj_v as usize] = Value::Unit;
                    }
                }
                Op::FieldV { dst, obj_v, field } => {
                    let val = match &regs_v[obj_v as usize] {
                        Value::Tuple(TupleVal::Boxed(fs)) => fs[field as usize].clone(),
                        Value::Tuple(TupleVal::OneInt(v)) => Value::Int(*v),
                        _ => Value::Unit,
                    };
                    if obj_v as usize >= chunk.num_local_v {
                        regs_v[obj_v as usize] = Value::Unit;
                    }
                    regs_v[dst as usize] = val;
                }
                Op::LoadFieldList1I { dst, obj_v, field, r_i } => {
                    let idx = regs_i[r_i as usize];
                    if let Value::Tuple(TupleVal::Boxed(fs)) = &regs_v[obj_v as usize] {
                        if let Value::List(list) = &fs[field as usize] {
                            if idx < 0 || (idx as usize) >= list.len() {
                                return trap(
                                    spans[cur_pc],
                                    format!(
                                        "index {idx} is outside a list of length {}",
                                        list.len()
                                    ),
                                );
                            }
                            regs_i[dst as usize] = ScalarKind::unbox_val(&list[idx as usize]);
                        } else {
                            return trap(spans[cur_pc], "internal: expected list field");
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected struct");
                    }
                }
                Op::FieldListLen { dst, obj_v, field } => {
                    regs_i[dst as usize] = match &regs_v[obj_v as usize] {
                        Value::Tuple(TupleVal::Boxed(fs)) => match &fs[field as usize] {
                            Value::List(list) => list.len() as i64,
                            _ => 0,
                        },
                        _ => 0,
                    };
                }
                Op::LoadFieldArr1I { dst, obj_v, field, r_i } => {
                    let idx = regs_i[r_i as usize];
                    if let Value::Tuple(TupleVal::Boxed(fs)) = &regs_v[obj_v as usize] {
                        if let Value::Array(arr) = &fs[field as usize] {
                            let len = arr.dims[0];
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!(
                                        "index {idx} is outside an array dimension of length {len}"
                                    ),
                                );
                            }
                            regs_i[dst as usize] = read_array_i(arr, idx as usize);
                        } else {
                            return trap(spans[cur_pc], "internal: expected array field");
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected struct");
                    }
                }
                Op::FieldArrLen { dst, obj_v, field } => {
                    regs_i[dst as usize] = match &regs_v[obj_v as usize] {
                        Value::Tuple(TupleVal::Boxed(fs)) => match &fs[field as usize] {
                            Value::Array(arr) => arr.dims.first().copied().unwrap_or(0),
                            _ => 0,
                        },
                        _ => 0,
                    };
                }
                Op::StrGetI { dst, str_v, r_i } => {
                    let idx = regs_i[r_i as usize];
                    match &regs_v[str_v as usize] {
                        Value::Str(s) => {
                            if idx < 0 || (idx as usize) >= s.len() {
                                return trap(
                                    spans[cur_pc],
                                    format!(
                                        "index {idx} is outside a string of length {}",
                                        s.len()
                                    ),
                                );
                            }
                            regs_i[dst as usize] = i64::from(s[idx as usize]);
                        }
                        Value::StrView(sv) => {
                            if idx < 0 || (idx as usize) >= sv.len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a string of length {}", sv.len),
                                );
                            }
                            let pos = if sv.rev {
                                sv.lo + sv.len - 1 - idx as usize
                            } else {
                                sv.lo + idx as usize
                            };
                            regs_i[dst as usize] = i64::from(sv.base[pos]);
                        }
                        _ => return trap(spans[cur_pc], "internal: expected string"),
                    }
                }
                Op::GStrGetI { dst, gid, r_i } => {
                    let idx = regs_i[r_i as usize];
                    match &self.globals[gid as usize] {
                        Value::Str(s) => {
                            if idx < 0 || (idx as usize) >= s.len() {
                                return trap(
                                    spans[cur_pc],
                                    format!(
                                        "index {idx} is outside a string of length {}",
                                        s.len()
                                    ),
                                );
                            }
                            regs_i[dst as usize] = i64::from(s[idx as usize]);
                        }
                        Value::StrView(sv) => {
                            if idx < 0 || (idx as usize) >= sv.len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a string of length {}", sv.len),
                                );
                            }
                            let pos = if sv.rev {
                                sv.lo + sv.len - 1 - idx as usize
                            } else {
                                sv.lo + idx as usize
                            };
                            regs_i[dst as usize] = i64::from(sv.base[pos]);
                        }
                        _ => return trap(spans[cur_pc], "internal: expected string"),
                    }
                }
                Op::MakeTuple { dst, base_v, len } => {
                    let b = base_v as usize;
                    let n = len as usize;
                    if n == 1 {
                        let v = std::mem::replace(&mut regs_v[b], Value::Unit);
                        regs_v[dst as usize] = match v {
                            Value::Int(x) => Value::Tuple(TupleVal::OneInt(x)),
                            other => Value::Tuple(TupleVal::Boxed(Rc::new(vec![other]))),
                        };
                    } else {
                        let mut fs = Vec::with_capacity(n);
                        for k in 0..n {
                            fs.push(std::mem::replace(&mut regs_v[b + k], Value::Unit));
                        }
                        regs_v[dst as usize] = Value::Tuple(TupleVal::Boxed(Rc::new(fs)));
                    }
                }
                Op::MakeList { dst, base_v, len } => {
                    let b = base_v as usize;
                    let n = len as usize;
                    let mut items = Vec::with_capacity(n);
                    for k in 0..n {
                        items.push(std::mem::replace(&mut regs_v[b + k], Value::Unit));
                    }
                    regs_v[dst as usize] = Value::List(Rc::new(items));
                }
                Op::ListFilled { dst, r_n, val_v } => {
                    let n = regs_i[r_n as usize];
                    if n < 0 {
                        return trap(spans[cur_pc], format!("List.filled with negative length {n}"));
                    }
                    let val = regs_v[val_v as usize].clone();
                    regs_v[dst as usize] = Value::List(Rc::new(vec![val; n as usize]));
                }
                Op::ToList { dst, src_v } => {
                    regs_v[dst as usize] = match &regs_v[src_v as usize] {
                        Value::Array(arr) => Value::List(Rc::new(arr.to_values())),
                        Value::List(items) => Value::List(Rc::new((**items).clone())),
                        Value::ListView(view) => Value::List(Rc::new(view.to_vec())),
                        other => other.clone(),
                    };
                }
                Op::MakeFill { dst, dims_base, ndims, fill_v, fill_ty } => {
                    let mut dims_vec = Vec::with_capacity(ndims as usize);
                    let mut cells: i64 = 1;
                    for k in 0..ndims as usize {
                        let len = regs_i[dims_base as usize + k];
                        if len < 0 {
                            return trap(
                                spans[cur_pc],
                                format!("an array can't have length {len}"),
                            );
                        }
                        cells = match cells.checked_mul(len).filter(|c| *c <= 1 << 32) {
                            Some(c) => c,
                            None => {
                                return trap(spans[cur_pc], "this array is too large to allocate");
                            }
                        };
                        dims_vec.push(len);
                    }
                    let cell = if fill_v == u16::MAX {
                        if fill_ty == u16::MAX {
                            Value::Unit
                        } else {
                            crate::zero(chunk.fill_tys[fill_ty as usize], self.program)
                        }
                    } else if fill_v as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[fill_v as usize], Value::Unit)
                    } else {
                        regs_v[fill_v as usize].clone()
                    };
                    regs_v[dst as usize] = Value::Array(Rc::new(Array::filled(dims_vec, cell)));
                }
                Op::ListSlice { dst, src_v, r_lo, r_hi } => {
                    let lo = regs_i[r_lo as usize];
                    let hi = regs_i[r_hi as usize];
                    let (view, len) = match &regs_v[src_v as usize] {
                        Value::List(items) => {
                            let n = items.len();
                            (crate::ListViewVal::whole(items.clone(), false), n)
                        }
                        Value::ListView(v) => {
                            let n = v.len;
                            ((**v).clone(), n)
                        }
                        _ => return trap(spans[cur_pc], "slice expects a list"),
                    };
                    if lo < 0 || hi < lo || hi as usize > len {
                        return trap(
                            spans[cur_pc],
                            format!("slice {lo}..{hi} is outside a list of length {len}"),
                        );
                    }
                    regs_v[dst as usize] =
                        Value::ListView(Rc::new(view.slice(lo as usize, hi as usize)));
                }
                Op::StrSlice { dst, src_v, r_lo, r_hi } => {
                    let lo = regs_i[r_lo as usize];
                    let hi = regs_i[r_hi as usize];
                    let (view, len) = match &regs_v[src_v as usize] {
                        Value::Str(bytes) => {
                            let n = bytes.len();
                            (crate::StrViewVal::whole(bytes.clone(), false), n)
                        }
                        Value::StrView(v) => {
                            let n = v.len;
                            ((**v).clone(), n)
                        }
                        _ => return trap(spans[cur_pc], "slice expects a string"),
                    };
                    if lo < 0 || hi < lo || hi as usize > len {
                        return trap(
                            spans[cur_pc],
                            format!("slice {lo}..{hi} is outside a string of length {len}"),
                        );
                    }
                    regs_v[dst as usize] =
                        Value::StrView(Rc::new(view.slice(lo as usize, hi as usize)));
                }
                Op::TakeLArr1V { dst, arr_v, r_i } => {
                    let idx = regs_i[r_i as usize];
                    let val = if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {len}"),
                            );
                        }
                        let a = Rc::make_mut(arr);
                        match &mut a.data {
                            ArrayData::Values(vs) => {
                                std::mem::replace(&mut vs[idx as usize], Value::Unit)
                            }
                            _ => a.get(idx as usize).unwrap_or(Value::Unit),
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    };
                    regs_v[dst as usize] = val;
                }
                Op::TakeGArr1V { dst, gid, r_i } => {
                    let idx = regs_i[r_i as usize];
                    let val = if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let len = arr.dims[0];
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {len}"),
                            );
                        }
                        let a = Rc::make_mut(arr);
                        match &mut a.data {
                            ArrayData::Values(vs) => {
                                std::mem::replace(&mut vs[idx as usize], Value::Unit)
                            }
                            _ => a.get(idx as usize).unwrap_or(Value::Unit),
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    };
                    regs_v[dst as usize] = val;
                }
                Op::TakeLList1V { dst, list_v, r_i } => {
                    let idx = regs_i[r_i as usize];
                    let val = if let Value::List(items) = &mut regs_v[list_v as usize] {
                        let len = items.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        std::mem::replace(&mut Rc::make_mut(items)[idx as usize], Value::Unit)
                    } else {
                        return trap(spans[cur_pc], "internal: expected list");
                    };
                    regs_v[dst as usize] = val;
                }
                Op::TakeGList1V { dst, gid, r_i } => {
                    let idx = regs_i[r_i as usize];
                    let val = if let Value::List(items) = &mut self.globals[gid as usize] {
                        let len = items.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        std::mem::replace(&mut Rc::make_mut(items)[idx as usize], Value::Unit)
                    } else {
                        return trap(spans[cur_pc], "internal: expected list");
                    };
                    regs_v[dst as usize] = val;
                }
                Op::LoadLArr2V { dst, arr_v, r_i, r_j } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    let val = if let Value::Array(arr) = &regs_v[arr_v as usize] {
                        let d0 = arr.dims[0];
                        let d1 = arr.dims[1];
                        if i < 0 || i >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array dimension of length {d0}"),
                            );
                        }
                        if j < 0 || j >= d1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {j} is outside an array dimension of length {d1}"),
                            );
                        }
                        arr.get((i * d1 + j) as usize).unwrap_or(Value::Unit)
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    };
                    regs_v[dst as usize] = val;
                }
                Op::LoadGArr2V { dst, gid, r_i, r_j } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    let val = if let Value::Array(arr) = &self.globals[gid as usize] {
                        let d0 = arr.dims[0];
                        let d1 = arr.dims[1];
                        if i < 0 || i >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array dimension of length {d0}"),
                            );
                        }
                        if j < 0 || j >= d1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {j} is outside an array dimension of length {d1}"),
                            );
                        }
                        arr.get((i * d1 + j) as usize).unwrap_or(Value::Unit)
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    };
                    regs_v[dst as usize] = val;
                }
                Op::StoreLArr2V { arr_v, r_i, r_j, r_val } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    let val = if r_val as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                    } else {
                        regs_v[r_val as usize].clone()
                    };
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let d0 = arr.dims[0];
                        let d1 = arr.dims[1];
                        if i < 0 || i >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array dimension of length {d0}"),
                            );
                        }
                        if j < 0 || j >= d1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {j} is outside an array dimension of length {d1}"),
                            );
                        }
                        Rc::make_mut(arr).set((i * d1 + j) as usize, val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::StoreGArr2V { gid, r_i, r_j, r_val } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    let val = if r_val as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                    } else {
                        regs_v[r_val as usize].clone()
                    };
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let d0 = arr.dims[0];
                        let d1 = arr.dims[1];
                        if i < 0 || i >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {i} is outside an array dimension of length {d0}"),
                            );
                        }
                        if j < 0 || j >= d1 {
                            return trap(
                                spans[cur_pc],
                                format!("index {j} is outside an array dimension of length {d1}"),
                            );
                        }
                        Rc::make_mut(arr).set((i * d1 + j) as usize, val);
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::ConstF64 { dst, idx } => {
                    regs_v[dst as usize] = Value::Float(f64::from_bits(chunk.consts_i[idx as usize] as u64));
                }
                Op::FloatBin { dst, a, b, op } => {
                    let (Value::Float(fa), Value::Float(fb)) =
                        (&regs_v[a as usize], &regs_v[b as usize])
                    else {
                        return trap(spans[cur_pc], "internal: expected float");
                    };
                    let res = match op {
                        BinOp::Add => *fa + *fb,
                        BinOp::Sub => *fa - *fb,
                        BinOp::Mul => *fa * *fb,
                        BinOp::Div => *fa / *fb,
                        BinOp::Pow => fa.powf(*fb),
                        _ => return trap(spans[cur_pc], "internal: unsupported float op"),
                    };
                    regs_v[dst as usize] = Value::Float(res);
                }
                Op::FloatMinMax { dst, a, b, is_max } => {
                    let (Value::Float(fa), Value::Float(fb)) =
                        (&regs_v[a as usize], &regs_v[b as usize])
                    else {
                        return trap(spans[cur_pc], "internal: expected float");
                    };
                    let res = if is_max {
                        if *fa >= *fb { *fa } else { *fb }
                    } else if *fa <= *fb {
                        *fa
                    } else {
                        *fb
                    };
                    regs_v[dst as usize] = Value::Float(res);
                }
                Op::CmpV { dst, a_v, b_v, op } => {
                    let a = &regs_v[a_v as usize];
                    let b = &regs_v[b_v as usize];
                    let res = match op {
                        CmpOp::Eq => a == b,
                        CmpOp::Ne => a != b,
                        CmpOp::Lt => a.cmp(b) == std::cmp::Ordering::Less,
                        CmpOp::Le => a.cmp(b) != std::cmp::Ordering::Greater,
                        CmpOp::Gt => a.cmp(b) == std::cmp::Ordering::Greater,
                        CmpOp::Ge => a.cmp(b) != std::cmp::Ordering::Less,
                        _ => false,
                    };
                    regs_i[dst as usize] = i64::from(res);
                }
                Op::CallVmI { fn_id, arg_base, argc, dst } => {
                    let f = &self.program.fns[fn_id as usize];
                    self.stack_left(spans[cur_pc], Some(&f.name))?;
                    let callee = compiled.fns[fn_id as usize].as_ref().unwrap();
                    let ab = arg_base as usize;
                    let mut arg_buf = [0i64; 8];
                    let n = argc as usize;
                    arg_buf[..n].copy_from_slice(&regs_i[ab..ab + n]);
                    let needs_stack = callee.needs_stack;
                    if chunk.has_by_ref_closure {
                        sync_to_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base, chunk.traced);
                    }
                    let saved_base = self.base;
                    if needs_stack {
                        self.base = self.stack.len();
                        self.stack.resize(self.base + f.locals.len(), Value::Unit);
                        self.frame_locals.push(&f.locals);
                    }
                    self.call_frames.push((f.name.as_str(), spans[cur_pc]));
                    let res = self.exec_chunk(compiled, callee, &arg_buf[..n], vm_regs_i);
                    if needs_stack {
                        self.frame_locals.pop();
                        self.stack.truncate(self.base);
                        self.base = saved_base;
                    }
                    let val = res?;
                    self.call_frames.pop();
                    regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                    if chunk.has_by_ref_closure {
                        sync_from_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base, chunk.traced);
                    }
                    regs_i[dst as usize] = ScalarKind::unbox_val(&val);
                }
                Op::CallVmMix {
                    fn_id,
                    arg_base_i,
                    arg_base_v,
                    dst_i,
                    dst_v,
                    dst_some,
                    ret_mode,
                } => {
                    let f = &self.program.fns[fn_id as usize];
                    self.stack_left(spans[cur_pc], Some(&f.name))?;
                    let callee = compiled.fns[fn_id as usize].as_ref().unwrap();
                    let mut ni = 0usize;
                    let mut nv = 0usize;
                    for loc in callee.slot_map.iter().take(callee.param_count) {
                        match loc {
                            SlotLoc::I(..) => ni += 1,
                            SlotLoc::OptI { .. } => ni += 2,
                            SlotLoc::V(_) => nv += 1,
                            SlotLoc::SmallTuple { .. } => {
                                unreachable!("params are never SmallTuple")
                            }
                        }
                    }
                    let mut arg_buf_i = [0i64; 8];
                    let abi = arg_base_i as usize;
                    arg_buf_i[..ni].copy_from_slice(&regs_i[abi..abi + ni]);
                    let mut arg_buf_v: [Value; 8] = std::array::from_fn(|_| Value::Unit);
                    let abv = arg_base_v as usize;
                    for k in 0..nv {
                        arg_buf_v[k] = std::mem::replace(&mut regs_v[abv + k], Value::Unit);
                    }
                    let needs_stack = callee.needs_stack;
                    if chunk.has_by_ref_closure {
                        sync_to_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base, chunk.traced);
                    }
                    let saved_base = self.base;
                    if needs_stack {
                        self.base = self.stack.len();
                        self.stack.resize(self.base + f.locals.len(), Value::Unit);
                        self.frame_locals.push(&f.locals);
                    }
                    self.call_frames.push((f.name.as_str(), spans[cur_pc]));
                    let res = self.exec_chunk_mix(
                        compiled,
                        callee,
                        &mut arg_buf_i[..ni],
                        &mut arg_buf_v[..nv],
                        &f.params,
                        vm_regs_i,
                    );
                    if needs_stack {
                        self.frame_locals.pop();
                        self.stack.truncate(self.base);
                        self.base = saved_base;
                    }
                    let val = res?;
                    self.call_frames.pop();
                    regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                    if chunk.has_by_ref_closure {
                        sync_from_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base, chunk.traced);
                    }
                    if f.params.iter().any(|p| p.mutable) {
                        regs_i[abi..abi + ni].copy_from_slice(&arg_buf_i[..ni]);
                        for k in 0..nv {
                            regs_v[abv + k] = std::mem::replace(&mut arg_buf_v[k], Value::Unit);
                        }
                    }
                    match ret_mode {
                        CallRet::Unit => {}
                        CallRet::I => regs_i[dst_i as usize] = ScalarKind::unbox_val(&val),
                        CallRet::OptI => {
                            let (v, s) = ScalarKind::unbox_opt(&val);
                            regs_i[dst_i as usize] = v;
                            regs_i[dst_some as usize] = s;
                        }
                        CallRet::V => regs_v[dst_v as usize] = val,
                    }
                }
                Op::CallClosure {
                    callee_v,
                    arg_base_i,
                    arg_base_v,
                    dst_i,
                    dst_v,
                    dst_some,
                    ret_mode,
                } => {
                    if let Value::Closure(cid, _) = &regs_v[callee_v as usize] {
                        if let Some(fast) = compiled.fast_closures[*cid] {
                            let abi = arg_base_i as usize;
                            let res_i = match fast {
                                FastClosure::Param(p) => regs_i[abi + p as usize],
                                FastClosure::Add(a, b) => {
                                    let x = regs_i[abi + a as usize];
                                    let y = regs_i[abi + b as usize];
                                    match x.checked_add(y) {
                                        Some(v) if v != i64::MIN => v,
                                        _ => {
                                            return trap(
                                                spans[cur_pc],
                                                format!("`{x} + {y}` overflows `Int`"),
                                            );
                                        }
                                    }
                                }
                                FastClosure::Sub(a, b) => {
                                    let x = regs_i[abi + a as usize];
                                    let y = regs_i[abi + b as usize];
                                    match x.checked_sub(y) {
                                        Some(v) if v != i64::MIN => v,
                                        _ => {
                                            return trap(
                                                spans[cur_pc],
                                                format!("`{x} - {y}` overflows `Int`"),
                                            );
                                        }
                                    }
                                }
                                FastClosure::Mul(a, b) => {
                                    let x = regs_i[abi + a as usize];
                                    let y = regs_i[abi + b as usize];
                                    match x.checked_mul(y) {
                                        Some(v) if v != i64::MIN => v,
                                        _ => {
                                            return trap(
                                                spans[cur_pc],
                                                format!("`{x} * {y}` overflows `Int`"),
                                            );
                                        }
                                    }
                                }
                                FastClosure::Min(a, b) => {
                                    regs_i[abi + a as usize].min(regs_i[abi + b as usize])
                                }
                                FastClosure::Max(a, b) => {
                                    regs_i[abi + a as usize].max(regs_i[abi + b as usize])
                                }
                                FastClosure::BitAnd(a, b) => {
                                    regs_i[abi + a as usize] & regs_i[abi + b as usize]
                                }
                                FastClosure::BitOr(a, b) => {
                                    regs_i[abi + a as usize] | regs_i[abi + b as usize]
                                }
                                FastClosure::BitXor(a, b) => {
                                    regs_i[abi + a as usize] ^ regs_i[abi + b as usize]
                                }
                                FastClosure::MulAdd(a, b, c) => {
                                    let x = regs_i[abi + a as usize];
                                    let y = regs_i[abi + b as usize];
                                    let z = regs_i[abi + c as usize];
                                    let prod = match x.checked_mul(y) {
                                        Some(v) if v != i64::MIN => v,
                                        _ => {
                                            return trap(
                                                spans[cur_pc],
                                                format!("`{x} * {y}` overflows `Int`"),
                                            );
                                        }
                                    };
                                    match prod.checked_add(z) {
                                        Some(v) if v != i64::MIN => v,
                                        _ => {
                                            return trap(
                                                spans[cur_pc],
                                                format!("`{prod} + {z}` overflows `Int`"),
                                            );
                                        }
                                    }
                                }
                            };
                            if callee_v as usize >= chunk.num_local_v {
                                regs_v[callee_v as usize] = Value::Unit;
                            }
                            match ret_mode {
                                CallRet::Unit => {}
                                CallRet::I => regs_i[dst_i as usize] = res_i,
                                CallRet::OptI => {
                                    regs_i[dst_i as usize] = res_i;
                                    regs_i[dst_some as usize] = 1;
                                }
                                CallRet::V => regs_v[dst_v as usize] = Value::Int(res_i),
                            }
                            continue;
                        }
                    }
                    let (cid, caps) = match &regs_v[callee_v as usize] {
                        Value::Closure(cid, caps) => (*cid, Rc::clone(caps)),
                        _ => return trap(spans[cur_pc], "internal: expected closure"),
                    };
                    if callee_v as usize >= chunk.num_local_v {
                        regs_v[callee_v as usize] = Value::Unit;
                    }
                    let def = &self.program.closures[cid];
                    let val = if let Some(callee) = compiled.closures[cid].as_ref() {
                        self.stack_left(spans[cur_pc], None)?;
                        let mut ni = 0usize;
                        let mut nv = 0usize;
                        for loc in callee.slot_map.iter().take(callee.param_count) {
                            match loc {
                                SlotLoc::I(..) => ni += 1,
                                SlotLoc::OptI { .. } => ni += 2,
                                SlotLoc::V(_) => nv += 1,
                                SlotLoc::SmallTuple { .. } => {
                                    unreachable!("params are never SmallTuple")
                                }
                            }
                        }
                        let mut arg_buf_i = [0i64; 8];
                        let abi = arg_base_i as usize;
                        arg_buf_i[..ni].copy_from_slice(&regs_i[abi..abi + ni]);
                        let mut arg_buf_v: [Value; 8] = std::array::from_fn(|_| Value::Unit);
                        let abv = arg_base_v as usize;
                        for k in 0..nv {
                            arg_buf_v[k] = std::mem::replace(&mut regs_v[abv + k], Value::Unit);
                        }
                        let needs_stack = callee.needs_stack;
                        let saved_base = self.base;
                        if needs_stack {
                            self.base = self.stack.len();
                            self.stack.resize(self.base + def.locals.len(), Value::Unit);
                            self.frame_locals.push(&def.locals);
                        }
                        self.call_frames.push(("<closure>", spans[cur_pc]));
                        let prev_in_closure = std::mem::replace(&mut self.in_closure, true);
                        let c_base_i = vm_regs_i.len();
                        vm_regs_i.resize(c_base_i + callee.num_i, 0);
                        let mut c_regs_v = self.take_regs_v(callee.num_v);
                        let mut ii = 0usize;
                        let mut iv = 0usize;
                        for loc in callee.slot_map.iter().take(callee.param_count) {
                            match *loc {
                                SlotLoc::I(r, _) => {
                                    vm_regs_i[c_base_i + r as usize] = arg_buf_i[ii];
                                    ii += 1;
                                }
                                SlotLoc::OptI { val, some, .. } => {
                                    vm_regs_i[c_base_i + val as usize] = arg_buf_i[ii];
                                    vm_regs_i[c_base_i + some as usize] = arg_buf_i[ii + 1];
                                    ii += 2;
                                }
                                SlotLoc::V(r) => {
                                    c_regs_v[r as usize] =
                                        std::mem::replace(&mut arg_buf_v[iv], Value::Unit);
                                    iv += 1;
                                }
                                SlotLoc::SmallTuple { .. } => {
                                    unreachable!("params are never SmallTuple")
                                }
                            }
                        }
                        for (cap, cval) in def.captures.iter().zip(caps.iter()) {
                            let crate::CaptureVal::Val(v) = cval else {
                                unreachable!("compiled closures only have value captures")
                            };
                            match callee.slot_map[cap.inner] {
                                SlotLoc::I(r, _) => {
                                    vm_regs_i[c_base_i + r as usize] = ScalarKind::unbox_val(v);
                                }
                                SlotLoc::OptI { val, some, .. } => {
                                    let (rv, rs) = ScalarKind::unbox_opt(v);
                                    vm_regs_i[c_base_i + val as usize] = rv;
                                    vm_regs_i[c_base_i + some as usize] = rs;
                                }
                                SlotLoc::SmallTuple { base: b, len, .. } => {
                                    if let Value::Tuple(TupleVal::Boxed(fs)) = v {
                                        for (k, f) in fs.iter().take(len as usize).enumerate() {
                                            vm_regs_i[c_base_i + b as usize + k] =
                                                ScalarKind::unbox_val(f);
                                        }
                                    }
                                }
                                SlotLoc::V(r) => c_regs_v[r as usize] = v.clone(),
                            }
                        }
                        let dom_mark = self.saved_domains.len();
                        for &d in &callee.guarded_domains {
                            self.saved_domains.push((d, self.domains[d]));
                        }
                        let res = self.exec_chunk_inner(
                            compiled,
                            callee,
                            c_base_i,
                            vm_regs_i,
                            &mut c_regs_v,
                        );
                        while self.saved_domains.len() > dom_mark {
                            let (id, prev) = self.saved_domains.pop().unwrap();
                            self.domains[id] = prev;
                        }
                        self.return_regs_v(c_regs_v);
                        vm_regs_i.truncate(c_base_i);
                        self.in_closure = prev_in_closure;
                        if needs_stack {
                            self.frame_locals.pop();
                            self.stack.truncate(self.base);
                            self.base = saved_base;
                        }
                        let v = res?;
                        self.call_frames.pop();
                        regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                        v
                    } else {
                        let mut boxed = Vec::with_capacity(def.params.len());
                        let mut ii = arg_base_i as usize;
                        let mut iv = arg_base_v as usize;
                        for &pslot in &def.params {
                            let ty = &def.locals[pslot].ty;
                            if let Some(kind) = ScalarKind::of_ty(ty, self.program) {
                                boxed.push(kind.box_val(regs_i[ii]));
                                ii += 1;
                            } else if let Some(kind) = ScalarKind::of_opt_ty(ty, self.program) {
                                boxed.push(kind.box_opt(regs_i[ii], regs_i[ii + 1]));
                                ii += 2;
                            } else {
                                boxed.push(std::mem::replace(&mut regs_v[iv], Value::Unit));
                                iv += 1;
                            }
                        }
                        let has_frame = chunk.needs_stack;
                        if has_frame {
                            sync_to_stack(
                                &chunk.slot_map,
                                regs_i,
                                regs_v,
                                &mut self.stack,
                                self.base, chunk.traced
                            );
                        }
                        let res = self.call_value(Value::Closure(cid, caps), boxed, spans[cur_pc]);
                        regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                        if has_frame {
                            sync_from_stack(
                                &chunk.slot_map,
                                regs_i,
                                regs_v,
                                &mut self.stack,
                                self.base, chunk.traced
                            );
                        }
                        res?
                    };
                    match ret_mode {
                        CallRet::Unit => {}
                        CallRet::I => regs_i[dst_i as usize] = ScalarKind::unbox_val(&val),
                        CallRet::OptI => {
                            let (v, s) = ScalarKind::unbox_opt(&val);
                            regs_i[dst_i as usize] = v;
                            regs_i[dst_some as usize] = s;
                        }
                        CallRet::V => regs_v[dst_v as usize] = val,
                    }
                }
                Op::RetI { src } => {
                    let raw = regs_i[src as usize];
                    let kind = chunk.ret_kind.unwrap_or(ScalarKind::Int);
                    return Ok(ExecExit::Return(kind.box_val(raw)));
                }
                Op::RetV { src } => {
                    return Ok(ExecExit::Return(std::mem::replace(
                        &mut regs_v[src as usize],
                        Value::Unit,
                    )));
                }
                Op::RetUnit => return Ok(ExecExit::Return(Value::Unit)),
                Op::IsSomeV { dst_i, src_v } => {
                    regs_i[dst_i as usize] =
                        i64::from(!matches!(regs_v[src_v as usize], Value::Opt(OptVal::None)));
                }
                Op::UnwrapV { dst_v, src_v } => {
                    let v = match &regs_v[src_v as usize] {
                        Value::Opt(OptVal::Int(n)) => Value::Int(*n),
                        Value::Opt(OptVal::Boxed(b)) => (**b).clone(),
                        Value::Opt(OptVal::None) => {
                            return trap(spans[cur_pc], "unwrapped a `None` value");
                        }
                        other => other.clone(),
                    };
                    if src_v as usize >= chunk.num_local_v {
                        regs_v[src_v as usize] = Value::Unit;
                    }
                    regs_v[dst_v as usize] = v;
                }
                Op::UnwrapVI { dst_i, src_v } => {
                    let v = match &regs_v[src_v as usize] {
                        Value::Opt(OptVal::Int(n)) => *n,
                        Value::Opt(OptVal::Boxed(b)) => ScalarKind::unbox_val(b),
                        Value::Opt(OptVal::None) => {
                            return trap(spans[cur_pc], "unwrapped a `None` value");
                        }
                        other => ScalarKind::unbox_val(other),
                    };
                    if src_v as usize >= chunk.num_local_v {
                        regs_v[src_v as usize] = Value::Unit;
                    }
                    regs_i[dst_i as usize] = v;
                }
                Op::WrapSomeV { dst_v, src_v } => {
                    let inner = if (src_v as usize) >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[src_v as usize], Value::Unit)
                    } else {
                        regs_v[src_v as usize].clone()
                    };
                    regs_v[dst_v as usize] = Value::Opt(OptVal::some(inner));
                }
                Op::ConstNoneV { dst_v } => {
                    regs_v[dst_v as usize] = Value::Opt(OptVal::None);
                }
                Op::SeqLastV { dst_v, seq_v } => {
                    let opt = match &regs_v[seq_v as usize] {
                        Value::List(items) => items.last().cloned(),
                        Value::ListView(view) => {
                            view.len.checked_sub(1).and_then(|i| view.get(i)).cloned()
                        }
                        Value::Str(bytes) => bytes.last().copied().map(Value::Char),
                        Value::StrView(view) => {
                            view.len.checked_sub(1).and_then(|i| view.get(i)).map(Value::Char)
                        }
                        Value::Array(arr) => arr.last(),
                        Value::Range(lo, hi) => (lo < hi).then_some(Value::Int(*hi - 1)),
                        Value::Range128(lo, hi) => (lo < hi).then_some(Value::I128(*hi - 1)),
                        _ => None,
                    };
                    if seq_v as usize >= chunk.num_local_v {
                        regs_v[seq_v as usize] = Value::Unit;
                    }
                    regs_v[dst_v as usize] = Value::Opt(OptVal::from_option(opt));
                }
                Op::SeqLastOptI { dst_val, dst_some, seq_v } => {
                    let (v, s) = match &regs_v[seq_v as usize] {
                        Value::List(items) => match items.last() {
                            Some(x) => (ScalarKind::unbox_val(x), 1),
                            None => (0, 0),
                        },
                        Value::ListView(view) => {
                            match view.len.checked_sub(1).and_then(|i| view.get(i)) {
                                Some(x) => (ScalarKind::unbox_val(x), 1),
                                None => (0, 0),
                            }
                        }
                        Value::Str(bytes) => match bytes.last() {
                            Some(&b) => (i64::from(b), 1),
                            None => (0, 0),
                        },
                        Value::StrView(view) => {
                            match view.len.checked_sub(1).and_then(|i| view.get(i)) {
                                Some(b) => (i64::from(b), 1),
                                None => (0, 0),
                            }
                        }
                        Value::Array(arr) => match arr.last() {
                            Some(x) => (ScalarKind::unbox_val(&x), 1),
                            None => (0, 0),
                        },
                        Value::Range(lo, hi) if lo < hi => (*hi - 1, 1),
                        _ => (0, 0),
                    };
                    if seq_v as usize >= chunk.num_local_v {
                        regs_v[seq_v as usize] = Value::Unit;
                    }
                    regs_i[dst_val as usize] = v;
                    regs_i[dst_some as usize] = s;
                }
                Op::SeqFirstV { dst_v, seq_v } => {
                    let opt = match &regs_v[seq_v as usize] {
                        Value::List(items) => items.first().cloned(),
                        Value::ListView(view) => view.get(0).cloned(),
                        Value::Str(bytes) => bytes.first().copied().map(Value::Char),
                        Value::StrView(view) => view.get(0).map(Value::Char),
                        Value::Array(arr) => arr.first(),
                        Value::Range(lo, hi) => (lo < hi).then_some(Value::Int(*lo)),
                        Value::Range128(lo, hi) => (lo < hi).then_some(Value::I128(*lo)),
                        _ => None,
                    };
                    if seq_v as usize >= chunk.num_local_v {
                        regs_v[seq_v as usize] = Value::Unit;
                    }
                    regs_v[dst_v as usize] = Value::Opt(OptVal::from_option(opt));
                }
                Op::SeqFirstOptI { dst_val, dst_some, seq_v } => {
                    let (v, s) = match &regs_v[seq_v as usize] {
                        Value::List(items) => match items.first() {
                            Some(x) => (ScalarKind::unbox_val(x), 1),
                            None => (0, 0),
                        },
                        Value::ListView(view) => match view.get(0) {
                            Some(x) => (ScalarKind::unbox_val(x), 1),
                            None => (0, 0),
                        },
                        Value::Str(bytes) => match bytes.first() {
                            Some(&b) => (i64::from(b), 1),
                            None => (0, 0),
                        },
                        Value::StrView(view) => match view.get(0) {
                            Some(b) => (i64::from(b), 1),
                            None => (0, 0),
                        },
                        Value::Array(arr) => match arr.first() {
                            Some(x) => (ScalarKind::unbox_val(&x), 1),
                            None => (0, 0),
                        },
                        Value::Range(lo, hi) if lo < hi => (*lo, 1),
                        _ => (0, 0),
                    };
                    if seq_v as usize >= chunk.num_local_v {
                        regs_v[seq_v as usize] = Value::Unit;
                    }
                    regs_i[dst_val as usize] = v;
                    regs_i[dst_some as usize] = s;
                }
                Op::SeqLenI { dst, seq_v } => {
                    regs_i[dst as usize] = match &regs_v[seq_v as usize] {
                        Value::Array(arr) => arr.len() as i64,
                        Value::List(items) => items.len() as i64,
                        Value::ListView(view) => view.len as i64,
                        Value::Str(bytes) => bytes.len() as i64,
                        Value::StrView(view) => view.len as i64,
                        Value::Map(m) => m.len() as i64,
                        Value::Set(s) => s.len() as i64,
                        Value::Range(lo, hi) => (hi - lo).max(0),
                        Value::Range128(lo, hi) if lo >= hi => 0,
                        Value::Range128(lo, hi) => {
                            match hi.checked_sub(*lo).and_then(|d| i64::try_from(d).ok()) {
                                Some(d) => d,
                                None => {
                                    return trap(
                                        spans[cur_pc],
                                        format!("integer overflow: range `{lo}..{hi}` length does not fit in an `Int` (64 bits)"),
                                    );
                                }
                            }
                        }
                        Value::DomainVal(_, len) => *len,
                        _ => 0,
                    };
                    if seq_v as usize >= chunk.num_local_v {
                        regs_v[seq_v as usize] = Value::Unit;
                    }
                }
                Op::RangeLoI { dst, src_v } => {
                    regs_i[dst as usize] = match &regs_v[src_v as usize] {
                        Value::Range(lo, _) => *lo,
                        _ => 0,
                    };
                    if src_v as usize >= chunk.num_local_v {
                        regs_v[src_v as usize] = Value::Unit;
                    }
                }
                Op::RangeHiI { dst, src_v } => {
                    regs_i[dst as usize] = match &regs_v[src_v as usize] {
                        Value::Range(_, hi) => *hi,
                        _ => 0,
                    };
                    if src_v as usize >= chunk.num_local_v {
                        regs_v[src_v as usize] = Value::Unit;
                    }
                }
                Op::BitCeilI { dst, src } => {
                    let x = regs_i[src as usize];
                    regs_i[dst as usize] = if x <= 1 {
                        1
                    } else {
                        let shift = 64 - ((x - 1) as u64).leading_zeros();
                        if shift > 62 {
                            return trap(
                                spans[cur_pc],
                                format!(
                                    "integer overflow: `bit_ceil({x})` does not fit in an `Int` (64 bits)"
                                ),
                            );
                        }
                        1i64 << shift
                    };
                }
                Op::PopcountI { dst, src } => {
                    regs_i[dst as usize] = i64::from((regs_i[src as usize] as u64).count_ones());
                }
                Op::CtzI { dst, src } => {
                    regs_i[dst as usize] =
                        i64::from((regs_i[src as usize] as u64).trailing_zeros());
                }
                Op::ClzI { dst, src } => {
                    regs_i[dst as usize] =
                        i64::from((regs_i[src as usize] as u64).leading_zeros());
                }
                Op::MinMaxGArr1SmallTuple { gid, r_i, src_base, len, is_max } => {
                    let idx = regs_i[r_i as usize];
                    let cand = &regs_i[src_base as usize..src_base as usize + len as usize];
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let d0 = arr.dims[0];
                        if idx < 0 || idx >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {d0}"),
                            );
                        }
                        let a = Rc::make_mut(arr);
                        let n = len as usize;
                        if let ArrayData::SmallIntTuple { stride, data, .. } = &mut a.data {
                            let off = idx as usize * (*stride as usize);
                            let cur = &mut data[off..off + n];
                            let better = if is_max { cand > cur } else { cand < cur };
                            if better {
                                cur.copy_from_slice(cand);
                            }
                        } else {
                            let vs = a.as_mut_values();
                            let mut cur = [0i64; 4];
                            if let Value::Tuple(TupleVal::Boxed(fs)) = &vs[idx as usize] {
                                for k in 0..n {
                                    cur[k] = ScalarKind::unbox_val(&fs[k]);
                                }
                            }
                            let better = if is_max { cand > &cur[..n] } else { cand < &cur[..n] };
                            if better {
                                if let Value::Tuple(TupleVal::Boxed(fs)) = &mut vs[idx as usize] {
                                    let fs = Rc::make_mut(fs);
                                    for k in 0..n {
                                        fs[k] = Value::Int(cand[k]);
                                    }
                                }
                            }
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::MinMaxLArr1SmallTuple { arr_v, r_i, src_base, len, is_max } => {
                    let idx = regs_i[r_i as usize];
                    let cand = &regs_i[src_base as usize..src_base as usize + len as usize];
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let d0 = arr.dims[0];
                        if idx < 0 || idx >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {d0}"),
                            );
                        }
                        let a = Rc::make_mut(arr);
                        let n = len as usize;
                        if let ArrayData::SmallIntTuple { stride, data, .. } = &mut a.data {
                            let off = idx as usize * (*stride as usize);
                            let cur = &mut data[off..off + n];
                            let better = if is_max { cand > cur } else { cand < cur };
                            if better {
                                cur.copy_from_slice(cand);
                            }
                        } else {
                            let vs = a.as_mut_values();
                            let mut cur = [0i64; 4];
                            if let Value::Tuple(TupleVal::Boxed(fs)) = &vs[idx as usize] {
                                for k in 0..n {
                                    cur[k] = ScalarKind::unbox_val(&fs[k]);
                                }
                            }
                            let better = if is_max { cand > &cur[..n] } else { cand < &cur[..n] };
                            if better {
                                if let Value::Tuple(TupleVal::Boxed(fs)) = &mut vs[idx as usize] {
                                    let fs = Rc::make_mut(fs);
                                    for k in 0..n {
                                        fs[k] = Value::Int(cand[k]);
                                    }
                                }
                            }
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::ListPushGArr1I { gid, r_i, val_i, kind } => {
                    let idx = regs_i[r_i as usize];
                    let v = kind.box_val(regs_i[val_i as usize]);
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let d0 = arr.dims[0];
                        if idx < 0 || idx >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {d0}"),
                            );
                        }
                        if let Value::List(list) = &mut Rc::make_mut(arr).as_mut_values()[idx as usize] {
                            Rc::make_mut(list).push(v);
                        } else {
                            return trap(spans[cur_pc], "internal: expected list element");
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::ListPushGArr1V { gid, r_i, val_v } => {
                    let idx = regs_i[r_i as usize];
                    let v = if val_v as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[val_v as usize], Value::Unit)
                    } else {
                        regs_v[val_v as usize].clone()
                    };
                    if let Value::Array(arr) = &mut self.globals[gid as usize] {
                        let d0 = arr.dims[0];
                        if idx < 0 || idx >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {d0}"),
                            );
                        }
                        if let Value::List(list) = &mut Rc::make_mut(arr).as_mut_values()[idx as usize] {
                            Rc::make_mut(list).push(v);
                        } else {
                            return trap(spans[cur_pc], "internal: expected list element");
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::ListPushLArr1I { arr_v, r_i, val_i, kind } => {
                    let idx = regs_i[r_i as usize];
                    let v = kind.box_val(regs_i[val_i as usize]);
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let d0 = arr.dims[0];
                        if idx < 0 || idx >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {d0}"),
                            );
                        }
                        if let Value::List(list) = &mut Rc::make_mut(arr).as_mut_values()[idx as usize] {
                            Rc::make_mut(list).push(v);
                        } else {
                            return trap(spans[cur_pc], "internal: expected list element");
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::ListPushLArr1V { arr_v, r_i, val_v } => {
                    let idx = regs_i[r_i as usize];
                    let v = if val_v as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[val_v as usize], Value::Unit)
                    } else {
                        regs_v[val_v as usize].clone()
                    };
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
                        let d0 = arr.dims[0];
                        if idx < 0 || idx >= d0 {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside an array dimension of length {d0}"),
                            );
                        }
                        if let Value::List(list) = &mut Rc::make_mut(arr).as_mut_values()[idx as usize] {
                            Rc::make_mut(list).push(v);
                        } else {
                            return trap(spans[cur_pc], "internal: expected list element");
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::ListPushLV { list_v, val_v } => {
                    let item = if (val_v as usize) >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[val_v as usize], Value::Unit)
                    } else {
                        regs_v[val_v as usize].clone()
                    };
                    match &mut regs_v[list_v as usize] {
                        Value::List(items) => Rc::make_mut(items).push(item),
                        Value::Str(bytes) => {
                            if let Value::Char(c) = item {
                                Rc::make_mut(bytes).push(c);
                            }
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list or string"),
                    }
                }
                Op::ListPushGV { gid, val_v } => {
                    let item = if (val_v as usize) >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[val_v as usize], Value::Unit)
                    } else {
                        regs_v[val_v as usize].clone()
                    };
                    match &mut self.globals[gid as usize] {
                        Value::List(items) => Rc::make_mut(items).push(item),
                        Value::Str(bytes) => {
                            if let Value::Char(c) = item {
                                Rc::make_mut(bytes).push(c);
                            }
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list or string"),
                    }
                }
                Op::ListPushLI { list_v, val_i, kind } => {
                    let raw = regs_i[val_i as usize];
                    match &mut regs_v[list_v as usize] {
                        Value::List(items) => Rc::make_mut(items).push(kind.box_val(raw)),
                        Value::Str(bytes) => Rc::make_mut(bytes).push(raw as u8),
                        _ => return trap(spans[cur_pc], "internal: expected list or string"),
                    }
                }
                Op::ListPushGI { gid, val_i, kind } => {
                    let raw = regs_i[val_i as usize];
                    match &mut self.globals[gid as usize] {
                        Value::List(items) => Rc::make_mut(items).push(kind.box_val(raw)),
                        Value::Str(bytes) => Rc::make_mut(bytes).push(raw as u8),
                        _ => return trap(spans[cur_pc], "internal: expected list or string"),
                    }
                }
                Op::ListPopLV { dst_v, list_v } => {
                    let popped = match &mut regs_v[list_v as usize] {
                        Value::List(items) => Rc::make_mut(items).pop(),
                        Value::Str(bytes) => Rc::make_mut(bytes).pop().map(Value::Char),
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    };
                    regs_v[dst_v as usize] = Value::Opt(OptVal::from_option(popped));
                }
                Op::ListPopGV { dst_v, gid } => {
                    let popped = match &mut self.globals[gid as usize] {
                        Value::List(items) => Rc::make_mut(items).pop(),
                        Value::Str(bytes) => Rc::make_mut(bytes).pop().map(Value::Char),
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    };
                    regs_v[dst_v as usize] = Value::Opt(OptVal::from_option(popped));
                }
                Op::ListPopOptIL { dst_val, dst_some, list_v } => {
                    let popped = match &mut regs_v[list_v as usize] {
                        Value::List(items) => Rc::make_mut(items).pop(),
                        Value::Str(bytes) => Rc::make_mut(bytes).pop().map(Value::Char),
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    };
                    match popped {
                        Some(x) => {
                            regs_i[dst_val as usize] = ScalarKind::unbox_val(&x);
                            regs_i[dst_some as usize] = 1;
                        }
                        None => {
                            regs_i[dst_val as usize] = 0;
                            regs_i[dst_some as usize] = 0;
                        }
                    }
                }
                Op::ListPopOptIG { dst_val, dst_some, gid } => {
                    let popped = match &mut self.globals[gid as usize] {
                        Value::List(items) => Rc::make_mut(items).pop(),
                        Value::Str(bytes) => Rc::make_mut(bytes).pop().map(Value::Char),
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    };
                    match popped {
                        Some(x) => {
                            regs_i[dst_val as usize] = ScalarKind::unbox_val(&x);
                            regs_i[dst_some as usize] = 1;
                        }
                        None => {
                            regs_i[dst_val as usize] = 0;
                            regs_i[dst_some as usize] = 0;
                        }
                    }
                }
                Op::ListPopDiscardL { list_v } => match &mut regs_v[list_v as usize] {
                    Value::List(items) => {
                        Rc::make_mut(items).pop();
                    }
                    Value::Str(bytes) => {
                        Rc::make_mut(bytes).pop();
                    }
                    _ => return trap(spans[cur_pc], "internal: expected list"),
                },
                Op::ListPopDiscardG { gid } => match &mut self.globals[gid as usize] {
                    Value::List(items) => {
                        Rc::make_mut(items).pop();
                    }
                    Value::Str(bytes) => {
                        Rc::make_mut(bytes).pop();
                    }
                    _ => return trap(spans[cur_pc], "internal: expected list"),
                },
                Op::LoadGlobalVMove { dst, gid } => {
                    regs_v[dst as usize] =
                        std::mem::replace(&mut self.globals[gid as usize], Value::Unit);
                }
                Op::StoreLFieldI { obj_v, field, src_i, kind } => {
                    let val = kind.box_val(regs_i[src_i as usize]);
                    match &mut regs_v[obj_v as usize] {
                        Value::Tuple(TupleVal::Boxed(fs)) => {
                            Rc::make_mut(fs)[field as usize] = val;
                        }
                        Value::Tuple(TupleVal::OneInt(v)) if field == 0 => {
                            *v = regs_i[src_i as usize];
                        }
                        _ => return trap(spans[cur_pc], "internal: expected tuple/struct"),
                    }
                }
                Op::StoreLFieldV { obj_v, field, src_v } => {
                    let val = if (src_v as usize) >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[src_v as usize], Value::Unit)
                    } else {
                        regs_v[src_v as usize].clone()
                    };
                    match &mut regs_v[obj_v as usize] {
                        Value::Tuple(TupleVal::Boxed(fs)) => {
                            Rc::make_mut(fs)[field as usize] = val;
                        }
                        _ => return trap(spans[cur_pc], "internal: expected tuple/struct"),
                    }
                }
                Op::StoreLFieldOptI { obj_v, field, val, some, kind } => {
                    let boxed = kind.box_opt(regs_i[val as usize], regs_i[some as usize]);
                    match &mut regs_v[obj_v as usize] {
                        Value::Tuple(TupleVal::Boxed(fs)) => {
                            Rc::make_mut(fs)[field as usize] = boxed;
                        }
                        _ => return trap(spans[cur_pc], "internal: expected tuple/struct"),
                    }
                }
                Op::StoreLFieldIdx1I { obj_v, field, r_i, r_val, kind } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                        match &mut Rc::make_mut(fs)[field as usize] {
                            Value::Array(arr) => {
                                let len = arr.dims[0];
                                if idx < 0 || idx >= len {
                                    return trap(
                                        spans[cur_pc],
                                        format!("index {idx} is outside an array of length {len}"),
                                    );
                                }
                                write_array_i(Rc::make_mut(arr), idx as usize, val);
                            }
                            Value::List(list) => {
                                let len = list.len() as i64;
                                if idx < 0 || idx >= len {
                                    return trap(
                                        spans[cur_pc],
                                        format!("index {idx} is outside a list of length {len}"),
                                    );
                                }
                                Rc::make_mut(list)[idx as usize] = kind.box_val(val);
                            }
                            _ => return trap(spans[cur_pc], "internal: expected array or list"),
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected tuple/struct");
                    }
                }
                Op::StoreLFieldIdx1V { obj_v, field, r_i, r_val } => {
                    let idx = regs_i[r_i as usize];
                    let val = if (r_val as usize) >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                    } else {
                        regs_v[r_val as usize].clone()
                    };
                    if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                        match &mut Rc::make_mut(fs)[field as usize] {
                            Value::Array(arr) => {
                                let len = arr.dims[0];
                                if idx < 0 || idx >= len {
                                    return trap(
                                        spans[cur_pc],
                                        format!("index {idx} is outside an array of length {len}"),
                                    );
                                }
                                Rc::make_mut(arr).set(idx as usize, val);
                            }
                            Value::List(list) => {
                                let len = list.len() as i64;
                                if idx < 0 || idx >= len {
                                    return trap(
                                        spans[cur_pc],
                                        format!("index {idx} is outside a list of length {len}"),
                                    );
                                }
                                Rc::make_mut(list)[idx as usize] = val;
                            }
                            _ => return trap(spans[cur_pc], "internal: expected array or list"),
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected tuple/struct");
                    }
                }
                Op::StoreLFieldIdx1OptI { obj_v, field, r_i, val, some, kind } => {
                    let idx = regs_i[r_i as usize];
                    let v = regs_i[val as usize];
                    let s = regs_i[some as usize];
                    if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                        match &mut Rc::make_mut(fs)[field as usize] {
                            Value::Array(arr) => {
                                let len = arr.dims[0];
                                if idx < 0 || idx >= len {
                                    return trap(
                                        spans[cur_pc],
                                        format!("index {idx} is outside an array of length {len}"),
                                    );
                                }
                                write_array_opt_i(Rc::make_mut(arr), idx as usize, v, s, kind);
                            }
                            Value::List(list) => {
                                let len = list.len() as i64;
                                if idx < 0 || idx >= len {
                                    return trap(
                                        spans[cur_pc],
                                        format!("index {idx} is outside a list of length {len}"),
                                    );
                                }
                                Rc::make_mut(list)[idx as usize] = kind.box_opt(v, s);
                            }
                            _ => return trap(spans[cur_pc], "internal: expected array or list"),
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected tuple/struct");
                    }
                }
                Op::ListPushFieldLI { obj_v, field, val_i, kind } => {
                    let raw = regs_i[val_i as usize];
                    if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                        match &mut Rc::make_mut(fs)[field as usize] {
                            Value::List(items) => Rc::make_mut(items).push(kind.box_val(raw)),
                            Value::Str(bytes) => Rc::make_mut(bytes).push(raw as u8),
                            _ => return trap(spans[cur_pc], "internal: expected list or string"),
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected tuple/struct");
                    }
                }
                Op::ListPushFieldLV { obj_v, field, val_v } => {
                    let item = if (val_v as usize) >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[val_v as usize], Value::Unit)
                    } else {
                        regs_v[val_v as usize].clone()
                    };
                    if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                        match &mut Rc::make_mut(fs)[field as usize] {
                            Value::List(items) => Rc::make_mut(items).push(item),
                            Value::Str(bytes) => {
                                if let Value::Char(c) = item {
                                    Rc::make_mut(bytes).push(c);
                                }
                            }
                            _ => return trap(spans[cur_pc], "internal: expected list or string"),
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected tuple/struct");
                    }
                }
                Op::ListPopDiscardFieldL { obj_v, field } => {
                    if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                        match &mut Rc::make_mut(fs)[field as usize] {
                            Value::List(items) => {
                                Rc::make_mut(items).pop();
                            }
                            _ => return trap(spans[cur_pc], "internal: expected list"),
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected tuple/struct");
                    }
                }
                Op::StoreGList1V { gid, r_i, r_val } => {
                    let idx = regs_i[r_i as usize];
                    let val = if (r_val as usize) >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                    } else {
                        regs_v[r_val as usize].clone()
                    };
                    if let Value::List(list) = &mut self.globals[gid as usize] {
                        let len = list.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        Rc::make_mut(list)[idx as usize] = val;
                    } else {
                        return trap(spans[cur_pc], "internal: expected list");
                    }
                }
                Op::StoreLList1V { list_v, r_i, r_val } => {
                    let idx = regs_i[r_i as usize];
                    let val = if (r_val as usize) >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                    } else {
                        regs_v[r_val as usize].clone()
                    };
                    if let Value::List(list) = &mut regs_v[list_v as usize] {
                        let len = list.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        Rc::make_mut(list)[idx as usize] = val;
                    } else {
                        return trap(spans[cur_pc], "internal: expected list");
                    }
                }
                Op::LoadGList1V { dst, gid, r_i } => {
                    let idx = regs_i[r_i as usize];
                    if let Value::List(list) = &self.globals[gid as usize] {
                        let len = list.len() as i64;
                        if idx < 0 || idx >= len {
                            return trap(
                                spans[cur_pc],
                                format!("index {idx} is outside a list of length {len}"),
                            );
                        }
                        regs_v[dst as usize] = list[idx as usize].clone();
                    } else {
                        return trap(spans[cur_pc], "internal: expected list");
                    }
                }
                Op::LoadLList1V { dst, list_v, r_i } => {
                    let idx = regs_i[r_i as usize];
                    match &regs_v[list_v as usize] {
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            regs_v[dst as usize] = list[idx as usize].clone();
                        }
                        Value::ListView(view) => {
                            let len = view.len as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            regs_v[dst as usize] =
                                view.get(idx as usize).cloned().unwrap_or(Value::Unit);
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    }
                }
                Op::LoadLList1OptI { dst_val, dst_some, list_v, r_i } => {
                    let idx = regs_i[r_i as usize];
                    let (v, s) = match &regs_v[list_v as usize] {
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            ScalarKind::unbox_opt(&list[idx as usize])
                        }
                        Value::ListView(view) => {
                            let len = view.len as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            ScalarKind::unbox_opt(view.get(idx as usize).unwrap_or(&Value::Unit))
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    };
                    regs_i[dst_val as usize] = v;
                    regs_i[dst_some as usize] = s;
                }
                Op::ListPopFieldOptIL { dst_val, dst_some, obj_v, field } => {
                    if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                        if let Value::List(items) = &mut Rc::make_mut(fs)[field as usize] {
                            match Rc::make_mut(items).pop() {
                                Some(v) => {
                                    regs_i[dst_val as usize] = ScalarKind::unbox_val(&v);
                                    regs_i[dst_some as usize] = 1;
                                }
                                None => {
                                    regs_i[dst_val as usize] = 0;
                                    regs_i[dst_some as usize] = 0;
                                }
                            }
                        } else {
                            return trap(spans[cur_pc], "internal: expected list");
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected tuple/struct");
                    }
                }
                Op::ListPopFieldLV { dst_v, obj_v, field } => {
                    if let Value::Tuple(TupleVal::Boxed(fs)) = &mut regs_v[obj_v as usize] {
                        if let Value::List(items) = &mut Rc::make_mut(fs)[field as usize] {
                            regs_v[dst_v as usize] =
                                Value::Opt(OptVal::from_option(Rc::make_mut(items).pop()));
                        } else {
                            return trap(spans[cur_pc], "internal: expected list");
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected tuple/struct");
                    }
                }
                Op::MakeRange { dst, r_lo, r_hi } => {
                    let lo = regs_i[r_lo as usize];
                    let hi = regs_i[r_hi as usize];
                    if hi > lo && hi.checked_sub(lo).is_none() {
                        return trap(
                            spans[cur_pc],
                            format!(
                                "integer overflow: range `{lo}..{hi}` span does not fit in an `Int` (64 bits)"
                            ),
                        );
                    }
                    regs_v[dst as usize] = Value::Range(lo, hi);
                }
                Op::CmpOptI { dst, a_val, a_some, b_val, b_some, op } => {
                    let a = if regs_i[a_some as usize] != 0 {
                        Some(regs_i[a_val as usize])
                    } else {
                        None
                    };
                    let b = if regs_i[b_some as usize] != 0 {
                        Some(regs_i[b_val as usize])
                    } else {
                        None
                    };
                    let res = match op {
                        CmpOp::Eq => a == b,
                        CmpOp::Ne => a != b,
                        CmpOp::Lt => a < b,
                        CmpOp::Le => a <= b,
                        CmpOp::Gt => a > b,
                        CmpOp::Ge => a >= b,
                        _ => false,
                    };
                    regs_i[dst as usize] = i64::from(res);
                }
                Op::LoadGIdxFieldI { dst, gid, r_i, field } => {
                    let idx = regs_i[r_i as usize];
                    let v = match &self.globals[gid as usize] {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside an array dimension of length {len}"),
                                );
                            }
                            match &arr.data {
                                ArrayData::SmallIntTuple { stride, data, .. } => {
                                    data[idx as usize * (*stride as usize) + field as usize]
                                }
                                ArrayData::Values(vs) => match &vs[idx as usize] {
                                    Value::Tuple(TupleVal::Boxed(fs)) => {
                                        ScalarKind::unbox_val(&fs[field as usize])
                                    }
                                    Value::Tuple(TupleVal::OneInt(x)) => *x,
                                    _ => 0,
                                },
                                _ => 0,
                            }
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            match &list[idx as usize] {
                                Value::Tuple(TupleVal::Boxed(fs)) => {
                                    ScalarKind::unbox_val(&fs[field as usize])
                                }
                                Value::Tuple(TupleVal::OneInt(x)) => *x,
                                _ => 0,
                            }
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array or list"),
                    };
                    regs_i[dst as usize] = v;
                }
                Op::StoreGIdxFieldI { gid, r_i, field, r_val, kind } => {
                    let idx = regs_i[r_i as usize];
                    let raw = regs_i[r_val as usize];
                    match &mut self.globals[gid as usize] {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside an array dimension of length {len}"),
                                );
                            }
                            let a = Rc::make_mut(arr);
                            match &mut a.data {
                                ArrayData::SmallIntTuple { stride, data, .. } => {
                                    data[idx as usize * (*stride as usize) + field as usize] = raw;
                                }
                                ArrayData::Values(vs) => match &mut vs[idx as usize] {
                                    Value::Tuple(TupleVal::Boxed(fs)) => {
                                        Rc::make_mut(fs)[field as usize] = kind.box_val(raw);
                                    }
                                    Value::Tuple(TupleVal::OneInt(x)) => *x = raw,
                                    _ => return trap(spans[cur_pc], "internal: expected tuple"),
                                },
                                _ => return trap(spans[cur_pc], "internal: expected tuple array"),
                            }
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            match &mut Rc::make_mut(list)[idx as usize] {
                                Value::Tuple(TupleVal::Boxed(fs)) => {
                                    Rc::make_mut(fs)[field as usize] = kind.box_val(raw);
                                }
                                Value::Tuple(TupleVal::OneInt(x)) => *x = raw,
                                _ => return trap(spans[cur_pc], "internal: expected tuple"),
                            }
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array or list"),
                    }
                }
                Op::LoadLIdxFieldI { dst, seq_v, r_i, field } => {
                    let idx = regs_i[r_i as usize];
                    let v = match &regs_v[seq_v as usize] {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside an array dimension of length {len}"),
                                );
                            }
                            match &arr.data {
                                ArrayData::SmallIntTuple { stride, data, .. } => {
                                    data[idx as usize * (*stride as usize) + field as usize]
                                }
                                ArrayData::Values(vs) => match &vs[idx as usize] {
                                    Value::Tuple(TupleVal::Boxed(fs)) => {
                                        ScalarKind::unbox_val(&fs[field as usize])
                                    }
                                    Value::Tuple(TupleVal::OneInt(x)) => *x,
                                    _ => 0,
                                },
                                _ => 0,
                            }
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            match &list[idx as usize] {
                                Value::Tuple(TupleVal::Boxed(fs)) => {
                                    ScalarKind::unbox_val(&fs[field as usize])
                                }
                                Value::Tuple(TupleVal::OneInt(x)) => *x,
                                _ => 0,
                            }
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array or list"),
                    };
                    regs_i[dst as usize] = v;
                }
                Op::StoreLIdxFieldI { seq_v, r_i, field, r_val, kind } => {
                    let idx = regs_i[r_i as usize];
                    let raw = regs_i[r_val as usize];
                    match &mut regs_v[seq_v as usize] {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside an array dimension of length {len}"),
                                );
                            }
                            let a = Rc::make_mut(arr);
                            match &mut a.data {
                                ArrayData::SmallIntTuple { stride, data, .. } => {
                                    data[idx as usize * (*stride as usize) + field as usize] = raw;
                                }
                                ArrayData::Values(vs) => match &mut vs[idx as usize] {
                                    Value::Tuple(TupleVal::Boxed(fs)) => {
                                        Rc::make_mut(fs)[field as usize] = kind.box_val(raw);
                                    }
                                    Value::Tuple(TupleVal::OneInt(x)) => *x = raw,
                                    _ => return trap(spans[cur_pc], "internal: expected tuple"),
                                },
                                _ => return trap(spans[cur_pc], "internal: expected tuple array"),
                            }
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if idx < 0 || idx >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {len}"),
                                );
                            }
                            match &mut Rc::make_mut(list)[idx as usize] {
                                Value::Tuple(TupleVal::Boxed(fs)) => {
                                    Rc::make_mut(fs)[field as usize] = kind.box_val(raw);
                                }
                                Value::Tuple(TupleVal::OneInt(x)) => *x = raw,
                                _ => return trap(spans[cur_pc], "internal: expected tuple"),
                            }
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array or list"),
                    }
                }
                Op::LStrLen { dst, str_v } => {
                    regs_i[dst as usize] = match &regs_v[str_v as usize] {
                        Value::Str(bytes) => bytes.len() as i64,
                        Value::StrView(view) => view.len as i64,
                        _ => 0,
                    };
                }
                Op::BinU64 { dst, a, b, op } => {
                    regs_i[dst as usize] =
                        exec_bin_u64(regs_i[a as usize], regs_i[b as usize], op, spans[cur_pc])?;
                }
                Op::NegU64 { dst, a } => {
                    regs_i[dst as usize] = (regs_i[a as usize] as u64).wrapping_neg() as i64;
                }
                Op::BitAndU64 { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize] & regs_i[b as usize];
                }
                Op::BitOrU64 { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize] | regs_i[b as usize];
                }
                Op::BitXorU64 { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize] ^ regs_i[b as usize];
                }
                Op::BitNotI { dst, a } => {
                    regs_i[dst as usize] = !regs_i[a as usize];
                }
                Op::CmpU64 { dst, a, b, op } => {
                    let av = regs_i[a as usize] as u64;
                    let bv = regs_i[b as usize] as u64;
                    let res = match op {
                        CmpOp::Eq => av == bv,
                        CmpOp::Ne => av != bv,
                        CmpOp::Lt => av < bv,
                        CmpOp::Le => av <= bv,
                        CmpOp::Gt => av > bv,
                        CmpOp::Ge => av >= bv,
                        _ => false,
                    };
                    regs_i[dst as usize] = i64::from(res);
                }
                Op::U64ToInt { dst, src } => {
                    let v = regs_i[src as usize] as u64;
                    match i64::try_from(v) {
                        Ok(iv) => regs_i[dst as usize] = iv,
                        Err(_) => {
                            return trap(spans[cur_pc], format!("{v} does not fit in an `Int`"));
                        }
                    }
                }
                Op::StorePlaceI { path, r_val, kind } => {
                    let pp = &chunk.places[path as usize];
                    let raw = regs_i[r_val as usize];
                    let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                    let (&last, prefix) = pp.steps.split_last().expect("non-empty place path");
                    let container = walk_place_mut(root, prefix, regs_i, spans[cur_pc])?;
                    store_place_leaf_scalar(container, last, raw, kind, regs_i, spans[cur_pc])?;
                }
                Op::StorePlaceOptI { path, r_val, r_some, kind } => {
                    let pp = &chunk.places[path as usize];
                    let raw = regs_i[r_val as usize];
                    let some = regs_i[r_some as usize];
                    let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                    let (&last, prefix) = pp.steps.split_last().expect("non-empty place path");
                    let container = walk_place_mut(root, prefix, regs_i, spans[cur_pc])?;
                    store_place_leaf_opt_scalar(
                        container,
                        last,
                        raw,
                        some,
                        kind,
                        regs_i,
                        spans[cur_pc],
                    )?;
                }
                Op::StorePlaceV { path, r_val } => {
                    let pp = &chunk.places[path as usize];
                    let val = regs_v[r_val as usize].clone();
                    let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                    let (&last, prefix) = pp.steps.split_last().expect("non-empty place path");
                    let container = walk_place_mut(root, prefix, regs_i, spans[cur_pc])?;
                    store_place_leaf_val(container, last, val, regs_i, spans[cur_pc])?;
                }
                Op::AssignPlaceI { path, r_rhs, op, kind } => {
                    let pp = &chunk.places[path as usize];
                    let rhs = regs_i[r_rhs as usize];
                    let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                    let (&last, prefix) = pp.steps.split_last().expect("non-empty place path");
                    let container = walk_place_mut(root, prefix, regs_i, spans[cur_pc])?;
                    assign_place_leaf_scalar(
                        container,
                        last,
                        rhs,
                        op,
                        kind,
                        regs_i,
                        spans[cur_pc],
                    )?;
                }
                Op::ListPopDiscardPlace { path } => {
                    let pp = &chunk.places[path as usize];
                    let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                    let target = walk_place_mut(root, &pp.steps, regs_i, spans[cur_pc])?;
                    match target {
                        Value::List(items) => {
                            Rc::make_mut(items).pop();
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    }
                }
                Op::ListPushPlaceI { path, val_i, kind } => {
                    let pp = &chunk.places[path as usize];
                    let val = kind.box_val(regs_i[val_i as usize]);
                    let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                    let target = walk_place_mut(root, &pp.steps, regs_i, spans[cur_pc])?;
                    match target {
                        Value::List(items) => {
                            Rc::make_mut(items).push(val);
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    }
                }
                Op::ListPushPlaceV { path, val_v } => {
                    let pp = &chunk.places[path as usize];
                    let val = regs_v[val_v as usize].clone();
                    let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                    let target = walk_place_mut(root, &pp.steps, regs_i, spans[cur_pc])?;
                    match target {
                        Value::List(items) => {
                            Rc::make_mut(items).push(val);
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    }
                }
                Op::SortPlace { path, desc_r } => {
                    let pp = &chunk.places[path as usize];
                    let desc = desc_r != u16::MAX && regs_i[desc_r as usize] != 0;
                    let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                    let target = walk_place_mut(root, &pp.steps, regs_i, spans[cur_pc])?;
                    exec_sort_place(target, desc, spans[cur_pc])?;
                }
                Op::RangeCheckI { dst, val, lo, hi, text_idx } => {
                    let v = regs_i[val as usize];
                    let lo_v = regs_i[lo as usize];
                    let hi_v = regs_i[hi as usize];
                    if v < lo_v || v > hi_v {
                        let text = chunk.range_texts[text_idx as usize];
                        return trap(spans[cur_pc], format!("{v} is outside `{text}`"));
                    }
                    regs_i[dst as usize] = v;
                }
                Op::ConstUnit { dst } => {
                    regs_v[dst as usize] = Value::Unit;
                }
                Op::SetMinMaxV { dst, src_v, is_max } => {
                    regs_v[dst as usize] = match &regs_v[src_v as usize] {
                        Value::Set(items) => {
                            let item = if is_max { items.last() } else { items.first() };
                            Value::Opt(OptVal::from_option(item.cloned()))
                        }
                        Value::Map(entries) => {
                            let entry = if is_max {
                                entries.iter().next_back()
                            } else {
                                entries.iter().next()
                            };
                            Value::Opt(OptVal::from_option(
                                entry.map(|(k, v)| Value::tuple(vec![k.clone(), v.clone()])),
                            ))
                        }
                        _ => Value::Opt(OptVal::None),
                    };
                }
                Op::ReversePlace { path } => {
                    let pp = &chunk.places[path as usize];
                    let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                    let target = walk_place_mut(root, &pp.steps, regs_i, spans[cur_pc])?;
                    match target {
                        Value::List(items) => Rc::make_mut(items).reverse(),
                        Value::Array(arr) => Rc::make_mut(arr).reverse(),
                        Value::Str(bytes) => Rc::make_mut(bytes).reverse(),
                        _ => return trap(spans[cur_pc], "internal: cannot reverse this value"),
                    }
                }
                Op::BitWidthI { dst, src } => {
                    regs_i[dst as usize] =
                        i64::from(64 - (regs_i[src as usize] as u64).leading_zeros());
                }
                Op::IntToFloat { dst, src } => {
                    regs_v[dst as usize] = Value::Float(regs_i[src as usize] as f64);
                }
                Op::FloatSqrt { dst, src } => {
                    let x = regs_v[src as usize].float();
                    if x < 0.0 {
                        return trap(spans[cur_pc], format!("sqrt({x}) of a negative number"));
                    }
                    regs_v[dst as usize] = Value::Float(x.sqrt());
                }
                Op::ContainsV { dst, src_v, key_v } => {
                    let needle = &regs_v[key_v as usize];
                    let found = match &regs_v[src_v as usize] {
                        Value::Set(items) => items.contains(needle),
                        Value::Map(entries) => entries.contains_key(needle),
                        Value::List(items) => items.contains(needle),
                        Value::ListView(view) => view.contains(needle),
                        Value::Array(arr) => arr.contains(needle),
                        _ => false,
                    };
                    regs_i[dst as usize] = i64::from(found);
                }
                Op::MapGetV { dst, map_v, key_v } => {
                    let found = match &regs_v[map_v as usize] {
                        Value::Map(entries) => entries.get(&regs_v[key_v as usize]).cloned(),
                        _ => None,
                    };
                    regs_v[dst as usize] = Value::Opt(OptVal::from_option(found));
                }
                Op::MapIndexV { dst, map_v, key_v, key_ty } => {
                    let found = match &regs_v[map_v as usize] {
                        Value::Map(entries) => entries.get(&regs_v[key_v as usize]).cloned(),
                        _ => None,
                    };
                    if let Some(v) = found {
                        regs_v[dst as usize] = v;
                    } else {
                        let mut key = Vec::new();
                        super::show(
                            &mut key,
                            &regs_v[key_v as usize],
                            chunk.fill_tys[key_ty as usize],
                            6,
                            self.program,
                            &self.domains,
                        );
                        let key: String = key.iter().map(|b| *b as char).collect();
                        return trap(
                            spans[cur_pc],
                            format!(
                                "the key `{key}` is not in the map; `m.get(k)` gives an optional"
                            ),
                        );
                    }
                }
                Op::SetMapMutPlace { dst, path, val_v, is_remove } => {
                    let arg = regs_v[val_v as usize].clone();
                    let pp = &chunk.places[path as usize];
                    let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                    let target = walk_place_mut(root, &pp.steps, regs_i, spans[cur_pc])?;
                    let ok = match (is_remove, target) {
                        (false, Value::Set(items)) => Rc::make_mut(items).insert(arg),
                        (true, Value::Set(items)) => Rc::make_mut(items).remove(&arg),
                        (true, Value::Map(entries)) => Rc::make_mut(entries).remove(&arg).is_some(),
                        _ => {
                            return trap(
                                spans[cur_pc],
                                "internal: this value can't be changed that way",
                            );
                        }
                    };
                    regs_i[dst as usize] = i64::from(ok);
                }
                Op::MapPlaceAssign { path, key_v, val_v, op, val_ty } => {
                    let key = regs_v[key_v as usize].clone();
                    let value = regs_v[val_v as usize].clone();
                    let pp = &chunk.places[path as usize];
                    let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                    let target = walk_place_mut(root, &pp.steps, regs_i, spans[cur_pc])?;
                    if let Value::Map(entries) = target {
                        let map = Rc::make_mut(entries);
                        if op == AssignOp::Set {
                            map.insert(key, value);
                        } else {
                            let next = if let Some(old) = map.get(&key).cloned() {
                                super::combine(op, old, value, spans[cur_pc])?
                            } else {
                                match op {
                                    AssignOp::Max
                                    | AssignOp::Min
                                    | AssignOp::Binary(
                                        BinOp::Add
                                        | BinOp::Mul
                                        | BinOp::BitOr
                                        | BinOp::BitXor
                                        | BinOp::BitAnd,
                                    ) => value,
                                    AssignOp::Binary(BinOp::Sub) => super::combine(
                                        op,
                                        super::zero(chunk.fill_tys[val_ty as usize], self.program),
                                        value,
                                        spans[cur_pc],
                                    )?,
                                    _ => {
                                        return trap(
                                            spans[cur_pc],
                                            "this key is not in the map; store with `m[k] = v` first",
                                        );
                                    }
                                }
                            };
                            map.insert(key, next);
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected map");
                    }
                }
                Op::SetMapBoundV { dst, src_v, key_v, mode } => {
                    use std::ops::Bound::{Excluded, Included, Unbounded};
                    let key = regs_v[key_v as usize].clone();
                    let range = match mode {
                        0 => (Included(key), Unbounded),
                        1 => (Excluded(key), Unbounded),
                        2 => (Unbounded, Included(key)),
                        _ => (Unbounded, Excluded(key)),
                    };
                    let first_in = mode <= 1;
                    regs_v[dst as usize] = match &regs_v[src_v as usize] {
                        Value::Set(items) => {
                            let mut found = items.range(range);
                            let item = if first_in { found.next() } else { found.next_back() };
                            Value::Opt(OptVal::from_option(item.cloned()))
                        }
                        Value::Map(entries) => {
                            let mut found = entries.range(range);
                            let entry = if first_in { found.next() } else { found.next_back() };
                            Value::Opt(OptVal::from_option(
                                entry.map(|(k, v)| Value::tuple(vec![k.clone(), v.clone()])),
                            ))
                        }
                        _ => Value::Opt(OptVal::None),
                    };
                }
                Op::SetMapPopEndPlace { dst, path, is_last } => {
                    let pp = &chunk.places[path as usize];
                    let root = place_root_mut(pp.root, regs_v, &mut self.globals);
                    let target = walk_place_mut(root, &pp.steps, regs_i, spans[cur_pc])?;
                    regs_v[dst as usize] = match target {
                        Value::Set(items) => {
                            let items = Rc::make_mut(items);
                            let v = if is_last { items.pop_last() } else { items.pop_first() };
                            Value::Opt(OptVal::from_option(v))
                        }
                        Value::Map(entries) => {
                            let entries = Rc::make_mut(entries);
                            let v =
                                if is_last { entries.pop_last() } else { entries.pop_first() };
                            Value::Opt(OptVal::from_option(
                                v.map(|(k, v)| Value::tuple(vec![k, v])),
                            ))
                        }
                        _ => {
                            return trap(
                                spans[cur_pc],
                                "internal: this value can't be changed that way",
                            );
                        }
                    };
                }
                Op::LoadList1SmallTuple { dst_base, len, root, is_global, r_i } => {
                    let idx = regs_i[r_i as usize];
                    let list_val = if is_global {
                        &self.globals[root as usize]
                    } else {
                        &regs_v[root as usize]
                    };
                    let elem = match list_val {
                        Value::List(list) => {
                            let n_len = list.len() as i64;
                            if idx < 0 || idx >= n_len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {n_len}"),
                                );
                            }
                            &list[idx as usize]
                        }
                        Value::ListView(view) => {
                            let n_len = view.len as i64;
                            if idx < 0 || idx >= n_len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {idx} is outside a list of length {n_len}"),
                                );
                            }
                            view.get(idx as usize).unwrap()
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    };
                    let b = dst_base as usize;
                    let n = len as usize;
                    if let Value::Tuple(TupleVal::Boxed(fs)) = elem {
                        for (k, f) in fs.iter().take(n).enumerate() {
                            regs_i[b + k] = ScalarKind::unbox_val(f);
                        }
                    }
                }
                Op::MinMaxArrOptI { root, is_global, is_2d, is_max, r_i, r_j, r_val } => {
                    let i0 = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    let target = if is_global {
                        &mut self.globals[root as usize]
                    } else {
                        &mut regs_v[root as usize]
                    };
                    if let Value::Array(arr) = target {
                        if is_2d && arr.dims.len() == 1 {
                            let l0 = arr.dims[0];
                            if i0 < 0 || i0 >= l0 {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {i0} is outside an array of length {l0}"),
                                );
                            }
                            let i1 = regs_i[r_j as usize];
                            let arr_mut = Rc::make_mut(arr);
                            let ArrayData::Values(rows) = &mut arr_mut.data else {
                                return trap(spans[cur_pc], "internal: expected nested array");
                            };
                            let Value::Array(row_arr) = &mut rows[i0 as usize] else {
                                return trap(spans[cur_pc], "internal: expected inner array");
                            };
                            let l1 = row_arr.dims[0];
                            if i1 < 0 || i1 >= l1 {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {i1} is outside an array of length {l1}"),
                                );
                            }
                            let row_mut = Rc::make_mut(row_arr);
                            match &mut row_mut.data {
                                ArrayData::OptInt(v) => {
                                    let slot = unsafe { v.get_unchecked_mut(i1 as usize) };
                                    let replace = match *slot {
                                        None => true,
                                        Some(cur) => {
                                            if is_max { val > cur } else { val < cur }
                                        }
                                    };
                                    if replace {
                                        *slot = Some(val);
                                    }
                                }
                                _ => {
                                    let (cur, some) = read_array_opt_i(row_mut, i1 as usize);
                                    if some == 0 || (if is_max { val > cur } else { val < cur }) {
                                        write_array_opt_i(
                                            row_mut,
                                            i1 as usize,
                                            val,
                                            1,
                                            ScalarKind::Int,
                                        );
                                    }
                                }
                            }
                            continue;
                        }
                        let flat = if is_2d {
                            let l0 = arr.dims[0];
                            if i0 < 0 || i0 >= l0 {
                                return trap(
                                    spans[cur_pc],
                                    format!(
                                        "index {i0} is outside an array dimension of length {l0}"
                                    ),
                                );
                            }
                            let i1 = regs_i[r_j as usize];
                            let l1 = arr.dims[1];
                            if i1 < 0 || i1 >= l1 {
                                return trap(
                                    spans[cur_pc],
                                    format!(
                                        "index {i1} is outside an array dimension of length {l1}"
                                    ),
                                );
                            }
                            (i0 * l1 + i1) as usize
                        } else {
                            let l0 = arr.dims[0];
                            if i0 < 0 || i0 >= l0 {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {i0} is outside an array of length {l0}"),
                                );
                            }
                            i0 as usize
                        };
                        let arr_mut = Rc::make_mut(arr);
                        match &mut arr_mut.data {
                            ArrayData::OptInt(v) => {
                                let slot = unsafe { v.get_unchecked_mut(flat) };
                                let replace = match *slot {
                                    None => true,
                                    Some(cur) => {
                                        if is_max { val > cur } else { val < cur }
                                    }
                                };
                                if replace {
                                    *slot = Some(val);
                                }
                            }
                            _ => {
                                let (cur, some) = read_array_opt_i(arr_mut, flat);
                                if some == 0 || (if is_max { val > cur } else { val < cur }) {
                                    write_array_opt_i(arr_mut, flat, val, 1, ScalarKind::Int);
                                }
                            }
                        }
                    } else {
                        return trap(spans[cur_pc], "internal: expected array");
                    }
                }
                Op::LoadIdxFieldIdxOptI {
                    dst_val,
                    dst_some,
                    root,
                    is_global,
                    r_i,
                    field,
                    r_j,
                } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    let seq_val = if is_global {
                        &self.globals[root as usize]
                    } else {
                        &regs_v[root as usize]
                    };
                    let elem = match seq_val {
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if i < 0 || i >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {i} is outside a list of length {len}"),
                                );
                            }
                            &list[i as usize]
                        }
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if i < 0 || i >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {i} is outside an array of length {len}"),
                                );
                            }
                            match &arr.data {
                                ArrayData::Values(v) => &v[i as usize],
                                _ => return trap(spans[cur_pc], "internal: expected struct element"),
                            }
                        }
                        _ => return trap(spans[cur_pc], "internal: expected list or array"),
                    };
                    let field_val = match elem {
                        Value::Tuple(TupleVal::Boxed(fs)) => &fs[field as usize],
                        _ => return trap(spans[cur_pc], "internal: expected struct/tuple"),
                    };
                    let (v, s) = match field_val {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if j < 0 || j >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {j} is outside an array of length {len}"),
                                );
                            }
                            read_array_opt_i(arr, j as usize)
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if j < 0 || j >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {j} is outside a list of length {len}"),
                                );
                            }
                            match &list[j as usize] {
                                Value::Opt(OptVal::Int(n)) => (*n, 1),
                                Value::Opt(OptVal::None) => (0, 0),
                                Value::Opt(OptVal::Boxed(b)) => (ScalarKind::unbox_val(b), 1),
                                _ => (0, 0),
                            }
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array or list field"),
                    };
                    regs_i[dst_val as usize] = v;
                    regs_i[dst_some as usize] = s;
                }
                Op::LoadNestedArrOptI { dst_val, dst_some, root, is_global, r_i, r_j } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    let outer_val = if is_global {
                        &self.globals[root as usize]
                    } else {
                        &regs_v[root as usize]
                    };
                    let inner_val = match outer_val {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if i < 0 || i >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {i} is outside an array of length {len}"),
                                );
                            }
                            match &arr.data {
                                ArrayData::Values(v) => &v[i as usize],
                                _ => return trap(spans[cur_pc], "internal: expected nested array"),
                            }
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if i < 0 || i >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {i} is outside a list of length {len}"),
                                );
                            }
                            &list[i as usize]
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array or list"),
                    };
                    let (v, s) = match inner_val {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if j < 0 || j >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {j} is outside an array of length {len}"),
                                );
                            }
                            read_array_opt_i(arr, j as usize)
                        }
                        _ => return trap(spans[cur_pc], "internal: expected inner array"),
                    };
                    regs_i[dst_val as usize] = v;
                    regs_i[dst_some as usize] = s;
                }
                Op::LoadNestedArrI { dst, root, is_global, r_i, r_j } => {
                    let i = regs_i[r_i as usize];
                    let j = regs_i[r_j as usize];
                    let outer_val = if is_global {
                        &self.globals[root as usize]
                    } else {
                        &regs_v[root as usize]
                    };
                    let inner_val = match outer_val {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if i < 0 || i >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {i} is outside an array of length {len}"),
                                );
                            }
                            match &arr.data {
                                ArrayData::Values(v) => &v[i as usize],
                                _ => return trap(spans[cur_pc], "internal: expected nested array"),
                            }
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if i < 0 || i >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {i} is outside a list of length {len}"),
                                );
                            }
                            &list[i as usize]
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array or list"),
                    };
                    regs_i[dst as usize] = match inner_val {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if j < 0 || j >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {j} is outside an array of length {len}"),
                                );
                            }
                            read_array_i(arr, j as usize)
                        }
                        _ => return trap(spans[cur_pc], "internal: expected inner array"),
                    };
                }
                Op::SeqIdxEndOptI { dst_val, dst_some, root, is_global, r_i, is_last } => {
                    let i = regs_i[r_i as usize];
                    let outer_val = if is_global {
                        &self.globals[root as usize]
                    } else {
                        &regs_v[root as usize]
                    };
                    let inner_val = match outer_val {
                        Value::Array(arr) => {
                            let len = arr.dims[0];
                            if i < 0 || i >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {i} is outside an array of length {len}"),
                                );
                            }
                            match &arr.data {
                                ArrayData::Values(v) => &v[i as usize],
                                _ => return trap(spans[cur_pc], "internal: expected sequence element"),
                            }
                        }
                        Value::List(list) => {
                            let len = list.len() as i64;
                            if i < 0 || i >= len {
                                return trap(
                                    spans[cur_pc],
                                    format!("index {i} is outside a list of length {len}"),
                                );
                            }
                            &list[i as usize]
                        }
                        _ => return trap(spans[cur_pc], "internal: expected array or list"),
                    };
                    let (v, s) = match inner_val {
                        Value::List(list) => {
                            let opt = if is_last { list.last() } else { list.first() };
                            match opt {
                                Some(el) => (ScalarKind::unbox_val(el), 1),
                                None => (0, 0),
                            }
                        }
                        Value::Array(arr) => {
                            let len = arr.dims[0] as usize;
                            if len == 0 {
                                (0, 0)
                            } else {
                                let at = if is_last { len - 1 } else { 0 };
                                (read_array_i(arr, at), 1)
                            }
                        }
                        _ => (0, 0),
                    };
                    regs_i[dst_val as usize] = v;
                    regs_i[dst_some as usize] = s;
                }
                Op::MakeClosure { dst, spec } => {
                    let cs = &chunk.closure_specs[spec as usize];
                    let mut caps = Vec::with_capacity(cs.caps.len());
                    for cap in &cs.caps {
                        match *cap {
                            CapSrc::GlobalRef(gid) => {
                                caps.push(CaptureVal::GlobalRef(gid as usize));
                            }
                            CapSrc::GlobalVal(gid) => {
                                caps.push(CaptureVal::Val(self.globals[gid as usize].clone()));
                            }
                            CapSrc::LocalI(r, kind) => {
                                caps.push(CaptureVal::Val(kind.box_val(regs_i[r as usize])));
                            }
                            CapSrc::LocalOptI { val, some, kind } => {
                                caps.push(CaptureVal::Val(
                                    kind.box_opt(regs_i[val as usize], regs_i[some as usize]),
                                ));
                            }
                            CapSrc::LocalSmallTuple { base, len, kinds } => {
                                let mut fs = Vec::with_capacity(len as usize);
                                for k in 0..len as usize {
                                    fs.push(kinds[k].box_val(regs_i[base as usize + k]));
                                }
                                caps.push(CaptureVal::Val(Value::Tuple(TupleVal::Boxed(Rc::new(
                                    fs,
                                )))));
                            }
                            CapSrc::LocalV(r) => {
                                caps.push(CaptureVal::Val(regs_v[r as usize].clone()));
                            }
                        }
                    }
                    regs_v[dst as usize] = Value::Closure(cs.cid as usize, Rc::new(caps));
                }
                Op::TakeLocalV { dst, src } => {
                    regs_v[dst as usize] = take_value(&mut regs_v[src as usize], spans[cur_pc])?;
                }
                Op::TakePlaceV { dst, path } => {
                    let pp = &chunk.places[path as usize];
                    let root = match pp.root {
                        PlaceRoot::LocalV(v) => &mut regs_v[v as usize],
                        PlaceRoot::Global(gid) => &mut self.globals[gid as usize],
                    };
                    let cell = walk_place_mut(root, &pp.steps, regs_i, spans[cur_pc])?;
                    regs_v[dst as usize] = take_value(cell, spans[cur_pc])?;
                }
                Op::CallVmFeed { spec } => {
                    let fc = &chunk.feed_calls[spec as usize];
                    let fed_chunk = compiled.fed_fns[fc.fn_id]
                        .as_ref()
                        .expect("fed_fn compiled");
                    let callee_base_i = vm_regs_i.len();
                    vm_regs_i.resize(callee_base_i + fed_chunk.num_i, 0);
                    let mut callee_regs_v = self.take_regs_v(fed_chunk.num_v);
                    {
                        let mut ii = 0usize;
                        let mut iv = 0usize;
                        for loc in fed_chunk.slot_map.iter().take(fed_chunk.param_count) {
                            match *loc {
                                SlotLoc::I(r, _) => {
                                    let val = vm_regs_i[base_i + fc.args_i[ii] as usize];
                                    vm_regs_i[callee_base_i + r as usize] = val;
                                    ii += 1;
                                }
                                SlotLoc::OptI { val, some, .. } => {
                                    let v = vm_regs_i[base_i + fc.args_i[ii] as usize];
                                    let s = vm_regs_i[base_i + fc.args_i[ii + 1] as usize];
                                    vm_regs_i[callee_base_i + val as usize] = v;
                                    vm_regs_i[callee_base_i + some as usize] = s;
                                    ii += 2;
                                }
                                SlotLoc::V(r) => {
                                    callee_regs_v[r as usize] =
                                        regs_v[fc.args_v[iv] as usize].clone();
                                    iv += 1;
                                }
                                SlotLoc::SmallTuple { .. } => {
                                    unreachable!("params are never SmallTuple")
                                }
                            }
                        }
                    }
                    let dom_mark = self.saved_domains.len();
                    for &d in &fed_chunk.guarded_domains {
                        self.saved_domains.push((d, self.domains[d]));
                    }
                    let caller_base = self.base;
                    let pushed_stack = fed_chunk.needs_stack;
                    if pushed_stack {
                        let new_base = self.stack.len();
                        self.stack
                            .resize(new_base + fed_chunk.slot_map.len(), Value::Unit);
                        self.base = new_base;
                    }
                    let mut ctx = FedCtx {
                        caller_chunk: chunk,
                        caller_base_i: base_i,
                        caller_regs_v: regs_v,
                        caller_base,
                        body_pc: fc.body_pc,
                        var_loc: fc.var_loc,
                    };
                    let res = self.exec_chunk_core(
                        compiled,
                        fed_chunk,
                        callee_base_i,
                        vm_regs_i,
                        &mut callee_regs_v,
                        0,
                        Some(&mut ctx),
                    );
                    if pushed_stack {
                        self.stack.truncate(self.base);
                        self.base = caller_base;
                    }
                    while self.saved_domains.len() > dom_mark {
                        let (id, prev) = self.saved_domains.pop().unwrap();
                        self.domains[id] = prev;
                    }
                    self.return_regs_v(callee_regs_v);
                    vm_regs_i.truncate(callee_base_i);
                    regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                    match res? {
                        ExecExit::FeedNext | ExecExit::FeedBreak => {}
                        ExecExit::Return(v) => return Ok(ExecExit::Return(v)),
                    }
                }
                Op::FeedYieldI { src, kind } => {
                    let raw = regs_i[src as usize];
                    let ctx = fed.as_deref_mut().expect("FeedYieldI outside fed call");
                    match ctx.var_loc {
                        SlotLoc::I(r, _) => {
                            vm_regs_i[ctx.caller_base_i + r as usize] = raw;
                        }
                        SlotLoc::OptI { val, some, .. } => {
                            vm_regs_i[ctx.caller_base_i + val as usize] = raw;
                            vm_regs_i[ctx.caller_base_i + some as usize] = 1;
                        }
                        SlotLoc::V(r) => {
                            ctx.caller_regs_v[r as usize] = kind.box_val(raw);
                        }
                        SlotLoc::SmallTuple { .. } => {
                            unreachable!("scalar yield into SmallTuple");
                        }
                    }
                    let saved_base = self.base;
                    self.base = ctx.caller_base;
                    let body_res = self.exec_chunk_core(
                        compiled,
                        ctx.caller_chunk,
                        ctx.caller_base_i,
                        vm_regs_i,
                        ctx.caller_regs_v,
                        ctx.body_pc,
                        None,
                    );
                    self.base = saved_base;
                    regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                    match body_res? {
                        ExecExit::FeedNext => {}
                        ExecExit::FeedBreak => return Ok(ExecExit::FeedBreak),
                        ExecExit::Return(v) => return Ok(ExecExit::Return(v)),
                    }
                }
                Op::FeedYieldV { src } => {
                    let val = if src >= chunk.num_local_v as u16 {
                        std::mem::replace(&mut regs_v[src as usize], Value::Unit)
                    } else {
                        regs_v[src as usize].clone()
                    };
                    let ctx = fed.as_deref_mut().expect("FeedYieldV outside fed call");
                    store_fed_val(ctx, vm_regs_i, val);
                    let saved_base = self.base;
                    self.base = ctx.caller_base;
                    let body_res = self.exec_chunk_core(
                        compiled,
                        ctx.caller_chunk,
                        ctx.caller_base_i,
                        vm_regs_i,
                        ctx.caller_regs_v,
                        ctx.body_pc,
                        None,
                    );
                    self.base = saved_base;
                    regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                    match body_res? {
                        ExecExit::FeedNext => {}
                        ExecExit::FeedBreak => return Ok(ExecExit::FeedBreak),
                        ExecExit::Return(v) => return Ok(ExecExit::Return(v)),
                    }
                }
                Op::FeedYieldExtend { src } => {
                    let seq = regs_v[src as usize].clone();
                    let ctx = fed.as_deref_mut().expect("FeedYieldExtend outside fed call");
                    let items: Vec<Value> = match seq {
                        Value::List(xs) => xs.iter().cloned().collect(),
                        Value::Array(arr) => (0..arr.len()).map(|i| arr.get_unchecked_val(i)).collect(),
                        _ => {
                            return trap(
                                spans[cur_pc],
                                "internal: a `yields` fn did something else to its list",
                            )
                        }
                    };
                    for item in items {
                        store_fed_val(ctx, vm_regs_i, item);
                        let saved_base = self.base;
                        self.base = ctx.caller_base;
                        let body_res = self.exec_chunk_core(
                            compiled,
                            ctx.caller_chunk,
                            ctx.caller_base_i,
                            vm_regs_i,
                            ctx.caller_regs_v,
                            ctx.body_pc,
                            None,
                        );
                        self.base = saved_base;
                        match body_res? {
                            ExecExit::FeedNext => {}
                            ExecExit::FeedBreak => return Ok(ExecExit::FeedBreak),
                            ExecExit::Return(v) => return Ok(ExecExit::Return(v)),
                        }
                    }
                    regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                }
                Op::FeedBodyNext => return Ok(ExecExit::FeedNext),
                Op::FeedBodyBreak => return Ok(ExecExit::FeedBreak),
                Op::FeedEnd => return Ok(ExecExit::FeedNext),
                Op::ClearV { reg } => {
                    regs_v[reg as usize] = Value::Unit;
                }
                Op::ConstV { dst, idx } => {
                    regs_v[dst as usize] = chunk.consts_v[idx as usize].clone();
                }
                Op::Print { idx } => {
                    let spec = &chunk.prints[idx as usize];
                    let vals: Vec<Value> = spec.regs.iter().map(|&r| regs_v[r as usize].clone()).collect();
                    let end = spec.end.map(|r| regs_v[r as usize].clone());
                    self.print_values(spec.args, &vals, end.as_ref(), spans[cur_pc])?;
                }
                Op::Format { idx, dst } => {
                    let spec = &chunk.formats[idx as usize];
                    let vals: Vec<Value> = spec.regs.iter().map(|&r| regs_v[r as usize].clone()).collect();
                    regs_v[dst as usize] = self.format_values(spec.pieces, &vals, spans[cur_pc])?;
                }
                Op::FallbackStmt { idx, clear_from_v, in_loop } => {
                    if let Some(stats) = &mut self.vm_stats {
                        stats.record_with_loop(
                            stmt_tag(chunk.fallback_stmts[idx as usize]),
                            in_loop,
                        );
                    }
                    for (r, held) in regs_v.iter_mut().enumerate().skip(clear_from_v as usize) {
                        // A traced local outlives its last use: the tracer shows it.
                        if !(chunk.traced && chunk.slot_map.contains(&SlotLoc::V(r as u16))) {
                            *held = Value::Unit;
                        }
                    }
                    sync_to_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base, chunk.traced);
                    let stmt = chunk.fallback_stmts[idx as usize];
                    let flow = self.stmt(stmt);
                    sync_from_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base, chunk.traced);
                    let flow = flow?;
                    match flow {
                        Flow::Next => {}
                        Flow::Return(v) | Flow::Give(v) => return Ok(ExecExit::Return(v)),
                        Flow::Break(id) => escaped = Some((true, id)),
                        Flow::Continue(id) => escaped = Some((false, id)),
                    }
                }
                Op::FallbackExprI { idx, dst, in_loop } => {
                    if let Some(stats) = &mut self.vm_stats {
                        stats.record_with_loop(
                            expr_tag(chunk.fallback_exprs[idx as usize], self.program),
                            in_loop,
                        );
                    }
                    sync_to_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base, chunk.traced);
                    let expr = chunk.fallback_exprs[idx as usize];
                    let val = self.eval(expr);
                    sync_from_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base, chunk.traced);
                    let val = val?;
                    regs_i[dst as usize] = ScalarKind::unbox_val(&val);
                }
                Op::Trace { idx } => match chunk.trace_points[idx as usize] {
                    TracePoint::LoopEnd(slot) => {
                        let held = &self.stack[self.base + slot];
                        if let (SlotLoc::I(r, _), Value::Int(_) | Value::Bool(_) | Value::Char(_)) =
                            (chunk.slot_map[slot], held)
                        {
                            regs_i[r as usize] = ScalarKind::unbox_val(held);
                        }
                    }
                    ref point => {
                        let mut quiet = false;
                        if let TracePoint::Let(slot, _) | TracePoint::Loop(slot, _) = *point {
                            self.stack[self.base + slot] = Value::Moved;
                            // A top-level local takes no step: only its own
                            // slot needs the register's value.
                            if let SlotLoc::I(r, kind) = chunk.slot_map[slot] {
                                if self.call_frames.is_empty() && !self.assignments() {
                                    self.stack[self.base + slot] = kind.box_val(regs_i[r as usize]);
                                    quiet = true;
                                }
                            }
                        }
                        if !quiet {
                            sync_to_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base, chunk.traced);
                            self.trace_point(point);
                            take_back_values(&chunk.slot_map, regs_v, &mut self.stack, self.base);
                        }
                    }
                },
                Op::FallbackExprV { idx, dst, in_loop } => {
                    if let Some(stats) = &mut self.vm_stats {
                        stats.record_with_loop(
                            expr_tag(chunk.fallback_exprs[idx as usize], self.program),
                            in_loop,
                        );
                    }
                    sync_to_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base, chunk.traced);
                    let expr = chunk.fallback_exprs[idx as usize];
                    let val = self.eval(expr);
                    sync_from_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base, chunk.traced);
                    let val = val?;
                    regs_v[dst as usize] = val;
                }
            }
        }
    }
}

/// Whether `e` neither writes, prints nor calls: what a traced run lets the
/// VM evaluate without telling the tracer.
fn quiet(e: &Expr) -> bool {
    let mut quiet = true;
    cpl_ir::walk::deep_expr(e, &mut |e| {
        quiet &= !matches!(
            e.kind,
            ExprKind::Call(..)
                | ExprKind::CallExpr(..)
                | ExprKind::Closure(..)
                | ExprKind::Do(_)
                | ExprKind::MutSlice(..)
                | ExprKind::Mutate(..)
                | ExprKind::Sort(..)
                | ExprKind::Print(..)
                | ExprKind::Debug(..)
        );
    });
    quiet
}

/// Whether the indices of `place` are [`quiet`] and do not read its root,
/// so that evaluating them after the write gives the cell that was written.
fn quiet_place(place: &Place, root: (bool, usize)) -> bool {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => true,
        PlaceKind::Index(base, indices) => {
            quiet_place(base, root)
                && indices.iter().all(|index| {
                    let mut reads_root = false;
                    cpl_ir::walk::deep_expr(index, &mut |e| {
                        reads_root |= matches!(e.kind, ExprKind::Global(id) if root == (true, id))
                            || matches!(e.kind, ExprKind::Local(slot) if root == (false, slot));
                    });
                    quiet(index) && !reads_root
                })
        }
        PlaceKind::Field(base, _) => quiet_place(base, root),
        PlaceKind::Some(_) => false,
    }
}

/// Where a traced `for` reports its variable, when its iterable is [`quiet`].
fn quiet_iter(iter: &Iter) -> Option<Span> {
    match iter {
        Iter::Range { lo, hi, step, .. } => {
            (quiet(lo) && quiet(hi) && step.as_ref().is_none_or(quiet)).then_some(lo.span)
        }
        Iter::RangeVal(e, _) | Iter::Seq(e, _) | Iter::Subsets(e) | Iter::Members(e) => {
            quiet(e).then_some(e.span)
        }
        Iter::TreeRange { .. } => None,
    }
}

#[inline(always)]
fn place_root_mut<'a>(
    root: PlaceRoot,
    regs_v: &'a mut [Value],
    globals: &'a mut [Value],
) -> &'a mut Value {
    match root {
        PlaceRoot::LocalV(v) => &mut regs_v[v as usize],
        PlaceRoot::Global(gid) => &mut globals[gid as usize],
    }
}

#[cfg_attr(target_arch = "wasm32", inline(never))]
fn exec_bin_u64(a_raw: i64, b_raw: i64, op: BinOp, span: Span) -> Res<i64> {
    let av = a_raw as u64;
    let bv = b_raw as u64;
    let out = match op {
        BinOp::Add => av.wrapping_add(bv),
        BinOp::Sub => av.wrapping_sub(bv),
        BinOp::Mul => av.wrapping_mul(bv),
        BinOp::Div => {
            if bv == 0 {
                return trap(span, format!("division by zero: `{av} / 0`"));
            }
            av / bv
        }
        BinOp::Rem => {
            if bv == 0 {
                return trap(span, format!("division by zero: `{av} % 0`"));
            }
            av % bv
        }
        BinOp::Pow => {
            let mut base = av;
            let mut exp = bv;
            let mut result = 1u64;
            while exp > 0 {
                if exp & 1 == 1 {
                    result = result.wrapping_mul(base);
                }
                base = base.wrapping_mul(base);
                exp >>= 1;
            }
            result
        }
        BinOp::Shl => {
            if !(0..64).contains(&b_raw) {
                return trap(span, format!("a shift by {b_raw} is outside 0..64"));
            }
            av << b_raw
        }
        BinOp::Shr => {
            if !(0..64).contains(&b_raw) {
                return trap(span, format!("a shift by {b_raw} is outside 0..64"));
            }
            av >> b_raw
        }
        _ => return trap(span, "internal: unsupported U64 op"),
    };
    Ok(out as i64)
}

#[inline(never)]
fn exec_sort_place(target: &mut Value, desc: bool, span: Span) -> Res<()> {
    fn sort_ord<T: Ord>(v: &mut [T], desc: bool) {
        v.sort_unstable();
        if desc {
            v.reverse();
        }
    }
    match target {
        Value::List(items) => sort_ord(Rc::make_mut(items).as_mut_slice(), desc),
        Value::Str(bytes) => sort_ord(Rc::make_mut(bytes).as_mut_slice(), desc),
        Value::Array(arr) => {
            let a = Rc::make_mut(arr);
            match &mut a.data {
                ArrayData::Int(v) | ArrayData::OneIntStruct(v) => sort_ord(v, desc),
                ArrayData::OptInt(v) => sort_ord(v, desc),
                ArrayData::Bool(v) => sort_ord(v, desc),
                ArrayData::Char(v) => sort_ord(v, desc),
                ArrayData::Values(v) => sort_ord(v, desc),
                ArrayData::Float(v) => {
                    v.sort_unstable_by(f64::total_cmp);
                    if desc {
                        v.reverse();
                    }
                }
                ArrayData::SmallIntTuple { .. } => {
                    let mut boxed: Vec<Value> =
                        (0..a.len()).map(|i| a.get_unchecked_val(i)).collect();
                    sort_ord(&mut boxed, desc);
                    for (i, val) in boxed.into_iter().enumerate() {
                        a.set(i, val);
                    }
                }
            }
        }
        _ => return trap(span, "internal: cannot sort this value"),
    }
    Ok(())
}

enum ExecExit {
    Return(Value),
    FeedNext,
    FeedBreak,
}

struct FedCtx<'a, 'p> {
    caller_chunk: &'a Chunk<'p>,
    caller_base_i: usize,
    caller_regs_v: &'a mut [Value],
    caller_base: usize,
    body_pc: usize,
    var_loc: SlotLoc,
}

fn store_fed_val(ctx: &mut FedCtx<'_, '_>, vm_regs_i: &mut [i64], val: Value) {
    match ctx.var_loc {
        SlotLoc::I(r, _) => {
            vm_regs_i[ctx.caller_base_i + r as usize] = ScalarKind::unbox_val(&val);
        }
        SlotLoc::OptI { val: r_v, some: r_s, .. } => {
            let (v, s) = ScalarKind::unbox_opt(&val);
            vm_regs_i[ctx.caller_base_i + r_v as usize] = v;
            vm_regs_i[ctx.caller_base_i + r_s as usize] = s;
        }
        SlotLoc::SmallTuple { base, len, .. } => match &val {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                for k in 0..len as usize {
                    vm_regs_i[ctx.caller_base_i + base as usize + k] =
                        ScalarKind::unbox_val(&fs[k]);
                }
            }
            Value::Tuple(TupleVal::OneInt(v)) => {
                vm_regs_i[ctx.caller_base_i + base as usize] = *v;
            }
            _ => {}
        },
        SlotLoc::V(r) => {
            ctx.caller_regs_v[r as usize] = val;
        }
    }
}

fn take_value(cell: &mut Value, span: Span) -> Res<Value> {
    if matches!(cell, Value::Moved) {
        return trap(span, LENT);
    }
    let empty = match &*cell {
        Value::List(_) => Value::List(Rc::default()),
        Value::Str(_) => Value::Str(Rc::default()),
        Value::Map(_) => Value::Map(Rc::default()),
        Value::Set(_) => Value::Set(Rc::default()),
        other => other.clone(),
    };
    Ok(std::mem::replace(cell, empty))
}

#[inline(never)]
fn array_flat_index(arr: &Array, step: PlaceStep, regs_i: &[i64], span: Span) -> Res<usize> {
    match step {
        PlaceStep::Index1(r_i) => {
            let idx = regs_i[r_i as usize];
            let len = arr.dims[0];
            if idx < 0 || idx >= len {
                return trap(
                    span,
                    format!("index {idx} is outside an array dimension of length {len}"),
                );
            }
            Ok(idx as usize)
        }
        PlaceStep::Index2(r_i, r_j) => {
            let i = regs_i[r_i as usize];
            let j = regs_i[r_j as usize];
            let d0 = arr.dims[0];
            let d1 = arr.dims[1];
            if i < 0 || i >= d0 {
                return trap(
                    span,
                    format!("index {i} is outside an array dimension of length {d0}"),
                );
            }
            if j < 0 || j >= d1 {
                return trap(
                    span,
                    format!("index {j} is outside an array dimension of length {d1}"),
                );
            }
            Ok((i * d1 + j) as usize)
        }
        PlaceStep::Field(_) => trap(span, "internal: expected index step"),
    }
}

#[inline(never)]
fn walk_place_mut<'a>(
    mut cur: &'a mut Value,
    steps: &[PlaceStep],
    regs_i: &[i64],
    span: Span,
) -> Res<&'a mut Value> {
    for &step in steps {
        match step {
            PlaceStep::Field(f) => match cur {
                Value::Tuple(TupleVal::Boxed(fs)) => {
                    cur = &mut Rc::make_mut(fs)[f as usize];
                }
                _ => return trap(span, "internal: expected tuple"),
            },
            PlaceStep::Index1(r_i) => match cur {
                Value::List(items) => {
                    let idx = regs_i[r_i as usize];
                    let len = items.len() as i64;
                    if idx < 0 || idx >= len {
                        return trap(span, format!("index {idx} is outside a list of length {len}"));
                    }
                    cur = &mut Rc::make_mut(items)[idx as usize];
                }
                Value::Array(arr) => {
                    let flat = array_flat_index(arr, step, regs_i, span)?;
                    let a = Rc::make_mut(arr);
                    match &mut a.data {
                        ArrayData::Values(vs) => cur = &mut vs[flat],
                        _ => return trap(span, "internal: expected boxed array element"),
                    }
                }
                _ => return trap(span, "internal: expected array or list"),
            },
            PlaceStep::Index2(..) => match cur {
                Value::Array(arr) => {
                    let flat = array_flat_index(arr, step, regs_i, span)?;
                    let a = Rc::make_mut(arr);
                    match &mut a.data {
                        ArrayData::Values(vs) => cur = &mut vs[flat],
                        _ => return trap(span, "internal: expected boxed array element"),
                    }
                }
                _ => return trap(span, "internal: expected array"),
            },
        }
    }
    Ok(cur)
}

#[inline(never)]
fn store_place_leaf_scalar(
    container: &mut Value,
    last: PlaceStep,
    raw: i64,
    kind: ScalarKind,
    regs_i: &[i64],
    span: Span,
) -> Res<()> {
    match last {
        PlaceStep::Field(f) => match container {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                Rc::make_mut(fs)[f as usize] = kind.box_val(raw);
                Ok(())
            }
            Value::Tuple(TupleVal::OneInt(x)) if f == 0 => {
                *x = raw;
                Ok(())
            }
            _ => trap(span, "internal: expected tuple"),
        },
        PlaceStep::Index1(r_i) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut a.data {
                    ArrayData::Int(v) | ArrayData::OneIntStruct(v) => v[flat] = raw,
                    ArrayData::Bool(v) => v[flat] = u8::from(raw != 0),
                    ArrayData::Char(v) => v[flat] = raw as u8,
                    ArrayData::Values(v) => v[flat] = kind.box_val(raw),
                    _ => a.set(flat, kind.box_val(raw)),
                }
                Ok(())
            }
            Value::List(items) => {
                let idx = regs_i[r_i as usize];
                let len = items.len() as i64;
                if idx < 0 || idx >= len {
                    return trap(span, format!("index {idx} is outside a list of length {len}"));
                }
                Rc::make_mut(items)[idx as usize] = kind.box_val(raw);
                Ok(())
            }
            _ => trap(span, "internal: expected array or list"),
        },
        PlaceStep::Index2(..) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut a.data {
                    ArrayData::Int(v) | ArrayData::OneIntStruct(v) => v[flat] = raw,
                    ArrayData::Bool(v) => v[flat] = u8::from(raw != 0),
                    ArrayData::Char(v) => v[flat] = raw as u8,
                    ArrayData::Values(v) => v[flat] = kind.box_val(raw),
                    _ => a.set(flat, kind.box_val(raw)),
                }
                Ok(())
            }
            _ => trap(span, "internal: expected array"),
        },
    }
}

#[inline(never)]
fn store_place_leaf_opt_scalar(
    container: &mut Value,
    last: PlaceStep,
    raw: i64,
    some: i64,
    kind: ScalarKind,
    regs_i: &[i64],
    span: Span,
) -> Res<()> {
    match last {
        PlaceStep::Field(f) => match container {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                Rc::make_mut(fs)[f as usize] = kind.box_opt(raw, some);
                Ok(())
            }
            _ => trap(span, "internal: expected tuple"),
        },
        PlaceStep::Index1(r_i) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut a.data {
                    ArrayData::OptInt(v) => v[flat] = if some != 0 { Some(raw) } else { None },
                    ArrayData::Values(v) => v[flat] = kind.box_opt(raw, some),
                    _ => a.set(flat, kind.box_opt(raw, some)),
                }
                Ok(())
            }
            Value::List(items) => {
                let idx = regs_i[r_i as usize];
                let len = items.len() as i64;
                if idx < 0 || idx >= len {
                    return trap(span, format!("index {idx} is outside a list of length {len}"));
                }
                Rc::make_mut(items)[idx as usize] = kind.box_opt(raw, some);
                Ok(())
            }
            _ => trap(span, "internal: expected array or list"),
        },
        PlaceStep::Index2(..) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut a.data {
                    ArrayData::OptInt(v) => v[flat] = if some != 0 { Some(raw) } else { None },
                    ArrayData::Values(v) => v[flat] = kind.box_opt(raw, some),
                    _ => a.set(flat, kind.box_opt(raw, some)),
                }
                Ok(())
            }
            _ => trap(span, "internal: expected array"),
        },
    }
}

#[inline(never)]
fn store_place_leaf_val(
    container: &mut Value,
    last: PlaceStep,
    val: Value,
    regs_i: &[i64],
    span: Span,
) -> Res<()> {
    match last {
        PlaceStep::Field(f) => match container {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                Rc::make_mut(fs)[f as usize] = val;
                Ok(())
            }
            _ => trap(span, "internal: expected tuple"),
        },
        PlaceStep::Index1(r_i) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                Rc::make_mut(arr).set(flat, val);
                Ok(())
            }
            Value::List(items) => {
                let idx = regs_i[r_i as usize];
                let len = items.len() as i64;
                if idx < 0 || idx >= len {
                    return trap(span, format!("index {idx} is outside a list of length {len}"));
                }
                Rc::make_mut(items)[idx as usize] = val;
                Ok(())
            }
            _ => trap(span, "internal: expected array or list"),
        },
        PlaceStep::Index2(..) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                Rc::make_mut(arr).set(flat, val);
                Ok(())
            }
            _ => trap(span, "internal: expected array"),
        },
    }
}

#[inline(never)]
fn apply_scalar_assign_op(
    cur: i64,
    rhs: i64,
    op: AssignOp,
    kind: ScalarKind,
    span: Span,
) -> Res<i64> {
    if kind == ScalarKind::U64 {
        let a = cur as u64;
        let b = rhs as u64;
        let out = match op {
            AssignOp::Binary(BinOp::Add) => a.wrapping_add(b),
            AssignOp::Binary(BinOp::Sub) => a.wrapping_sub(b),
            AssignOp::Binary(BinOp::Mul) => a.wrapping_mul(b),
            AssignOp::Binary(BinOp::Div) => {
                if b == 0 {
                    return trap(span, format!("division by zero: `{a} / 0`"));
                }
                a / b
            }
            AssignOp::Binary(BinOp::Rem) => {
                if b == 0 {
                    return trap(span, format!("division by zero: `{a} % 0`"));
                }
                a % b
            }
            AssignOp::Binary(BinOp::BitAnd) => a & b,
            AssignOp::Binary(BinOp::BitOr) => a | b,
            AssignOp::Binary(BinOp::BitXor) => a ^ b,
            AssignOp::Binary(BinOp::Shl) => {
                if !(0..64).contains(&rhs) {
                    return trap(span, format!("a shift by {rhs} is outside 0..64"));
                }
                a << rhs
            }
            AssignOp::Binary(BinOp::Shr) => {
                if !(0..64).contains(&rhs) {
                    return trap(span, format!("a shift by {rhs} is outside 0..64"));
                }
                a >> rhs
            }
            AssignOp::Min => a.min(b),
            AssignOp::Max => a.max(b),
            _ => b,
        };
        return Ok(out as i64);
    }
    let out = match op {
        AssignOp::Binary(BinOp::Add) => match cur.checked_add(rhs) {
            Some(v) => v,
            None => return trap(span, format!("integer overflow: `{cur} + {rhs}` does not fit in an `Int` (64 bits)")),
        },
        AssignOp::Binary(BinOp::Sub) => match cur.checked_sub(rhs) {
            Some(v) => v,
            None => return trap(span, format!("integer overflow: `{cur} - {rhs}` does not fit in an `Int` (64 bits)")),
        },
        AssignOp::Binary(BinOp::Mul) => match cur.checked_mul(rhs) {
            Some(v) => v,
            None => return trap(span, format!("integer overflow: `{cur} * {rhs}` does not fit in an `Int` (64 bits)")),
        },
        AssignOp::Binary(BinOp::Div) => {
            if rhs == 0 {
                return trap(span, format!("division by zero: `{cur} / 0`"));
            }
            match cur.checked_div_euclid(rhs) {
                Some(v) => v,
                None => return trap(span, format!("integer overflow: `{cur} / {rhs}` does not fit in an `Int` (64 bits)")),
            }
        }
        AssignOp::Binary(BinOp::Rem) => {
            if rhs == 0 {
                return trap(span, format!("division by zero: `{cur} % 0`"));
            }
            match cur.checked_rem_euclid(rhs) {
                Some(v) => v,
                None => return trap(span, format!("integer overflow: `{cur} % {rhs}` does not fit in an `Int` (64 bits)")),
            }
        }
        AssignOp::Binary(BinOp::BitAnd) => cur & rhs,
        AssignOp::Binary(BinOp::BitOr) => cur | rhs,
        AssignOp::Binary(BinOp::BitXor) => cur ^ rhs,
        AssignOp::Binary(BinOp::Shl) => {
            if !(0..64).contains(&rhs) {
                return trap(span, format!("a shift by {rhs} is outside 0..64"));
            }
            cur.wrapping_shl(rhs as u32)
        }
        AssignOp::Binary(BinOp::Shr) => {
            if !(0..64).contains(&rhs) {
                return trap(span, format!("a shift by {rhs} is outside 0..64"));
            }
            cur.wrapping_shr(rhs as u32)
        }
        AssignOp::Min => cur.min(rhs),
        AssignOp::Max => cur.max(rhs),
        _ => rhs,
    };
    Ok(out)
}

#[inline(never)]
fn assign_place_leaf_scalar(
    container: &mut Value,
    last: PlaceStep,
    rhs: i64,
    op: AssignOp,
    kind: ScalarKind,
    regs_i: &[i64],
    span: Span,
) -> Res<()> {
    match last {
        PlaceStep::Field(f) => match container {
            Value::Tuple(TupleVal::Boxed(fs)) => {
                let slice = Rc::make_mut(fs);
                let cur = ScalarKind::unbox_val(&slice[f as usize]);
                let next = apply_scalar_assign_op(cur, rhs, op, kind, span)?;
                slice[f as usize] = kind.box_val(next);
                Ok(())
            }
            Value::Tuple(TupleVal::OneInt(x)) if f == 0 => {
                *x = apply_scalar_assign_op(*x, rhs, op, kind, span)?;
                Ok(())
            }
            _ => trap(span, "internal: expected tuple"),
        },
        PlaceStep::Index1(r_i) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut a.data {
                    ArrayData::Int(v) | ArrayData::OneIntStruct(v) => {
                        v[flat] = apply_scalar_assign_op(v[flat], rhs, op, kind, span)?;
                    }
                    ArrayData::Values(v) => {
                        let cur = ScalarKind::unbox_val(&v[flat]);
                        let next = apply_scalar_assign_op(cur, rhs, op, kind, span)?;
                        v[flat] = kind.box_val(next);
                    }
                    _ => return trap(span, "internal: unsupported array assign"),
                }
                Ok(())
            }
            Value::List(items) => {
                let idx = regs_i[r_i as usize];
                let len = items.len() as i64;
                if idx < 0 || idx >= len {
                    return trap(span, format!("index {idx} is outside a list of length {len}"));
                }
                let slice = Rc::make_mut(items);
                let cur = ScalarKind::unbox_val(&slice[idx as usize]);
                let next = apply_scalar_assign_op(cur, rhs, op, kind, span)?;
                slice[idx as usize] = kind.box_val(next);
                Ok(())
            }
            _ => trap(span, "internal: expected array or list"),
        },
        PlaceStep::Index2(..) => match container {
            Value::Array(arr) => {
                let flat = array_flat_index(arr, last, regs_i, span)?;
                let a = Rc::make_mut(arr);
                match &mut a.data {
                    ArrayData::Int(v) | ArrayData::OneIntStruct(v) => {
                        v[flat] = apply_scalar_assign_op(v[flat], rhs, op, kind, span)?;
                    }
                    ArrayData::Values(v) => {
                        let cur = ScalarKind::unbox_val(&v[flat]);
                        let next = apply_scalar_assign_op(cur, rhs, op, kind, span)?;
                        v[flat] = kind.box_val(next);
                    }
                    _ => return trap(span, "internal: unsupported array assign"),
                }
                Ok(())
            }
            _ => trap(span, "internal: expected array"),
        },
    }
}

#[derive(Default)]
pub(crate) struct VmStats {
    pub(crate) total_fallbacks: u64,
    pub(crate) loop_fallbacks: u64,
    pub(crate) hot_loop_fallbacks: u64,
    pub(crate) counts: std::collections::BTreeMap<String, u64>,
    pub(crate) loop_counts: std::collections::BTreeMap<String, u64>,
}

impl VmStats {
    pub(crate) fn from_env() -> Option<Self> {
        let val = std::env::var_os("CPL_VM_STATS")?;
        if val.is_empty() || val == "0" {
            return None;
        }
        Some(Self::default())
    }

    pub(crate) fn record(&mut self, tag: String) {
        self.record_with_loop(tag, false);
    }

    pub(crate) fn record_with_loop(&mut self, tag: String, in_loop: bool) {
        self.total_fallbacks += 1;
        if in_loop {
            self.loop_fallbacks += 1;
            let is_io = matches!(tag.as_str(), "expr:Read" | "stmt:Expr(Print)");
            if !is_io {
                self.hot_loop_fallbacks += 1;
                *self.loop_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        *self.counts.entry(tag).or_insert(0) += 1;
    }

    pub(crate) fn emit_stderr(&self) {
        let mut entries: Vec<(&str, u64)> =
            self.counts.iter().map(|(k, &v)| (k.as_str(), v)).collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        let mut loop_entries: Vec<(&str, u64)> =
            self.loop_counts.iter().map(|(k, &v)| (k.as_str(), v)).collect();
        loop_entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        if std::env::var("CPL_VM_STATS").ok().as_deref() == Some("json") {
            let mut s = format!(
                "{{\"total_fallbacks\":{},\"loop_fallbacks\":{},\"hot_loop_fallbacks\":{},\"top\":[",
                self.total_fallbacks, self.loop_fallbacks, self.hot_loop_fallbacks
            );
            for (i, (k, v)) in entries.iter().take(20).enumerate() {
                if i > 0 {
                    s.push(',');
                }
                let escaped = cpl_json::escape_string(k);
                s.push_str(&format!("[{escaped},{v}]"));
            }
            s.push_str("],\"loop_top\":[");
            for (i, (k, v)) in loop_entries.iter().take(20).enumerate() {
                if i > 0 {
                    s.push(',');
                }
                let escaped = cpl_json::escape_string(k);
                s.push_str(&format!("[{escaped},{v}]"));
            }
            s.push_str("]}");
            eprintln!("{s}");
        } else {
            eprintln!(
                "vm-stats: total_fallbacks={} loop_fallbacks={} hot_loop_fallbacks={}",
                self.total_fallbacks, self.loop_fallbacks, self.hot_loop_fallbacks
            );
            for (k, v) in entries.iter().take(15) {
                eprintln!("  {v:>10}  {k}");
            }
        }
    }
}

fn place_tag(place: &Place) -> &'static str {
    match &place.kind {
        PlaceKind::Local(_) => "Local",
        PlaceKind::Global(_) => "Global",
        PlaceKind::Index(base, idx) => match (&base.kind, idx.len()) {
            (PlaceKind::Index(..), _) => "Index2D",
            (_, 2) => "Index2D",
            (_, 1) => "Index1D",
            _ => "IndexND",
        },
        PlaceKind::Field(base, _) => match &base.kind {
            PlaceKind::Index(..) => "Index.Field",
            PlaceKind::Field(..) => "Field.Field",
            _ => "Field",
        },
        PlaceKind::Some(_) => "Some",
    }
}

fn stmt_tag(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Let(_, e) => format!("stmt:Let({})", e.as_ref().map_or("Uninit", expr_tag_short)),
        Stmt::Assign(place, op, _) => format!("stmt:Assign({},{:?})", place_tag(place), op),
        Stmt::Expr(e) => format!("stmt:Expr({})", expr_tag_short(e)),
        Stmt::If(..) => "stmt:If".into(),
        Stmt::While(..) => "stmt:While".into(),
        Stmt::For(_, _, iter, _) => match iter {
            Iter::Range { .. } => "stmt:For(Range)".into(),
            Iter::Seq(_, rev) => {
                if *rev {
                    "stmt:For(SeqRev)".into()
                } else {
                    "stmt:For(Seq)".into()
                }
            }
            _ => "stmt:For(Other)".into(),
        },
        Stmt::Return(_) => "stmt:Return".into(),
        Stmt::Break(_) => "stmt:Break".into(),
        Stmt::Continue(_) => "stmt:Continue".into(),
        Stmt::Give(_) => "stmt:Give".into(),
        Stmt::Assert { .. } => "stmt:Assert".into(),
        Stmt::Trap(..) => "stmt:Trap".into(),
        Stmt::DefDomain { .. } => "stmt:DefDomain".into(),
        Stmt::DefDomainFromVal { .. } => "stmt:DefDomainFromVal".into(),
    }
}

fn expr_tag_short(expr: &Expr) -> &'static str {
    match &expr.kind {
        ExprKind::Call(..) => "Call",
        ExprKind::CallExpr(..) => "CallExpr",
        ExprKind::Builtin(..) => "Builtin",
        ExprKind::Mutate(..) => "Mutate",
        ExprKind::Index(..) => "Index",
        ExprKind::Field(..) => "Field",
        ExprKind::Tuple(_) => "Tuple",
        ExprKind::Fill(..) => "Fill",
        ExprKind::Cast(..) => "Cast",
        ExprKind::Unary(..) => "Unary",
        ExprKind::Binary(..) => "Binary",
        ExprKind::Closure { .. } => "Closure",
        ExprKind::Variant(..) => "Variant",
        ExprKind::IsVariant(..) => "IsVariant",
        ExprKind::Payload(..) => "Payload",
        ExprKind::Read(_) => "Read",
        ExprKind::Parse(..) => "Parse",
        ExprKind::Print { .. } => "Print",
        ExprKind::Debug { .. } => "Debug",
        ExprKind::Format(_) => "Format",
        ExprKind::Some(_) => "Some",
        ExprKind::None => "None",
        ExprKind::IsSome(_) => "IsSome",
        ExprKind::Unwrap(..) => "Unwrap",
        ExprKind::If(..) => "If",
        ExprKind::Do(_) => "Do",
        ExprKind::Range { .. } => "Range",
        ExprKind::Cmp(..) => "Cmp",
        ExprKind::Sort(..) => "Sort",
        ExprKind::DomainVal(_) => "DomainVal",
        ExprKind::DomainLen(_) => "DomainLen",
        ExprKind::DomainLo(_) => "DomainLo",
        ExprKind::Str(_) => "Str",
        ExprKind::Float(_) => "Float",
        ExprKind::Int(_) => "Int",
        ExprKind::Bool(_) => "Bool",
        ExprKind::Char(_) => "Char",
        ExprKind::Local(_) => "Local",
        ExprKind::Global(_) => "Global",
        _ => "Other",
    }
}

fn expr_tag(expr: &Expr, program: &Program) -> String {
    match &expr.kind {
        ExprKind::Call(id, _) => {
            let name = program
                .fns
                .get(*id)
                .map(|f| f.name.as_str())
                .unwrap_or("?");
            format!("expr:Call({name})")
        }
        ExprKind::Builtin(b, _) => format!("expr:Builtin({b:?})"),
        ExprKind::Mutate(m, _, _) => format!("expr:Mutate({m:?})"),
        ExprKind::Cast(c, _) => format!("expr:Cast({c:?})"),
        ExprKind::Binary(op, _, _) => format!("expr:Binary({op:?})"),
        ExprKind::Unary(op, _) => format!("expr:Unary({op:?})"),
        _ => {
            let s = expr_tag_short(expr);
            if s == "Other" {
                let dbg = format!("{:?}", expr.kind);
                let head = dbg.split(['(', '{', ' ']).next().unwrap_or("Other");
                format!("expr:{head}")
            } else {
                format!("expr:{s}")
            }
        }
    }
}
