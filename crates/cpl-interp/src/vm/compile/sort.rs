use super::seq_load::SeqKind;
use super::Compiler;
use crate::vm::helpers::ty_uses_structural_cmp;
use crate::vm::ir::*;
use cpl_diag::Span;
use cpl_ir::{Expr, Place, Slot, Ty};

impl<'p, 'f> Compiler<'p, 'f> {
    /// `xs.sort(by: x => key)`: takes `xs`, reads each element's key with
    /// the element in `slot`, then puts the elements back in a stable order
    /// of their keys, as the tree-walker does.
    pub(super) fn try_compile_sort_key_stmt(
        &mut self,
        place: &'p Place,
        slot: Slot,
        key: &'p Expr,
        desc: Option<&'p Expr>,
        span: Span,
    ) -> bool {
        let seq_kind = match &place.ty {
            Ty::List(_) => SeqKind::List,
            Ty::Array { dims, .. } if dims.len() == 1 => SeqKind::Array,
            _ => return false,
        };
        if !ty_uses_structural_cmp(&key.ty, self.program) {
            return false;
        }
        let slot_loc = self.slot_loc(slot);
        let saved_ops = self.ops.len();
        let saved_spans = self.spans.len();
        let saved_places = self.places.len();
        let desc_r = match desc {
            Some(e) => self.compile_expr_i(e),
            None => u16::MAX,
        };
        let Some(path) = self.compile_place_path(place) else {
            self.ops.truncate(saved_ops);
            self.spans.truncate(saved_spans);
            self.places.truncate(saved_places);
            return false;
        };
        let r_seq = self.alloc_v();
        self.emit(Op::TakePlaceV { dst: r_seq, path }, span);
        let r_keys = self.alloc_v();
        self.emit(
            Op::MakeList {
                dst: r_keys,
                base_v: 0,
                len: 0,
            },
            span,
        );
        let r_len = self.alloc_i();
        match seq_kind {
            SeqKind::Array => self.emit(
                Op::LArrLen {
                    dst: r_len,
                    arr_v: r_seq,
                },
                span,
            ),
            _ => self.emit(
                Op::LListLen {
                    dst: r_len,
                    list_v: r_seq,
                },
                span,
            ),
        };
        let r_idx = self.alloc_i();
        let r_one = self.alloc_i();
        self.emit(Op::ConstImmI { dst: r_idx, val: 0 }, span);
        self.emit(Op::ConstImmI { dst: r_one, val: 1 }, span);
        let j_skip = self.emit(
            Op::JmpGeI {
                a: r_idx,
                b: r_len,
                target: 0,
            },
            span,
        );
        let body_pc = self.ops.len() as u32;
        let saved_pinned = self.pinned_v;
        self.pinned_v = self.pinned_v.max(r_seq.max(r_keys) + 1);
        self.emit_seq_load(slot_loc, seq_kind, None, Some(r_seq), r_idx, span);
        if let Some(kind) = ScalarKind::of_ty(&key.ty, self.program) {
            let val_i = self.compile_expr_i(key);
            self.emit(
                Op::ListPushLI {
                    list_v: r_keys,
                    val_i,
                    kind,
                },
                span,
            );
        } else {
            let val_v = self.compile_expr_v(key);
            self.emit(
                Op::ListPushLV {
                    list_v: r_keys,
                    val_v,
                },
                span,
            );
        }
        self.pinned_v = saved_pinned;
        self.emit(
            Op::AddI {
                dst: r_idx,
                a: r_idx,
                b: r_one,
            },
            span,
        );
        self.emit(
            Op::JmpLtI {
                a: r_idx,
                b: r_len,
                target: body_pc,
            },
            span,
        );
        let end_pc = self.ops.len() as u32;
        self.patch(j_skip, end_pc);
        self.emit(
            Op::SortByKeysPlace {
                path,
                seq_v: r_seq,
                keys_v: r_keys,
                desc_r,
            },
            span,
        );
        true
    }
}
