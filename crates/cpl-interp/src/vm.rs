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

use crate::value::{Array, ArrayData, OptVal, SmallFieldKind, TupleVal, Value};
use crate::{describe, trap, Flow, Interp, Res};
use cpl_diag::Span;
use cpl_ir::{
    Arg, AssignOp, BinOp, Block, Builtin, Cast, CmpOp, Expr, ExprKind, FnId, Iter, Local, LoopId,
    Mutation, Place, PlaceKind, Program, Slot, Stmt, Ty, UnOp,
};
use std::rc::Rc;

/// How an unboxed `i64` register maps back to a [`Value`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ScalarKind {
    Int,
    Bool,
    Char,
    OneIntStruct,
}

impl ScalarKind {
    pub fn of_ty(ty: &Ty, program: &Program) -> Option<ScalarKind> {
        match ty {
            Ty::Int | Ty::Pos(_) | Ty::Bound(_) => Some(ScalarKind::Int),
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
        };
    }
    Some((len as u8, kinds))
}

/// Where a local `Slot` lives inside a VM frame.
#[derive(Clone, Copy, Debug)]
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
    ShlI { dst: u16, a: u16, b: u16 },
    ShrI { dst: u16, a: u16, b: u16 },
    NotBool { dst: u16, a: u16 },
    MinI { dst: u16, a: u16, b: u16 },
    MaxI { dst: u16, a: u16, b: u16 },
    AbsI { dst: u16, a: u16 },

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

    // Seamless fallback to tree-walker for any non-lowered IR stmt/expr
    FallbackStmt { idx: u32, clear_from_v: u16 },
    FallbackExprI { idx: u32, dst: u16 },
    FallbackExprV { idx: u32, dst: u16 },
}

pub(crate) struct Chunk<'p> {
    pub ops: Vec<Op>,
    pub spans: Vec<Span>,
    pub consts_i: Vec<i64>,
    pub fallback_stmts: Vec<&'p Stmt>,
    pub fallback_exprs: Vec<&'p Expr>,
    pub num_i: usize,
    pub num_v: usize,
    pub num_local_v: usize,
    pub slot_map: Vec<SlotLoc>,
    pub param_count: usize,
    pub ret_kind: Option<ScalarKind>,
    pub guarded_domains: Vec<cpl_ir::DomainId>,
}

pub(crate) struct CompiledProgram<'p> {
    pub main: Chunk<'p>,
    pub fns: Vec<Option<Chunk<'p>>>,
    pub closures: Vec<Option<Chunk<'p>>>,
}

impl<'p> CompiledProgram<'p> {
    pub fn compile(program: &'p Program) -> Self {
        let mut fns = Vec::with_capacity(program.fns.len());
        for (id, f) in program.fns.iter().enumerate() {
            let guarded_domains = program.fn_guarded_domains(&f.ret, &f.body);
            if f.locals.len() > 512 {
                fns.push(None);
            } else {
                let mut c = Compiler::new(program, &f.locals, Some(id), f.params.len());
                let ret_kind = ScalarKind::of_ty(&f.ret, program);
                c.compile_block(&f.body, ret_kind, true);
                let mut chunk = c.finish();
                chunk.param_count = f.params.len();
                chunk.ret_kind = ret_kind;
                chunk.guarded_domains = guarded_domains;
                fns.push(Some(chunk));
            }
        }
        let mut closures = Vec::with_capacity(program.closures.len());
        for def in &program.closures {
            let can_compile = def.locals.len() <= 512
                && def.muts.is_empty()
                && def.captures.iter().all(|cap| !cap.by_ref)
                && def.params.iter().enumerate().all(|(i, &s)| s == i);
            if !can_compile {
                closures.push(None);
            } else {
                let guarded_domains = program.fn_guarded_domains(&def.ret, &def.body);
                let mut c = Compiler::new(program, &def.locals, None, def.params.len());
                let ret_kind = ScalarKind::of_ty(&def.ret, program);
                c.compile_block(&def.body, ret_kind, true);
                let mut chunk = c.finish();
                chunk.param_count = def.params.len();
                chunk.ret_kind = ret_kind;
                chunk.guarded_domains = guarded_domains;
                closures.push(Some(chunk));
            }
        }
        let mut main_c = Compiler::new(program, &program.main_locals, None, 0);
        main_c.compile_block(&program.main, None, false);
        main_c.emit(Op::RetUnit, Span::default());
        let main = main_c.finish();
        CompiledProgram { main, fns, closures }
    }
}

struct LoopTargets {
    id: LoopId,
    breaks: Vec<usize>,
    continues: Vec<usize>,
}

#[derive(Clone, Copy)]
enum ModSuperOp {
    Add(u32),
    Sub(u32),
    Mul(u32),
    Neg(u32),
    New(u32),
}

struct Compiler<'p> {
    program: &'p Program,
    ops: Vec<Op>,
    spans: Vec<Span>,
    consts_i: Vec<i64>,
    fallback_stmts: Vec<&'p Stmt>,
    fallback_exprs: Vec<&'p Expr>,
    slot_map: Vec<SlotLoc>,
    next_i: u16,
    next_v: u16,
    num_local_v: u16,
    pinned_v: u16,
    loops: Vec<LoopTargets>,
    cur_fn: Option<FnId>,
    inline_slots: Option<Vec<SlotLoc>>,
    inline_ret: Option<(u16, Vec<usize>)>,
}

impl<'p> Compiler<'p> {
    fn new(
        program: &'p Program,
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
            ops: Vec::new(),
            spans: Vec::new(),
            consts_i: Vec::new(),
            fallback_stmts: Vec::new(),
            fallback_exprs: Vec::new(),
            slot_map,
            next_i,
            next_v,
            num_local_v: next_v,
            pinned_v: next_v,
            loops: Vec::new(),
            cur_fn,
            inline_slots: None,
            inline_ret: None,
        }
    }

    fn finish(self) -> Chunk<'p> {
        Chunk {
            ops: self.ops,
            spans: self.spans,
            consts_i: self.consts_i,
            fallback_stmts: self.fallback_stmts,
            fallback_exprs: self.fallback_exprs,
            num_i: self.next_i as usize,
            num_v: self.next_v as usize,
            num_local_v: self.num_local_v as usize,
            slot_map: self.slot_map,
            param_count: 0,
            ret_kind: None,
            guarded_domains: Vec::new(),
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
        if f.locals.len() > 512 || args.len() != f.params.len() || (has_mut && has_fn_param) {
            return None;
        }
        // Check that all mut args are simple local or global places (so move-in/move-out is exact).
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
                        _ => return None,
                    }
                }
            }
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
        if self.cur_fn == Some(fn_id) || self.inline_slots.is_some() {
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
            .all(|l| ScalarKind::of_ty(&l.ty, self.program).is_some())
        {
            return false;
        }
        fn simple_block(b: &Block, program: &Program) -> bool {
            b.stmts.iter().all(|s| simple_stmt(s, program))
        }
        fn simple_stmt(s: &Stmt, program: &Program) -> bool {
            match s {
                Stmt::Let(_, e) => e.as_ref().is_none_or(|e| simple_expr(e, program)),
                Stmt::Assign(p, _, e) => {
                    matches!(p.kind, PlaceKind::Local(_)) && simple_expr(e, program)
                }
                Stmt::Return(Some(e)) | Stmt::Give(e) | Stmt::Expr(e) => simple_expr(e, program),
                Stmt::If(c, t, e) => {
                    if let Some(b) = Compiler::const_bool(c) {
                        if b { simple_block(t, program) } else { simple_block(e, program) }
                    } else {
                        simple_expr(c, program)
                            && simple_block(t, program)
                            && simple_block(e, program)
                    }
                }
                _ => false,
            }
        }
        fn simple_expr(e: &Expr, program: &Program) -> bool {
            if ScalarKind::of_ty(&e.ty, program).is_none() {
                return false;
            }
            match &e.kind {
                ExprKind::Int(_)
                | ExprKind::Bool(_)
                | ExprKind::Char(_)
                | ExprKind::Local(_)
                | ExprKind::Global(_) => true,
                ExprKind::Unary(_, a) | ExprKind::Field(a, _) | ExprKind::Cast(_, a) => {
                    simple_expr(a, program)
                }
                ExprKind::Binary(op, a, b) => {
                    let safe_op = match op {
                        BinOp::Div | BinOp::Rem => matches!(b.kind, ExprKind::Int(v) if v != 0),
                        BinOp::Shl | BinOp::Shr => matches!(b.kind, ExprKind::Int(v) if (0..64).contains(&v)),
                        _ => true,
                    };
                    safe_op && simple_expr(a, program) && simple_expr(b, program)
                }
                ExprKind::Cmp(_, a, b) => {
                    simple_expr(a, program) && simple_expr(b, program)
                }
                ExprKind::Tuple(fs) => fs.len() == 1 && simple_expr(&fs[0], program),
                ExprKind::Index(base, indices) => {
                    matches!(base.ty, Ty::Array { .. })
                        && (indices.len() == 1 || indices.len() == 2)
                        && matches!(base.kind, ExprKind::Global(_))
                        && indices.iter().all(|i| simple_expr(i, program))
                }
                ExprKind::Call(callee, args) => {
                    Compiler::recognize_mod_op(program, *callee).is_some()
                        && args.iter().all(|a| matches!(a, Arg::Value(e) if simple_expr(e, program)))
                }
                ExprKind::If(c, t, e) => {
                    if let Some(b) = Compiler::const_bool(c) {
                        if b { simple_expr(t, program) } else { simple_expr(e, program) }
                    } else {
                        simple_expr(c, program)
                            && simple_expr(t, program)
                            && simple_expr(e, program)
                    }
                }
                ExprKind::Do(b) => simple_block(b, program),
                _ => false,
            }
        }
        simple_block(&f.body, self.program)
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
            ExprKind::Read(_) => true,
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

    fn compile_stmts(&mut self, stmts: &'p [Stmt]) {
        let has_closure = Self::stmts_have_closure(stmts);
        let mut declared = Vec::new();
        for (i, stmt) in stmts.iter().enumerate() {
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
        match stmt {
            Stmt::Let(slot, init) => match self.slot_loc(*slot) {
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
            },
            Stmt::Expr(expr) => {
                if ScalarKind::of_ty(&expr.ty, self.program).is_some() {
                    let _ = self.compile_expr_i(expr);
                } else if let ExprKind::Mutate(op, place, args) = &expr.kind {
                    if !self.try_compile_mutate_stmt(*op, place, args, expr.span) {
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
                if let Some((ret_dst, _)) = self.inline_ret {
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
                if let Some((ret_dst, _)) = self.inline_ret {
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

    fn try_compile_for(
        &mut self,
        id: LoopId,
        slot: Slot,
        iter: &'p Iter,
        body: &'p Block,
    ) -> bool {
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
            Iter::Seq(seq_expr, rev) => {
                let is_array = matches!(
                    &seq_expr.ty,
                    Ty::Array { dims, elem }
                        if dims.len() == 1 && ScalarKind::of_ty(elem, self.program).is_some()
                );
                let is_list = matches!(
                    &seq_expr.ty,
                    Ty::List(elem) if ScalarKind::of_ty(elem, self.program).is_some()
                ) && matches!(seq_expr.kind, ExprKind::Local(_) | ExprKind::Global(_));
                if !is_array && !is_list {
                    return false;
                }
                let r_idx = self.alloc_i();
                let r_len = self.alloc_i();
                let global_id = match &seq_expr.kind {
                    ExprKind::Global(gid) => Some(*gid as u16),
                    _ => None,
                };
                let local_v = if global_id.is_none() {
                    Some(self.compile_expr_v(seq_expr))
                } else {
                    None
                };
                match (is_array, global_id) {
                    (true, Some(gid)) => self.emit(Op::GArrLen { dst: r_len, gid }, seq_expr.span),
                    (true, None) => self.emit(
                        Op::LArrLen { dst: r_len, arr_v: local_v.unwrap() },
                        seq_expr.span,
                    ),
                    (false, Some(gid)) => self.emit(Op::GListLen { dst: r_len, gid }, seq_expr.span),
                    (false, None) => self.emit(
                        Op::LListLen { dst: r_len, list_v: local_v.unwrap() },
                        seq_expr.span,
                    ),
                };
                let r_zero = self.alloc_i();
                let j_skip = if *rev {
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
                match (is_array, global_id) {
                    (true, Some(gid)) => self.emit(
                        Op::LoadGArr1I { dst: r_var, gid, r_i: r_idx },
                        seq_expr.span,
                    ),
                    (true, None) => self.emit(
                        Op::LoadLArr1I { dst: r_var, arr_v: local_v.unwrap(), r_i: r_idx },
                        seq_expr.span,
                    ),
                    (false, Some(gid)) => self.emit(
                        Op::LoadGList1I { dst: r_var, gid, r_i: r_idx },
                        seq_expr.span,
                    ),
                    (false, None) => self.emit(
                        Op::LoadLList1I { dst: r_var, list_v: local_v.unwrap(), r_i: r_idx },
                        seq_expr.span,
                    ),
                };
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
                if *rev {
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
                true
            }
            _ => false,
        }
    }

    fn try_compile_assign(&mut self, place: &'p Place, op: AssignOp, value: &'p Expr) -> bool {
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
                if matches!(base.ty, Ty::Array { .. }) && indices.len() == 1 =>
            {
                match &base.kind {
                    PlaceKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_v(value);
                        self.emit(Op::StoreGArr1V { gid: *gid as u16, r_i, r_val }, place.span);
                        true
                    }
                    PlaceKind::Local(slot) => {
                        let SlotLoc::V(arr_v) = self.slot_loc(*slot) else {
                            return false;
                        };
                        let r_i = self.compile_expr_i(&indices[0]);
                        let r_val = self.compile_expr_v(value);
                        self.emit(Op::StoreLArr1V { arr_v, r_i, r_val }, place.span);
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
            (PlaceKind::Field(obj_place, field), AssignOp::Set, Some(kind), _) => {
                let PlaceKind::Local(slot) = obj_place.kind else {
                    return false;
                };
                let SlotLoc::V(obj_v) = self.slot_loc(slot) else {
                    return false;
                };
                let src_i = self.compile_expr_i(value);
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
            ExprKind::Builtin(Builtin::Last, args) if args.len() == 1 => {
                let seq_v = self.compile_expr_v(&args[0]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(Op::SeqLastOptI { dst_val, dst_some, seq_v }, span);
                (dst_val, dst_some)
            }
            ExprKind::Builtin(Builtin::First, args) if args.len() == 1 => {
                let seq_v = self.compile_expr_v(&args[0]);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(Op::SeqFirstOptI { dst_val, dst_some, seq_v }, span);
                (dst_val, dst_some)
            }
            ExprKind::Mutate(Mutation::Pop, place, args) if args.is_empty() => match &place.kind {
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
                    _ => {
                        let src_v = self.emit_fallback_expr_v(expr);
                        let dst_val = self.alloc_i();
                        let dst_some = self.alloc_i();
                        self.emit(Op::UnboxOptI { dst_val, dst_some, src_v }, span);
                        (dst_val, dst_some)
                    }
                }
            }
            ExprKind::Field(base, idx) => {
                let obj_v = self.compile_expr_v(base);
                let dst_val = self.alloc_i();
                let dst_some = self.alloc_i();
                self.emit(
                    Op::FieldOptI { dst_val, dst_some, obj_v, field: *idx as u16 },
                    span,
                );
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
                if ScalarKind::of_ty(&lhs.ty, self.program).is_some()
                    && ScalarKind::of_ty(&rhs.ty, self.program).is_some()
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
            ExprKind::Cast(Cast::Ord | Cast::Trust | Cast::Raw | Cast::CharToInt, inner) => {
                self.compile_expr_i(inner)
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
                    ExprKind::Field(obj, fidx) if indices.len() == 1 => {
                        let obj_v = self.compile_expr_v(obj);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        self.emit(
                            Op::LoadFieldArr1I { dst, obj_v, field: *fidx as u16, r_i },
                            span,
                        );
                        dst
                    }
                    _ => self.emit_fallback_expr_i(expr),
                }
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::List(_)) && indices.len() == 1 =>
            {
                match &base.kind {
                    ExprKind::Global(gid) => {
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
                    ExprKind::Field(obj, fidx) => {
                        let obj_v = self.compile_expr_v(obj);
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_i();
                        self.emit(
                            Op::LoadFieldList1I { dst, obj_v, field: *fidx as u16, r_i },
                            span,
                        );
                        dst
                    }
                    _ => self.emit_fallback_expr_i(expr),
                }
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::Str) && indices.len() == 1 =>
            {
                let r_i = self.compile_expr_i(&indices[0]);
                let dst = self.alloc_i();
                if let ExprKind::Global(gid) = base.kind {
                    self.emit(Op::GStrGetI { dst, gid: gid as u16, r_i }, span);
                } else {
                    let str_v = self.compile_expr_v(base);
                    self.emit(Op::StrGetI { dst, str_v, r_i }, span);
                }
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
            ExprKind::Do(block) => {
                let dst = self.alloc_i();
                let prev_ret = self.inline_ret.replace((dst, Vec::new()));
                for s in &block.stmts {
                    self.compile_stmt(s);
                }
                let end_pc = self.ops.len() as u32;
                let (_, jumps) = self.inline_ret.take().unwrap();
                self.inline_ret = prev_ret;
                for j in jumps {
                    self.patch(j, end_pc);
                }
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
                if self.is_pure_leaf_fn(*fn_id) {
                    let f = &self.program.fns[*fn_id];
                    let mut arg_regs = Vec::with_capacity(args.len());
                    for a in args {
                        let Arg::Value(ae) = a else {
                            return self.emit_fallback_expr_i(expr);
                        };
                        arg_regs.push(self.compile_expr_i(ae));
                    }
                    let mut map = Vec::with_capacity(f.locals.len());
                    for (i, l) in f.locals.iter().enumerate() {
                        let kind = ScalarKind::of_ty(&l.ty, self.program).unwrap();
                        let reg = self.alloc_i();
                        if i < arg_regs.len() {
                            self.emit(Op::MoveI { dst: reg, src: arg_regs[i] }, span);
                        } else {
                            self.emit(Op::ConstImmI { dst: reg, val: 0 }, span);
                        }
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
                    let can_vm = !f.params.iter().any(|p| p.mutable)
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
                dst
            }
            ExprKind::Index(base, indices)
                if matches!(base.ty, Ty::Array { .. }) && indices.len() == 1 =>
            {
                match &base.kind {
                    ExprKind::Global(gid) => {
                        let r_i = self.compile_expr_i(&indices[0]);
                        let dst = self.alloc_v();
                        self.emit(Op::LoadGArr1V { dst, gid: *gid as u16, r_i }, span);
                        dst
                    }
                    ExprKind::Local(slot) => {
                        if let SlotLoc::V(arr_v) = self.slot_loc(*slot) {
                            let r_i = self.compile_expr_i(&indices[0]);
                            let dst = self.alloc_v();
                            self.emit(Op::LoadLArr1V { dst, arr_v, r_i }, span);
                            dst
                        } else {
                            self.emit_fallback_expr_v(expr)
                        }
                    }
                    _ => self.emit_fallback_expr_v(expr),
                }
            }
            ExprKind::Tuple(fields)
                if fields.len() >= 2
                    && fields.len() <= 16
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
            ExprKind::Mutate(Mutation::Pop, place, args) if args.is_empty() => match &place.kind {
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
                _ => self.emit_fallback_expr_v(expr),
            },
            ExprKind::Call(fn_id, args) => {
                if let Some((_, dst_v, _)) =
                    self.try_compile_call_mix(*fn_id, args, CallRet::V, span)
                {
                    dst_v
                } else {
                    self.emit_fallback_expr_v(expr)
                }
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
            _ => false,
        }
    }

    fn emit_fallback_stmt(&mut self, stmt: &'p Stmt) {
        let idx = self.fallback_stmts.len() as u32;
        self.fallback_stmts.push(stmt);
        self.emit(
            Op::FallbackStmt { idx, clear_from_v: self.pinned_v },
            stmt.span().unwrap_or_default(),
        );
    }

    fn emit_fallback_expr_i(&mut self, expr: &'p Expr) -> u16 {
        let idx = self.fallback_exprs.len() as u32;
        self.fallback_exprs.push(expr);
        let dst = self.alloc_i();
        self.emit(Op::FallbackExprI { idx, dst }, expr.span);
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
        self.emit(Op::FallbackExprV { idx, dst }, expr.span);
        dst
    }
}

fn sync_to_stack(
    slot_map: &[SlotLoc],
    regs_i: &[i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
) {
    for (slot, loc) in slot_map.iter().enumerate() {
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

fn sync_from_stack(
    slot_map: &[SlotLoc],
    regs_i: &mut [i64],
    regs_v: &mut [Value],
    stack: &mut [Value],
    base: usize,
) {
    for (slot, loc) in slot_map.iter().enumerate() {
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
        let mut regs_v = if chunk.num_v > 0 {
            vec![Value::Unit; chunk.num_v]
        } else {
            Vec::new()
        };
        sync_from_stack(
            &chunk.slot_map,
            &mut vm_regs_i[base_i..base_i + chunk.num_i],
            &mut regs_v,
            &mut self.stack,
            self.base,
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
        let mut regs_v = if chunk.num_v > 0 {
            vec![Value::Unit; chunk.num_v]
        } else {
            Vec::new()
        };
        let dom_mark = self.saved_domains.len();
        for &d in &chunk.guarded_domains {
            self.saved_domains.push((d, self.domains[d]));
        }
        let res = self.exec_chunk_inner(compiled, chunk, base_i, vm_regs_i, &mut regs_v);
        while self.saved_domains.len() > dom_mark {
            let (id, prev) = self.saved_domains.pop().unwrap();
            self.domains[id] = prev;
        }
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
        let mut regs_v = if chunk.num_v > 0 {
            vec![Value::Unit; chunk.num_v]
        } else {
            Vec::new()
        };
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
        let ops = chunk.ops.as_slice();
        let spans = chunk.spans.as_slice();
        let mut pc: usize = 0;
        let mut regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];

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
                    match av.checked_add(bv) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} + {bv}`")),
                    }
                }
                Op::SubI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    match av.checked_sub(bv) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} - {bv}`")),
                    }
                }
                Op::MulI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    match av.checked_mul(bv) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} * {bv}`")),
                    }
                }
                Op::DivI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    if bv == 0 {
                        return trap(spans[cur_pc], format!("division by zero: `{av} / 0`"));
                    }
                    match av.checked_div_euclid(bv) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} / {bv}`")),
                    }
                }
                Op::RemI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    if bv == 0 {
                        return trap(spans[cur_pc], format!("remainder by zero: `{av} % 0`"));
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
                    match av.checked_add(bv) {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `{av} + {bv}`")),
                    }
                }
                Op::NegI { dst, a } => {
                    let av = regs_i[a as usize];
                    match av.checked_neg() {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: `-{av}`")),
                    }
                }
                Op::BitAndI { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize] & regs_i[b as usize];
                }
                Op::BitOrI { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize] | regs_i[b as usize];
                }
                Op::BitXorI { dst, a, b } => {
                    regs_i[dst as usize] = regs_i[a as usize] ^ regs_i[b as usize];
                }
                Op::ShlI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    if !(0..64).contains(&bv) {
                        return trap(spans[cur_pc], format!("shift by {bv} is outside 0..64"));
                    }
                    if av < 0 {
                        return trap(spans[cur_pc], format!("left shift of negative number {av}"));
                    }
                    if bv > 0 && av > (i64::MAX >> bv) {
                        return trap(spans[cur_pc], format!("integer overflow: {av} << {bv}"));
                    }
                    regs_i[dst as usize] = av << bv;
                }
                Op::ShrI { dst, a, b } => {
                    let av = regs_i[a as usize];
                    let bv = regs_i[b as usize];
                    if !(0..64).contains(&bv) {
                        return trap(spans[cur_pc], format!("shift by {bv} is outside 0..64"));
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
                Op::AbsI { dst, a } => {
                    let av = regs_i[a as usize];
                    match av.checked_abs() {
                        Some(v) => regs_i[dst as usize] = v,
                        None => return trap(spans[cur_pc], format!("integer overflow: abs({av})")),
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
                        return trap(spans[cur_pc], "unwrapped none");
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
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
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
                Op::StoreLArr1I { arr_v, r_i, r_val } => {
                    let idx = regs_i[r_i as usize];
                    let val = regs_i[r_val as usize];
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
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
                    if let Value::Array(arr) = &regs_v[arr_v as usize] {
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
                Op::StoreLArr1V { arr_v, r_i, r_val } => {
                    let idx = regs_i[r_i as usize];
                    let val = if r_val as usize >= chunk.num_local_v {
                        std::mem::replace(&mut regs_v[r_val as usize], Value::Unit)
                    } else {
                        regs_v[r_val as usize].clone()
                    };
                    if let Value::Array(arr) = &mut regs_v[arr_v as usize] {
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
                    if let Value::Str(s) = &regs_v[str_v as usize] {
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
                    } else {
                        return trap(spans[cur_pc], "internal: expected string");
                    }
                }
                Op::GStrGetI { dst, gid, r_i } => {
                    let idx = regs_i[r_i as usize];
                    if let Value::Str(s) = &self.globals[gid as usize] {
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
                    } else {
                        return trap(spans[cur_pc], "internal: expected string");
                    }
                }
                Op::MakeTuple { dst, base_v, len } => {
                    let b = base_v as usize;
                    let n = len as usize;
                    let mut fs = Vec::with_capacity(n);
                    for k in 0..n {
                        fs.push(std::mem::replace(&mut regs_v[b + k], Value::Unit));
                    }
                    regs_v[dst as usize] = Value::Tuple(TupleVal::Boxed(Rc::new(fs)));
                }
                Op::CallVmI { fn_id, arg_base, argc, dst } => {
                    let f = &self.program.fns[fn_id as usize];
                    self.stack_left(spans[cur_pc], Some(&f.name))?;
                    let callee = compiled.fns[fn_id as usize].as_ref().unwrap();
                    let ab = arg_base as usize;
                    let mut arg_buf = [0i64; 8];
                    let n = argc as usize;
                    arg_buf[..n].copy_from_slice(&regs_i[ab..ab + n]);
                    let needs_stack = !callee.fallback_stmts.is_empty() || !callee.fallback_exprs.is_empty();
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
                    let needs_stack = !callee.fallback_stmts.is_empty()
                        || !callee.fallback_exprs.is_empty();
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
                        let needs_stack = !callee.fallback_stmts.is_empty()
                            || !callee.fallback_exprs.is_empty();
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
                        let mut c_regs_v = if callee.num_v > 0 {
                            vec![Value::Unit; callee.num_v]
                        } else {
                            Vec::new()
                        };
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
                        let has_frame = self.base + chunk.slot_map.len() <= self.stack.len();
                        if has_frame {
                            sync_to_stack(
                                &chunk.slot_map,
                                regs_i,
                                regs_v,
                                &mut self.stack,
                                self.base,
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
                                self.base,
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
                    return Ok(kind.box_val(raw));
                }
                Op::RetV { src } => {
                    return Ok(std::mem::replace(&mut regs_v[src as usize], Value::Unit));
                }
                Op::RetUnit => return Ok(Value::Unit),
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
                        Value::Range128(lo, hi) => (hi - lo).max(0) as i64,
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
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    };
                    regs_v[dst_v as usize] = Value::Opt(OptVal::from_option(popped));
                }
                Op::ListPopGV { dst_v, gid } => {
                    let popped = match &mut self.globals[gid as usize] {
                        Value::List(items) => Rc::make_mut(items).pop(),
                        _ => return trap(spans[cur_pc], "internal: expected list"),
                    };
                    regs_v[dst_v as usize] = Value::Opt(OptVal::from_option(popped));
                }
                Op::ListPopOptIL { dst_val, dst_some, list_v } => {
                    let popped = match &mut regs_v[list_v as usize] {
                        Value::List(items) => Rc::make_mut(items).pop(),
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
                    _ => return trap(spans[cur_pc], "internal: expected list"),
                },
                Op::ListPopDiscardG { gid } => match &mut self.globals[gid as usize] {
                    Value::List(items) => {
                        Rc::make_mut(items).pop();
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
                Op::FallbackStmt { idx, clear_from_v } => {
                    if let Some(stats) = &mut self.vm_stats {
                        stats.record(stmt_tag(chunk.fallback_stmts[idx as usize]));
                    }
                    for r in &mut regs_v[clear_from_v as usize..] {
                        *r = Value::Unit;
                    }
                    sync_to_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base);
                    let stmt = chunk.fallback_stmts[idx as usize];
                    let flow = self.stmt(stmt)?;
                    sync_from_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base);
                    match flow {
                        Flow::Next => {}
                        Flow::Return(v) | Flow::Give(v) => return Ok(v),
                        Flow::Break(_) | Flow::Continue(_) => {}
                    }
                }
                Op::FallbackExprI { idx, dst } => {
                    if let Some(stats) = &mut self.vm_stats {
                        stats.record(expr_tag(chunk.fallback_exprs[idx as usize], self.program));
                    }
                    sync_to_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base);
                    let expr = chunk.fallback_exprs[idx as usize];
                    let val = self.eval(expr)?;
                    sync_from_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base);
                    regs_i[dst as usize] = ScalarKind::unbox_val(&val);
                }
                Op::FallbackExprV { idx, dst } => {
                    if let Some(stats) = &mut self.vm_stats {
                        stats.record(expr_tag(chunk.fallback_exprs[idx as usize], self.program));
                    }
                    sync_to_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base);
                    let expr = chunk.fallback_exprs[idx as usize];
                    let val = self.eval(expr)?;
                    sync_from_stack(&chunk.slot_map, regs_i, regs_v, &mut self.stack, self.base);
                    regs_v[dst as usize] = val;
                }
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct VmStats {
    pub(crate) total_fallbacks: u64,
    pub(crate) counts: std::collections::BTreeMap<String, u64>,
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
        self.total_fallbacks += 1;
        *self.counts.entry(tag).or_insert(0) += 1;
    }

    pub(crate) fn emit_stderr(&self) {
        let mut entries: Vec<(&str, u64)> =
            self.counts.iter().map(|(k, &v)| (k.as_str(), v)).collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        if std::env::var("CPL_VM_STATS").ok().as_deref() == Some("json") {
            let mut s = format!("{{\"total_fallbacks\":{},\"top\":[", self.total_fallbacks);
            for (i, (k, v)) in entries.iter().take(20).enumerate() {
                if i > 0 {
                    s.push(',');
                }
                let escaped = k.replace('\\', "\\\\").replace('"', "\\\"");
                s.push_str(&format!("[\"{escaped}\",{v}]"));
            }
            s.push_str("]}");
            eprintln!("{s}");
        } else {
            eprintln!("vm-stats: total_fallbacks={}", self.total_fallbacks);
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
        ExprKind::Print { .. } => "Print",
        ExprKind::Debug { .. } => "Debug",
        ExprKind::Format(_) => "Format",
        ExprKind::Some(_) => "Some",
        ExprKind::None => "None",
        ExprKind::IsSome(_) => "IsSome",
        ExprKind::Unwrap(..) => "Unwrap",
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
        _ => format!("expr:{}", expr_tag_short(expr)),
    }
}
