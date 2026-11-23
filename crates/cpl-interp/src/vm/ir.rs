use super::compile::Compiler;
use super::helpers::block_has_by_ref_closure;
use crate::value::{SmallFieldKind, Value};
use cpl_diag::Span;
use cpl_ir::{
    Arg, AssignOp, BinOp, Builtin, CmpOp, Expr, ExprKind, FmtArg, FmtPiece, FnId, LoopId, Place,
    Program, Slot, Stmt, Ty,
};

pub(super) use super::scalar::small_tuple_shape;
pub(crate) use super::scalar::ScalarKind;

/// Where a local `Slot` lives inside a VM frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SlotLoc {
    I(u16, ScalarKind),
    OptI {
        val: u16,
        some: u16,
        kind: ScalarKind,
    },
    SmallTuple {
        base: u16,
        len: u8,
        kinds: [SmallFieldKind; 4],
    },
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
    /// A map entry, keyed by the `n`th of the path's `keys`.
    Key(u16),
}

#[derive(Clone, Debug)]
pub(crate) struct PlacePath {
    pub root: PlaceRoot,
    pub steps: Vec<PlaceStep>,
    /// The registers holding each `Key` step's key.
    pub keys: Vec<u16>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Op {
    // Constants & moves
    ConstImmI {
        dst: u16,
        val: i32,
    },
    ConstI {
        dst: u16,
        idx: u32,
    },
    MoveI {
        dst: u16,
        src: u16,
    },
    MoveV {
        dst: u16,
        src: u16,
    },
    MoveVConsume {
        dst: u16,
        src: u16,
    },
    BoxI {
        dst_v: u16,
        src_i: u16,
        kind: ScalarKind,
    },
    UnboxI {
        dst_i: u16,
        src_v: u16,
    },
    BoxOptI {
        dst_v: u16,
        val: u16,
        some: u16,
        kind: ScalarKind,
    },
    UnboxOptI {
        dst_val: u16,
        dst_some: u16,
        src_v: u16,
    },

    // Globals
    LoadGlobalI {
        dst: u16,
        gid: u16,
    },
    StoreGlobalI {
        gid: u16,
        src: u16,
        kind: ScalarKind,
    },
    LoadGlobalOptI {
        dst_val: u16,
        dst_some: u16,
        gid: u16,
    },
    StoreGlobalOptI {
        gid: u16,
        val: u16,
        some: u16,
        kind: ScalarKind,
    },
    LoadGlobalV {
        dst: u16,
        gid: u16,
    },
    StoreGlobalV {
        gid: u16,
        src: u16,
    },
    StoreGlobalVMove {
        gid: u16,
        src: u16,
    },

    // Checked i64 arithmetic
    AddI {
        dst: u16,
        a: u16,
        b: u16,
    },
    SubI {
        dst: u16,
        a: u16,
        b: u16,
    },
    MulI {
        dst: u16,
        a: u16,
        b: u16,
    },
    DivI {
        dst: u16,
        a: u16,
        b: u16,
    },
    RemI {
        dst: u16,
        a: u16,
        b: u16,
    },
    ModAddI {
        dst: u16,
        a: u16,
        b: u16,
        modulus: u32,
    },
    ModSubI {
        dst: u16,
        a: u16,
        b: u16,
        modulus: u32,
    },
    ModMulI {
        dst: u16,
        a: u16,
        b: u16,
        modulus: u32,
    },
    ModNegI {
        dst: u16,
        a: u16,
        modulus: u32,
    },
    AddImmI {
        dst: u16,
        a: u16,
        imm: i32,
    },
    NegI {
        dst: u16,
        a: u16,
    },
    BitAndI {
        dst: u16,
        a: u16,
        b: u16,
    },
    BitOrI {
        dst: u16,
        a: u16,
        b: u16,
    },
    BitXorI {
        dst: u16,
        a: u16,
        b: u16,
    },
    BitAndU64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    BitOrU64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    BitXorU64 {
        dst: u16,
        a: u16,
        b: u16,
    },
    ShlI {
        dst: u16,
        a: u16,
        b: u16,
    },
    ShrI {
        dst: u16,
        a: u16,
        b: u16,
    },
    NotBool {
        dst: u16,
        a: u16,
    },
    MinI {
        dst: u16,
        a: u16,
        b: u16,
    },
    MaxI {
        dst: u16,
        a: u16,
        b: u16,
    },
    AbsI {
        dst: u16,
        a: u16,
    },
    WrapAddI {
        dst: u16,
        a: u16,
        b: u16,
    },
    WrapSubI {
        dst: u16,
        a: u16,
        b: u16,
    },
    WrapMulI {
        dst: u16,
        a: u16,
        b: u16,
    },

    // Domain/Pos/Bound & Option superinstructions
    DomainLen {
        dst: u16,
        dom: u16,
    },
    DomainLo {
        dst: u16,
        dom: u16,
    },
    FromLabel {
        dst: u16,
        src: u16,
        dom: u16,
    },
    FromOrd {
        dst: u16,
        src: u16,
        dom: u16,
    },
    PosNext {
        dst_val: u16,
        dst_some: u16,
        src: u16,
        len: u16,
    },
    PosPrev {
        dst_val: u16,
        dst_some: u16,
        src: u16,
    },
    PosOffset {
        dst_val: u16,
        dst_some: u16,
        src: u16,
        delta: u16,
        dom: u16,
    },
    PosTryOrd {
        dst_val: u16,
        dst_some: u16,
        src: u16,
        dom: u16,
    },
    BoundStepL {
        dst: u16,
        ord: u16,
        k: u16,
        dom: u16,
    },
    BoundStepR {
        dst: u16,
        ord: u16,
        k: u16,
        dom: u16,
    },
    UnwrapOptI {
        dst: u16,
        val: u16,
        some: u16,
    },

    // Small unboxed scalar tuples (2..=4 fields)
    LoadGArr1SmallTuple {
        dst_base: u16,
        len: u8,
        gid: u16,
        r_i: u16,
    },
    LoadLArr1SmallTuple {
        dst_base: u16,
        len: u8,
        arr_v: u16,
        r_i: u16,
    },
    StoreGArr1SmallTuple {
        gid: u16,
        r_i: u16,
        src_base: u16,
        len: u8,
        kinds: [SmallFieldKind; 4],
    },
    StoreLArr1SmallTuple {
        arr_v: u16,
        r_i: u16,
        src_base: u16,
        len: u8,
        kinds: [SmallFieldKind; 4],
    },
    LoadGArr2SmallTuple {
        dst_base: u16,
        len: u8,
        gid: u16,
        r_i: u16,
        r_j: u16,
    },
    LoadLArr2SmallTuple {
        dst_base: u16,
        len: u8,
        arr_v: u16,
        r_i: u16,
        r_j: u16,
    },
    StoreGArr2SmallTuple {
        gid: u16,
        r_i: u16,
        r_j: u16,
        src_base: u16,
        len: u8,
        kinds: [SmallFieldKind; 4],
    },
    StoreLArr2SmallTuple {
        arr_v: u16,
        r_i: u16,
        r_j: u16,
        src_base: u16,
        len: u8,
        kinds: [SmallFieldKind; 4],
    },
    BoxSmallTuple {
        dst_v: u16,
        src_base: u16,
        len: u8,
        kinds: [SmallFieldKind; 4],
    },
    UnboxSmallTuple {
        dst_base: u16,
        len: u8,
        src_v: u16,
    },

    // Comparisons & branches
    CmpI {
        dst: u16,
        a: u16,
        b: u16,
        op: CmpOp,
    },
    Jmp {
        target: u32,
    },
    /// Jumps when the last fallback statement broke out of or continued
    /// the loop `id`, or any loop when `innermost` and it named none.
    JmpEscape {
        brk: bool,
        id: LoopId,
        innermost: bool,
        target: u32,
    },
    JmpIfZero {
        reg: u16,
        target: u32,
    },
    JmpIfNonZero {
        reg: u16,
        target: u32,
    },
    JmpLtI {
        a: u16,
        b: u16,
        target: u32,
    },
    JmpLeI {
        a: u16,
        b: u16,
        target: u32,
    },
    JmpGtI {
        a: u16,
        b: u16,
        target: u32,
    },
    JmpGeI {
        a: u16,
        b: u16,
        target: u32,
    },
    JmpEqI {
        a: u16,
        b: u16,
        target: u32,
    },
    JmpNeI {
        a: u16,
        b: u16,
        target: u32,
    },

    // Counted loop superinstructions
    ForInc {
        r_i: u16,
        r_hi: u16,
        target: u32,
    },
    ForIncClosed {
        r_i: u16,
        r_hi: u16,
        target: u32,
    },
    /// Enters `lo..hi by step` (`..=hi` when `closed`): traps on a step
    /// that is not positive, and jumps to `skip` when no value is in range.
    ForStepEnter {
        r_lo: u16,
        r_hi: u16,
        r_step: u16,
        closed: bool,
        skip: u32,
    },
    /// Steps `r_i` by `r_step` and jumps to `target` while still in range.
    ForStepNext {
        r_i: u16,
        r_hi: u16,
        r_step: u16,
        closed: bool,
        target: u32,
    },
    ForDec {
        r_i: u16,
        r_lo: u16,
        target: u32,
    },

    // Unboxed array & list superinstructions (global and local)
    LoadGArr1I {
        dst: u16,
        gid: u16,
        r_i: u16,
    },
    StoreGArr1I {
        gid: u16,
        r_i: u16,
        r_val: u16,
    },
    LoadGArr2I {
        dst: u16,
        gid: u16,
        r_i: u16,
        r_j: u16,
    },
    StoreGArr2I {
        gid: u16,
        r_i: u16,
        r_j: u16,
        r_val: u16,
    },
    LoadLArr1I {
        dst: u16,
        arr_v: u16,
        r_i: u16,
    },
    StoreLArr1I {
        arr_v: u16,
        r_i: u16,
        r_val: u16,
    },
    LoadLArr2I {
        dst: u16,
        arr_v: u16,
        r_i: u16,
        r_j: u16,
    },
    StoreLArr2I {
        arr_v: u16,
        r_i: u16,
        r_j: u16,
        r_val: u16,
    },
    LoadGArr1OptI {
        dst_val: u16,
        dst_some: u16,
        gid: u16,
        r_i: u16,
    },
    StoreGArr1OptI {
        gid: u16,
        r_i: u16,
        r_val: u16,
        r_some: u16,
        kind: ScalarKind,
    },
    LoadGArr2OptI {
        dst_val: u16,
        dst_some: u16,
        gid: u16,
        r_i: u16,
        r_j: u16,
    },
    StoreGArr2OptI {
        gid: u16,
        r_i: u16,
        r_j: u16,
        r_val: u16,
        r_some: u16,
        kind: ScalarKind,
    },
    LoadLArr1OptI {
        dst_val: u16,
        dst_some: u16,
        arr_v: u16,
        r_i: u16,
    },
    StoreLArr1OptI {
        arr_v: u16,
        r_i: u16,
        r_val: u16,
        r_some: u16,
        kind: ScalarKind,
    },
    LoadLArr2OptI {
        dst_val: u16,
        dst_some: u16,
        arr_v: u16,
        r_i: u16,
        r_j: u16,
    },
    StoreLArr2OptI {
        arr_v: u16,
        r_i: u16,
        r_j: u16,
        r_val: u16,
        r_some: u16,
        kind: ScalarKind,
    },
    LoadGList1I {
        dst: u16,
        gid: u16,
        r_i: u16,
    },
    StoreGList1I {
        gid: u16,
        r_i: u16,
        r_val: u16,
        kind: ScalarKind,
    },
    LoadLList1I {
        dst: u16,
        list_v: u16,
        r_i: u16,
    },
    StoreLList1I {
        list_v: u16,
        r_i: u16,
        r_val: u16,
        kind: ScalarKind,
    },
    GArrLen {
        dst: u16,
        gid: u16,
    },
    LArrLen {
        dst: u16,
        arr_v: u16,
    },
    GListLen {
        dst: u16,
        gid: u16,
    },
    LListLen {
        dst: u16,
        list_v: u16,
    },

    LoadGArr1V {
        dst: u16,
        gid: u16,
        r_i: u16,
    },
    StoreGArr1V {
        gid: u16,
        r_i: u16,
        r_val: u16,
    },
    LoadLArr1V {
        dst: u16,
        arr_v: u16,
        r_i: u16,
    },
    StoreLArr1V {
        arr_v: u16,
        r_i: u16,
        r_val: u16,
    },
    FieldI {
        dst: u16,
        obj_v: u16,
        field: u16,
    },
    FieldOptI {
        dst_val: u16,
        dst_some: u16,
        obj_v: u16,
        field: u16,
    },
    FieldV {
        dst: u16,
        obj_v: u16,
        field: u16,
    },
    LoadFieldList1I {
        dst: u16,
        obj_v: u16,
        field: u16,
        r_i: u16,
    },
    FieldListLen {
        dst: u16,
        obj_v: u16,
        field: u16,
    },
    LoadFieldArr1I {
        dst: u16,
        obj_v: u16,
        field: u16,
        r_i: u16,
    },
    FieldArrLen {
        dst: u16,
        obj_v: u16,
        field: u16,
    },
    StrGetI {
        dst: u16,
        str_v: u16,
        r_i: u16,
    },
    GStrGetI {
        dst: u16,
        gid: u16,
        r_i: u16,
    },
    MakeTuple {
        dst: u16,
        base_v: u16,
        len: u8,
    },

    // Direct VM call (unboxed i64 args/return, or mixed I/V args & Unit/I/OptI/V return)
    CallVmI {
        fn_id: u16,
        arg_base: u16,
        argc: u8,
        dst: u16,
    },
    CallVmMix {
        fn_id: u16,
        arg_base_i: u16,
        arg_base_v: u16,
        dst_i: u16,
        dst_v: u16,
        dst_some: u16,
        ret_mode: CallRet,
    },
    /// A traced chunk's call: the tracer sees it enter and leave, as when
    /// the tree-walker calls. `dst_i` is `u16::MAX` unless it returns a scalar.
    /// `lends` is `u16::MAX`, or the `lent_calls` entry whose `mut`
    /// arguments lend their places, their indices in the registers after
    /// the arguments.
    CallTraced {
        /// `u16::MAX` for a call to the fn value in `callee_v`.
        fn_id: u16,
        callee_v: u16,
        arg_base_v: u16,
        argc: u8,
        dst_v: u16,
        dst_i: u16,
        lends: u16,
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
    RetI {
        src: u16,
    },
    RetV {
        src: u16,
    },
    RetUnit,

    // General V-channel optionals & sequence/range/bit/list ops
    IsSomeV {
        dst_i: u16,
        src_v: u16,
    },
    UnwrapV {
        dst_v: u16,
        src_v: u16,
    },
    UnwrapVI {
        dst_i: u16,
        src_v: u16,
    },
    WrapSomeV {
        dst_v: u16,
        src_v: u16,
    },
    ConstNoneV {
        dst_v: u16,
    },
    SeqLastV {
        dst_v: u16,
        seq_v: u16,
    },
    SeqLastOptI {
        dst_val: u16,
        dst_some: u16,
        seq_v: u16,
    },
    SeqFirstV {
        dst_v: u16,
        seq_v: u16,
    },
    SeqFirstOptI {
        dst_val: u16,
        dst_some: u16,
        seq_v: u16,
    },
    SeqLenI {
        dst: u16,
        seq_v: u16,
    },
    RangeLoI {
        dst: u16,
        src_v: u16,
    },
    RangeHiI {
        dst: u16,
        src_v: u16,
    },
    /// Both ends of the `Range` in `src_v`, `hi` exclusive.
    RangeBoundsI {
        lo: u16,
        hi: u16,
        src_v: u16,
    },
    BitCeilI {
        dst: u16,
        src: u16,
    },
    PopcountI {
        dst: u16,
        src: u16,
    },
    CtzI {
        dst: u16,
        src: u16,
    },
    ClzI {
        dst: u16,
        src: u16,
    },
    MinMaxGArr1SmallTuple {
        gid: u16,
        r_i: u16,
        src_base: u16,
        len: u8,
        is_max: bool,
    },
    MinMaxLArr1SmallTuple {
        arr_v: u16,
        r_i: u16,
        src_base: u16,
        len: u8,
        is_max: bool,
    },
    ListPushGArr1I {
        gid: u16,
        r_i: u16,
        val_i: u16,
        kind: ScalarKind,
    },
    ListPushGArr1V {
        gid: u16,
        r_i: u16,
        val_v: u16,
    },
    ListPushLArr1I {
        arr_v: u16,
        r_i: u16,
        val_i: u16,
        kind: ScalarKind,
    },
    ListPushLArr1V {
        arr_v: u16,
        r_i: u16,
        val_v: u16,
    },
    ListPushLV {
        list_v: u16,
        val_v: u16,
    },
    ListPushGV {
        gid: u16,
        val_v: u16,
    },
    ListPushLI {
        list_v: u16,
        val_i: u16,
        kind: ScalarKind,
    },
    ListPushGI {
        gid: u16,
        val_i: u16,
        kind: ScalarKind,
    },
    ListPopLV {
        dst_v: u16,
        list_v: u16,
    },
    ListPopGV {
        dst_v: u16,
        gid: u16,
    },
    ListPopOptIL {
        dst_val: u16,
        dst_some: u16,
        list_v: u16,
    },
    ListPopOptIG {
        dst_val: u16,
        dst_some: u16,
        gid: u16,
    },
    ListPopDiscardL {
        list_v: u16,
    },
    ListPopDiscardG {
        gid: u16,
    },
    LoadGlobalVMove {
        dst: u16,
        gid: u16,
    },
    StoreLFieldI {
        obj_v: u16,
        field: u16,
        src_i: u16,
        kind: ScalarKind,
    },
    StoreLFieldV {
        obj_v: u16,
        field: u16,
        src_v: u16,
    },
    StoreLFieldOptI {
        obj_v: u16,
        field: u16,
        val: u16,
        some: u16,
        kind: ScalarKind,
    },
    StoreLFieldIdx1I {
        obj_v: u16,
        field: u16,
        r_i: u16,
        r_val: u16,
        kind: ScalarKind,
    },
    StoreLFieldIdx1V {
        obj_v: u16,
        field: u16,
        r_i: u16,
        r_val: u16,
    },
    StoreLFieldIdx1OptI {
        obj_v: u16,
        field: u16,
        r_i: u16,
        val: u16,
        some: u16,
        kind: ScalarKind,
    },
    ListPushFieldLI {
        obj_v: u16,
        field: u16,
        val_i: u16,
        kind: ScalarKind,
    },
    ListPushFieldLV {
        obj_v: u16,
        field: u16,
        val_v: u16,
    },
    ListPopDiscardFieldL {
        obj_v: u16,
        field: u16,
    },
    StoreGList1V {
        gid: u16,
        r_i: u16,
        r_val: u16,
    },
    StoreLList1V {
        list_v: u16,
        r_i: u16,
        r_val: u16,
    },
    LoadGList1V {
        dst: u16,
        gid: u16,
        r_i: u16,
    },
    LoadLList1V {
        dst: u16,
        list_v: u16,
        r_i: u16,
    },
    LoadLList1OptI {
        dst_val: u16,
        dst_some: u16,
        list_v: u16,
        r_i: u16,
    },
    /// The members of a set, or the `(key, value)` pairs of a map, as a
    /// list: what a `for` loop over it walks, as it was when the loop began.
    SeqItems {
        dst: u16,
        src: u16,
    },
    ListPopFieldOptIL {
        dst_val: u16,
        dst_some: u16,
        obj_v: u16,
        field: u16,
    },
    ListPopFieldLV {
        dst_v: u16,
        obj_v: u16,
        field: u16,
    },
    MakeRange {
        dst: u16,
        r_lo: u16,
        r_hi: u16,
    },
    CmpOptI {
        dst: u16,
        a_val: u16,
        a_some: u16,
        b_val: u16,
        b_some: u16,
        op: CmpOp,
    },
    CmpV {
        dst: u16,
        a_v: u16,
        b_v: u16,
        op: CmpOp,
    },
    MakeList {
        dst: u16,
        base_v: u16,
        len: u8,
    },
    ListFilled {
        dst: u16,
        r_n: u16,
        val_v: u16,
    },
    ToList {
        dst: u16,
        src_v: u16,
    },
    MakeFill {
        dst: u16,
        dims_base: u16,
        ndims: u8,
        fill_v: u16,
        fill_ty: u16,
    },
    ListSlice {
        dst: u16,
        src_v: u16,
        r_lo: u16,
        r_hi: u16,
    },
    StrSlice {
        dst: u16,
        src_v: u16,
        r_lo: u16,
        r_hi: u16,
    },
    TakeLArr1V {
        dst: u16,
        arr_v: u16,
        r_i: u16,
    },
    TakeGArr1V {
        dst: u16,
        gid: u16,
        r_i: u16,
    },
    TakeLList1V {
        dst: u16,
        list_v: u16,
        r_i: u16,
    },
    TakeGList1V {
        dst: u16,
        gid: u16,
        r_i: u16,
    },
    LoadLArr2V {
        dst: u16,
        arr_v: u16,
        r_i: u16,
        r_j: u16,
    },
    LoadGArr2V {
        dst: u16,
        gid: u16,
        r_i: u16,
        r_j: u16,
    },
    StoreLArr2V {
        arr_v: u16,
        r_i: u16,
        r_j: u16,
        r_val: u16,
    },
    StoreGArr2V {
        gid: u16,
        r_i: u16,
        r_j: u16,
        r_val: u16,
    },
    ConstF64 {
        dst: u16,
        idx: u16,
    },
    FloatBin {
        dst: u16,
        a: u16,
        b: u16,
        op: BinOp,
    },
    FloatMinMax {
        dst: u16,
        a: u16,
        b: u16,
        is_max: bool,
    },
    LoadGIdxFieldI {
        dst: u16,
        gid: u16,
        r_i: u16,
        field: u16,
    },
    StoreGIdxFieldI {
        gid: u16,
        r_i: u16,
        field: u16,
        r_val: u16,
        kind: ScalarKind,
    },
    LoadLIdxFieldI {
        dst: u16,
        seq_v: u16,
        r_i: u16,
        field: u16,
    },
    StoreLIdxFieldI {
        seq_v: u16,
        r_i: u16,
        field: u16,
        r_val: u16,
        kind: ScalarKind,
    },
    LStrLen {
        dst: u16,
        str_v: u16,
    },
    ClearV {
        reg: u16,
    },

    // Wrapping u64 arithmetic, casts & comparisons
    BinU64 {
        dst: u16,
        a: u16,
        b: u16,
        op: BinOp,
    },
    NegU64 {
        dst: u16,
        a: u16,
    },
    BitNotI {
        dst: u16,
        a: u16,
    },
    CmpU64 {
        dst: u16,
        a: u16,
        b: u16,
        op: CmpOp,
    },
    U64ToInt {
        dst: u16,
        src: u16,
    },

    // General nested PlacePath mutations & sort
    StorePlaceI {
        path: u16,
        r_val: u16,
        kind: ScalarKind,
    },
    StorePlaceOptI {
        path: u16,
        r_val: u16,
        r_some: u16,
        kind: ScalarKind,
    },
    StorePlaceV {
        path: u16,
        r_val: u16,
    },
    AssignPlaceI {
        path: u16,
        r_rhs: u16,
        op: AssignOp,
        kind: ScalarKind,
    },
    ListPopDiscardPlace {
        path: u16,
    },
    ListPushPlaceI {
        path: u16,
        val_i: u16,
        kind: ScalarKind,
    },
    ListPushPlaceV {
        path: u16,
        val_v: u16,
    },
    /// Puts the elements of `seq_v` back into `path` in the stable order
    /// of their keys in `keys_v`.
    SortByKeysPlace {
        path: u16,
        seq_v: u16,
        keys_v: u16,
        desc_r: u16,
    },
    SortPlace {
        path: u16,
        desc_r: u16,
    },

    // Closure construction, Take mutation, and fed `yields` loops
    MakeClosure {
        dst: u16,
        spec: u16,
    },
    TakeLocalV {
        dst: u16,
        src: u16,
    },
    TakePlaceV {
        dst: u16,
        path: u16,
    },
    CallVmFeed {
        spec: u16,
    },
    /// `back`: when not 0, the yield is of local `back - 1`, which takes
    /// the loop variable's value after the body, as the tree-walker's
    /// does, so the body can steer the fn (`st.prune()`).
    FeedYieldI {
        src: u16,
        kind: ScalarKind,
        back: u16,
    },
    FeedYieldV {
        src: u16,
        back: u16,
    },
    FeedYieldExtend {
        src: u16,
    },
    /// A `yield` in a traced chunk: when the fn feeds a loop, the value
    /// in `src` goes to the body as the tree-walker's `yield` sends it,
    /// the yielded local `back - 1` (if not 0) takes the loop variable
    /// back, the push to list `acc` (the `Assign` trace point `point`) is
    /// reported, and the run goes on at `skip`. Otherwise the next op, a
    /// fallback, pushes it.
    FeedTree {
        src: u16,
        back: u16,
        acc: u16,
        point: u32,
        skip: u32,
    },
    FeedBodyNext,
    FeedBodyBreak,
    FeedEnd,

    // RangeCheck, Unit, SetMin/SetMax, Reverse, BitWidth, IntToFloat, FloatSqrt, Map/Set
    RangeCheckI {
        dst: u16,
        val: u16,
        lo: u16,
        hi: u16,
        text_idx: u16,
    },
    ConstUnit {
        dst: u16,
    },
    SetMinMaxV {
        dst: u16,
        src_v: u16,
        is_max: bool,
    },
    ReversePlace {
        path: u16,
    },
    BitWidthI {
        dst: u16,
        src: u16,
    },
    IntToFloat {
        dst: u16,
        src: u16,
    },
    FloatSqrt {
        dst: u16,
        src: u16,
    },
    ContainsV {
        dst: u16,
        src_v: u16,
        key_v: u16,
    },
    MapGetV {
        dst: u16,
        map_v: u16,
        key_v: u16,
    },
    MapIndexV {
        dst: u16,
        map_v: u16,
        key_v: u16,
        key_ty: u16,
    },
    SetMapMutPlace {
        dst: u16,
        path: u16,
        val_v: u16,
        is_remove: bool,
    },
    MapPlaceAssign {
        path: u16,
        key_v: u16,
        val_v: u16,
        op: AssignOp,
        val_ty: u16,
    },
    SetMapBoundV {
        dst: u16,
        src_v: u16,
        key_v: u16,
        mode: u8,
    },
    SetMapPopEndPlace {
        dst: u16,
        path: u16,
        is_last: bool,
    },
    LoadList1SmallTuple {
        dst_base: u16,
        len: u8,
        root: u16,
        is_global: bool,
        r_i: u16,
    },
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
    LoadNestedArrI {
        dst: u16,
        root: u16,
        is_global: bool,
        r_i: u16,
        r_j: u16,
    },
    SeqIdxEndOptI {
        dst_val: u16,
        dst_some: u16,
        root: u16,
        is_global: bool,
        r_i: u16,
        is_last: bool,
    },

    // Seamless fallback to tree-walker for any non-lowered IR stmt/expr
    ConstV {
        dst: u16,
        idx: u32,
    },
    Print {
        idx: u32,
    },
    Format {
        idx: u32,
        dst: u16,
    },
    /// Fixes domain `dom` to start at label `lo` and hold `len` positions.
    DefDomain {
        dom: u16,
        lo: u16,
        len: u16,
    },
    /// Fixes domain `dom` to the `(lo, len)` of the domain value in `src`.
    DefDomainFromVal {
        dom: u16,
        src: u16,
    },
    /// `dst` = `Interp::apply` of `applies[idx]` to its operand registers;
    /// `sync` hands the frame to the stack first, for user code it calls.
    ApplyV {
        idx: u32,
        dst: u16,
        sync: bool,
    },
    /// `dst` = the mutation `applies[idx]` (an `ExprKind::Mutate`) of the
    /// place at `path`, given its arguments in the spec's registers.
    MutatePlace {
        idx: u32,
        path: u16,
        dst: u16,
    },
    FallbackStmt {
        idx: u32,
        clear_from_v: u16,
        in_loop: bool,
    },
    FallbackExprI {
        idx: u32,
        dst: u16,
        in_loop: bool,
    },
    FallbackExprV {
        idx: u32,
        dst: u16,
        in_loop: bool,
    },
    /// A traced run tells the tracer of a write the VM just did itself.
    Trace {
        idx: u32,
    },
}

/// A write a traced run reports: the same steps the tree-walker's
/// `Stmt::Let`, `Stmt::Assign` and `for` rounds record.
pub(crate) enum TracePoint<'p> {
    Let(Slot, &'p Expr),
    Assign(&'p Place),
    /// A method that changed `place` in place, as `xs.push(x)`.
    Mutated(&'p Place, Span),
    /// A write to a global or to a cell of one, with the registers its
    /// indices were evaluated into again after the write.
    Cell(&'p Place, Vec<u16>),
    Loop(Slot, Span),
    /// After the loop the variable keeps its last value, not one past the end.
    LoopEnd(Slot),
    /// The back edge of a `while`: a loop that writes nothing still meets the time limit.
    Tick,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum CapSrc {
    GlobalRef(u16),
    GlobalVal(u16),
    LocalI(u16, ScalarKind),
    LocalOptI {
        val: u16,
        some: u16,
        kind: ScalarKind,
    },
    LocalSmallTuple {
        base: u16,
        len: u8,
        kinds: [SmallFieldKind; 4],
    },
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

/// An expression applied to the values of its operands, in `regs`.
pub(crate) struct ApplySpec<'p> {
    pub expr: &'p Expr,
    pub regs: Vec<u16>,
}

/// One of main's loops, as a checkpoint finds it in the registers.
#[derive(Clone, Copy, Debug)]
pub(crate) enum LoopReg {
    While(cpl_ir::LoopId),
    /// `for v in lo..hi` (or `..=`): the variable and the end.
    Range {
        id: cpl_ir::LoopId,
        r_var: u16,
        r_hi: u16,
    },
    /// `for v in seq` (or `seq.rev`): the position and where the sequence
    /// is held.
    Seq {
        id: cpl_ir::LoopId,
        r_idx: u16,
        held: SeqHeld,
    },
}

impl LoopReg {
    pub(crate) fn id(&self) -> cpl_ir::LoopId {
        match *self {
            LoopReg::While(id) | LoopReg::Range { id, .. } | LoopReg::Seq { id, .. } => id,
        }
    }
}

/// Where a `for` loop over a sequence holds it.
#[derive(Clone, Copy, Debug)]
pub(crate) enum SeqHeld {
    Local(u16),
    Global(u16),
}

/// A jump back to the top of one of main's loops, where a checkpoint can
/// be taken: the loops around it, outermost first, ending with its own.
#[derive(Clone, Debug)]
pub(crate) struct BackEdge {
    pub pc: u32,
    pub loops: Vec<LoopReg>,
}

/// The op of `call`, the one effect of statement `stmt` before its
/// body, where a checkpoint taken inside the callee can stand: the loops
/// around it, outermost first.
#[derive(Clone, Debug)]
pub(crate) struct CallSite<'p> {
    pub pc: u32,
    pub loops: Vec<LoopReg>,
    pub stmt: &'p Stmt,
    pub call: &'p Expr,
    /// The calls `stmt` evaluates before `call`: a run from a checkpoint
    /// takes what they returned instead of making them again.
    pub before: Vec<&'p Expr>,
}

pub(crate) struct Chunk<'p> {
    pub ops: Vec<Op>,
    pub spans: Vec<Span>,
    pub consts_i: Vec<i64>,
    pub range_texts: Vec<&'p str>,
    pub places: Vec<PlacePath>,
    pub closure_specs: Vec<ClosureMakeSpec>,
    pub feed_calls: Vec<FeedCallSpec>,
    /// In a traced chunk, the bodies of fed loops it leaves to the
    /// tree-walker, which runs each from here as the fn yields.
    pub fed_bodies: Vec<(LoopId, u32)>,
    pub consts_v: Vec<Value>,
    pub prints: Vec<PrintSpec<'p>>,
    pub formats: Vec<FormatSpec<'p>>,
    pub applies: Vec<ApplySpec<'p>>,
    pub fallback_stmts: Vec<&'p Stmt>,
    pub fallback_exprs: Vec<&'p Expr>,
    /// The arguments of each `CallTraced` that has `mut` ones.
    pub lent_calls: Vec<&'p [Arg]>,
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
    pub back_edges: Vec<BackEdge>,
    pub call_sites: Vec<CallSite<'p>>,
    /// The call ops, by pc in order, whose result a call site's `before`
    /// needs: where each one's op puts it (`None` for `Unit`), and the call's
    /// address.
    pub keeps: Vec<(u32, Option<SlotLoc>, usize)>,
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

pub(super) fn recognize_fast_closure(
    def: &cpl_ir::ClosureDef,
    program: &Program,
) -> Option<FastClosure> {
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
                let mut fc = Compiler::new(
                    program,
                    feeds,
                    &empty_fed,
                    &f.locals,
                    Some(id),
                    f.params.len(),
                );
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
            if f.locals.len() > 512 || program.judge_fns.contains(&id) {
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
        CompiledProgram {
            main,
            fns,
            fed_fns,
            closures,
            fast_closures,
        }
    }
}
