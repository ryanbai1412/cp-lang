use crate::observe::Observer;
use crate::value::Value;
use crate::vm::helpers::*;
use crate::vm::ir::*;
use crate::{Interp, Res};
use cpl_ir::LoopId;

impl<'p, 'io, O: Observer> Interp<'p, 'io, O> {
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub(super) fn exec_ext_op(
        &mut self,
        op: Op,
        cur_pc: usize,
        compiled: &CompiledProgram<'p>,
        chunk: &Chunk<'p>,
        base_i: usize,
        vm_regs_i: &mut Vec<i64>,
        regs_v: &mut [Value],
        escaped: &mut Option<(bool, Option<LoopId>)>,
        fed: &mut Option<&mut FedCtx<'_, 'p>>,
    ) -> Res<Option<ExecExit>> {
        match op {
            Op::FieldOptI { .. }
            | Op::FieldV { .. }
            | Op::LoadFieldArr1I { .. }
            | Op::FieldArrLen { .. }
            | Op::StrGetI { .. }
            | Op::GStrGetI { .. }
            | Op::MakeTuple { .. }
            | Op::MakeTupleI { .. }
            | Op::MakeList { .. }
            | Op::ListFilled { .. }
            | Op::ToList { .. }
            | Op::MakeFill { .. }
            | Op::ListSlice { .. }
            | Op::StrSlice { .. }
            | Op::TakeLArr1V { .. }
            | Op::TakeGArr1V { .. }
            | Op::TakeLList1V { .. }
            | Op::TakeGList1V { .. }
            | Op::LoadLArr2V { .. }
            | Op::LoadGArr2V { .. }
            | Op::LoadArrNV { .. }
            | Op::StoreLArr2V { .. }
            | Op::StoreGArr2V { .. }
            | Op::ConstF64 { .. }
            | Op::FloatBin { .. }
            | Op::FloatMinMax { .. }
            | Op::CmpV { .. }
            | Op::CallVmI { .. }
            | Op::CallVmMix { .. }
            | Op::CallTraced { .. }
            | Op::CallClosure { .. }
            | Op::RetI { .. }
            | Op::RetV { .. }
            | Op::RetSmallTuple { .. }
            | Op::RetUnit
            | Op::IsSomeV { .. }
            | Op::UnwrapV { .. }
            | Op::UnwrapVI { .. }
            | Op::WrapSomeV { .. }
            | Op::ConstNoneV { .. }
            | Op::SeqLastV { .. }
            | Op::SeqLastOptI { .. }
            | Op::SeqFirstV { .. }
            | Op::SeqFirstOptI { .. } => {
                self.exec_fields_and_calls(op, cur_pc, compiled, chunk, base_i, vm_regs_i, regs_v)
            }
            Op::SeqLenI { .. }
            | Op::CutI { .. }
            | Op::RangeLoI { .. }
            | Op::RangeHiI { .. }
            | Op::RangeBoundsI { .. }
            | Op::BitCeilI { .. }
            | Op::MinMaxGArr1SmallTuple { .. }
            | Op::MinMaxLArr1SmallTuple { .. }
            | Op::ListPushGArr1I { .. }
            | Op::ListPushGArr1V { .. }
            | Op::ListPushLArr1I { .. }
            | Op::ListPushLArr1V { .. }
            | Op::ListPushLV { .. }
            | Op::ListPushGV { .. }
            | Op::ListPushLI { .. }
            | Op::ListPushGI { .. }
            | Op::ListPopLV { .. }
            | Op::ListPopGV { .. }
            | Op::ListPopOptIL { .. }
            | Op::ListPopOptIG { .. }
            | Op::ListPopDiscardL { .. }
            | Op::ListPopDiscardG { .. }
            | Op::LoadGlobalVMove { .. }
            | Op::StoreLFieldIdx1I { .. }
            | Op::StoreLFieldIdx1V { .. }
            | Op::LoadFieldList1SmallTuple { .. }
            | Op::FieldListLen { .. }
            | Op::StoreLFieldIdx1SmallTuple { .. }
            | Op::StoreLFieldIdx1OptI { .. }
            | Op::ListPushFieldLI { .. }
            | Op::ListPushFieldLV { .. }
            | Op::ListPushFieldSmallTuple { .. }
            | Op::ListPopDiscardFieldL { .. }
            | Op::StoreGList1V { .. }
            | Op::StoreLList1V { .. }
            | Op::LoadGList1V { .. }
            | Op::LoadLList1V { .. }
            | Op::LoadLList1OptI { .. }
            | Op::SeqItems { .. }
            | Op::TreeRangeItems { .. }
            | Op::ListPopFieldOptIL { .. }
            | Op::MakeRange { .. }
            | Op::CmpOptI { .. }
            | Op::CmpSmallTuple { .. }
            | Op::LoadGIdxFieldI { .. }
            | Op::StoreGIdxFieldI { .. }
            | Op::LoadLIdxFieldI { .. }
            | Op::StoreLIdxFieldI { .. } => {
                let regs_i = &mut vm_regs_i[base_i..base_i + chunk.num_i];
                self.exec_seq_and_list(op, cur_pc, chunk, regs_i, regs_v)?;
                Ok(None)
            }
            _ => self.exec_places_and_fallback(
                op, cur_pc, compiled, chunk, base_i, vm_regs_i, regs_v, escaped, fed,
            ),
        }
    }
}
