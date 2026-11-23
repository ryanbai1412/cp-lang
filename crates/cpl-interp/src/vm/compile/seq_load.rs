use super::Compiler;
use crate::vm::ir::*;
use cpl_diag::Span;

/// How a sequence's elements are read by position.
#[derive(Clone, Copy)]
pub(super) enum SeqKind {
    Array,
    List,
    Str,
}

impl<'p, 'f> Compiler<'p, 'f> {
    /// Loads element `r_idx` of a sequence (the global `global_id`, else
    /// the register `local_v`) into a loop variable at `slot_loc`.
    pub(super) fn emit_seq_load(
        &mut self,
        slot_loc: SlotLoc,
        seq_kind: SeqKind,
        global_id: Option<u16>,
        local_v: Option<u16>,
        r_idx: u16,
        span: Span,
    ) {
        match slot_loc {
            SlotLoc::I(r_var, _) => match (seq_kind, global_id) {
                (SeqKind::Array, Some(gid)) => {
                    self.emit(
                        Op::LoadGArr1I {
                            dst: r_var,
                            gid,
                            r_i: r_idx,
                        },
                        span,
                    );
                }
                (SeqKind::Array, None) => {
                    self.emit(
                        Op::LoadLArr1I {
                            dst: r_var,
                            arr_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        span,
                    );
                }
                (SeqKind::List, Some(gid)) => {
                    self.emit(
                        Op::LoadGList1I {
                            dst: r_var,
                            gid,
                            r_i: r_idx,
                        },
                        span,
                    );
                }
                (SeqKind::List, None) => {
                    self.emit(
                        Op::LoadLList1I {
                            dst: r_var,
                            list_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        span,
                    );
                }
                (SeqKind::Str, Some(gid)) => {
                    self.emit(
                        Op::GStrGetI {
                            dst: r_var,
                            gid,
                            r_i: r_idx,
                        },
                        span,
                    );
                }
                (SeqKind::Str, None) => {
                    self.emit(
                        Op::StrGetI {
                            dst: r_var,
                            str_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        span,
                    );
                }
            },
            SlotLoc::V(r_var) => match (seq_kind, global_id) {
                (SeqKind::Array, Some(gid)) => {
                    self.emit(
                        Op::LoadGArr1V {
                            dst: r_var,
                            gid,
                            r_i: r_idx,
                        },
                        span,
                    );
                }
                (SeqKind::Array, None) => {
                    self.emit(
                        Op::LoadLArr1V {
                            dst: r_var,
                            arr_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        span,
                    );
                }
                (SeqKind::List, Some(gid)) => {
                    self.emit(
                        Op::LoadGList1V {
                            dst: r_var,
                            gid,
                            r_i: r_idx,
                        },
                        span,
                    );
                }
                (SeqKind::List, None) => {
                    self.emit(
                        Op::LoadLList1V {
                            dst: r_var,
                            list_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        span,
                    );
                }
                (SeqKind::Str, _) => unreachable!(),
            },
            SlotLoc::SmallTuple { base, len, .. } => match (seq_kind, global_id) {
                (SeqKind::Array, Some(gid)) => {
                    self.emit(
                        Op::LoadGArr1SmallTuple {
                            dst_base: base,
                            len,
                            gid,
                            r_i: r_idx,
                        },
                        span,
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
                        span,
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
                        span,
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
                        span,
                    );
                }
                (SeqKind::Str, _) => unreachable!(),
            },
            SlotLoc::OptI { val, some, .. } => match (seq_kind, global_id) {
                (SeqKind::Array, Some(gid)) => {
                    self.emit(
                        Op::LoadGArr1OptI {
                            dst_val: val,
                            dst_some: some,
                            gid,
                            r_i: r_idx,
                        },
                        span,
                    );
                }
                (SeqKind::Array, None) => {
                    self.emit(
                        Op::LoadLArr1OptI {
                            dst_val: val,
                            dst_some: some,
                            arr_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        span,
                    );
                }
                (SeqKind::List, None) => {
                    self.emit(
                        Op::LoadLList1OptI {
                            dst_val: val,
                            dst_some: some,
                            list_v: local_v.unwrap(),
                            r_i: r_idx,
                        },
                        span,
                    );
                }
                (SeqKind::List, Some(gid)) => {
                    let cell = self.alloc_v();
                    self.emit(
                        Op::LoadGList1V {
                            dst: cell,
                            gid,
                            r_i: r_idx,
                        },
                        span,
                    );
                    self.emit(
                        Op::UnboxOptI {
                            dst_val: val,
                            dst_some: some,
                            src_v: cell,
                        },
                        span,
                    );
                }
                (SeqKind::Str, _) => unreachable!(),
            },
        }
    }
}
