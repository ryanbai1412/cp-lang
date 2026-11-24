use crate::value::SmallFieldKind;
use cpl_diag::Span;
use cpl_ir::{AssignOp, BinOp, CmpOp, Expr, FmtArg, FmtPiece, FnId, LoopId, Place, Slot};

pub(crate) use super::program::*;
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
    /// An element of an array of 3 or more dims, its `n` indices in the
    /// registers from the first.
    IndexN(u16, u16),
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
    /// With `rev` the values are walked backward, and `r_hi` is set to the
    /// first of them, the last value in range.
    ForStepEnter {
        r_lo: u16,
        r_hi: u16,
        r_step: u16,
        closed: bool,
        rev: bool,
        skip: u32,
    },
    /// Steps `r_i` by `r_step` and jumps to `target` while still in range:
    /// below `r_end` (`..=r_end` when `closed`), or with `rev` down by
    /// `r_step` while at least `r_end`.
    ForStepNext {
        r_i: u16,
        r_end: u16,
        r_step: u16,
        closed: bool,
        rev: bool,
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
    /// `dst = arr_v[idx..idx + n]`: an element of an `n`-D array.
    LoadArrNV {
        dst: u16,
        arr_v: u16,
        idx: u16,
        n: u16,
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
    /// Stops the run with the report of `fails[idx]`: an `assert` whose
    /// condition failed, its message in `msg_v` unless that is `u16::MAX`,
    /// or a `Trap`.
    Fail {
        idx: u32,
        msg_v: u16,
    },
    /// `place op= r_val` for a boxed value, combined as the tree-walker does.
    AssignPlaceV {
        path: u16,
        r_val: u16,
        op: AssignOp,
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
    /// `lo..hi` with `F64` (or else `I128`) ends, held in value registers.
    MakeRangeV {
        dst: u16,
        lo: u16,
        hi: u16,
        float: bool,
    },
    /// A domain as a value: its current `(lo, len)`.
    DomainValV {
        dst: u16,
        dom: u16,
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

    /// `dst` = the zero value of `fill_tys[ty]`, what a `let` without a value holds.
    ZeroV {
        dst: u16,
        ty: u16,
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
    /// A user statement starts: the profiler's step (see `Observer::step`).
    Step {
        idx: u32,
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
    /// A local captured by reference: its stack slot, and the register that
    /// holds it when it is boxed (`u16::MAX` for a scalar), which may already
    /// be an alias of a place outside the frame.
    LocalRef {
        slot: u16,
        v: u16,
    },
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
    /// Each `mut` parameter, by position, and where its value goes back
    /// when the loop ends.
    pub muts: Vec<(usize, FeedBack)>,
}

/// Where a `mut` argument of a fed call is stored back.
#[derive(Clone, Copy, Debug)]
pub(crate) enum FeedBack {
    I(u16),
    OptI(u16, u16),
    Place(u16),
    PlaceI(u16, ScalarKind),
    PlaceOptI(u16, ScalarKind),
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
