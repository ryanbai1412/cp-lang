use super::seq_load::SeqKind;
use super::Compiler;
use crate::vm::helpers::ty_uses_structural_cmp;
use crate::vm::ir::*;
use cpl_diag::Span;
use cpl_ir::{Expr, ExprKind, Place, Slot, SortBy, Ty};

/// What a keyed sort reads each element's key with.
#[derive(Clone, Copy)]
pub(super) enum SortKey<'p> {
    /// `by: x => key`: `key` with the element in the slot.
    Slot(Slot, &'p Expr),
    /// `by: f`, a fn value from the element to its key.
    Fn(&'p Expr),
}

/// The parts of a keyed sort statement.
pub(super) fn sort_key_of(kind: &ExprKind) -> Option<(&Place, SortKey<'_>, Option<&Expr>)> {
    match kind {
        ExprKind::Sort(place, SortBy::Key(slot, key), desc) => {
            Some((place, SortKey::Slot(*slot, key), desc.as_deref()))
        }
        ExprKind::Sort(
            place,
            SortBy::Fn {
                func,
                is_order: false,
            },
            desc,
        ) => Some((place, SortKey::Fn(func), desc.as_deref())),
        _ => None,
    }
}

impl<'p, 'f> Compiler<'p, 'f> {
    /// `xs.sort(by: ..)`: takes `xs`, reads each element's key, then puts the elements back in a stable order
    /// of their keys, as the tree-walker does.
    pub(super) fn try_compile_sort_key_stmt(
        &mut self,
        place: &'p Place,
        by: SortKey<'p>,
        desc: Option<&'p Expr>,
        span: Span,
    ) -> bool {
        let seq_kind = match &place.ty {
            Ty::List(_) => SeqKind::List,
            Ty::Array { dims, .. } if dims.len() == 1 => SeqKind::Array,
            _ => return false,
        };
        let key_ty = match by {
            SortKey::Slot(_, key) => &key.ty,
            SortKey::Fn(func) => match &func.ty {
                Ty::Fn(params, ret) if params.len() == 1 => &**ret,
                _ => return false,
            },
        };
        if !ty_uses_structural_cmp(key_ty, self.program) {
            return false;
        }
        let key_kind = ScalarKind::of_ty(key_ty, self.program);
        if key_kind.is_none() && ScalarKind::of_opt_ty(key_ty, self.program).is_some() {
            return false;
        }
        let saved_ops = self.ops.len();
        let saved_spans = self.spans.len();
        let saved_places = self.places.len();
        let desc_r = match desc {
            Some(e) => self.compile_expr_i(e),
            None => u16::MAX,
        };
        let Some(path) = self.compile_place_path_to(place, true) else {
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
        let callee_v = match by {
            SortKey::Fn(func) => self.compile_expr_v(func),
            SortKey::Slot(..) => r_keys,
        };
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
        self.pinned_v = self.pinned_v.max(r_seq.max(r_keys).max(callee_v) + 1);
        let key = match by {
            SortKey::Slot(slot, key) => {
                let slot_loc = self.slot_loc(slot);
                self.emit_seq_load(slot_loc, seq_kind, None, Some(r_seq), r_idx, span);
                Some(key)
            }
            SortKey::Fn(func) => {
                let Ty::Fn(params, _) = &func.ty else {
                    unreachable!()
                };
                self.emit_seq_call(callee_v, &params[0], key_kind, seq_kind, r_seq, r_idx, span);
                None
            }
        };
        if let Some(kind) = key_kind {
            let val_i = match key {
                Some(key) => self.compile_expr_i(key),
                None => self.next_i - 1,
            };
            self.emit(
                Op::ListPushLI {
                    list_v: r_keys,
                    val_i,
                    kind,
                },
                span,
            );
        } else {
            let val_v = match key {
                Some(key) => self.compile_expr_v(key),
                None => self.next_v - 1,
            };
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

    /// Calls the fn value `callee_v` on element `r_idx` of `r_seq`, its
    /// one parameter of type `param`; the result is in the last register
    /// allocated (an integer one for a `key_kind` result).
    #[allow(clippy::too_many_arguments)]
    fn emit_seq_call(
        &mut self,
        callee_v: u16,
        param: &Ty,
        key_kind: Option<ScalarKind>,
        seq_kind: SeqKind,
        r_seq: u16,
        r_idx: u16,
        span: Span,
    ) {
        let arg_base_i = self.next_i;
        let arg_base_v = self.next_v;
        let arg = if let Some(kind) = ScalarKind::of_ty(param, self.program) {
            SlotLoc::I(self.alloc_i(), kind)
        } else if let Some(kind) = ScalarKind::of_opt_ty(param, self.program) {
            let val = self.alloc_i();
            let some = self.alloc_i();
            SlotLoc::OptI { val, some, kind }
        } else {
            SlotLoc::V(self.alloc_v())
        };
        self.emit_seq_load(arg, seq_kind, None, Some(r_seq), r_idx, span);
        let (ret_mode, dst_i, dst_v) = match key_kind {
            Some(_) => (CallRet::I, self.alloc_i(), 0),
            None => (CallRet::V, 0, self.alloc_v()),
        };
        self.emit(
            Op::CallClosure {
                callee_v,
                arg_base_i,
                arg_base_v,
                dst_i,
                dst_v,
                dst_some: 0,
                ret_mode,
            },
            span,
        );
    }
}
