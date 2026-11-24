//! Expressions the VM evaluates by applying `Interp::apply` to operands it
//! computed in registers, rather than handing the whole expression to the
//! tree-walker.

use super::Compiler;
use crate::apply::{applies, operands};
use crate::vm::helpers::{place_indices, quiet};
use crate::vm::ir::*;
use cpl_ir::{Dim, Expr, ExprKind, Mutation, Place, PlaceKind, Program, Ty};

/// Whether comparing values of `ty` never runs user code (a struct's `eq`
/// or `lt`), so the frame can stay in registers.
fn plain(ty: &Ty) -> bool {
    match ty {
        Ty::Unit
        | Ty::Int
        | Ty::I128
        | Ty::U64
        | Ty::Float
        | Ty::Bool
        | Ty::Char
        | Ty::Str
        | Ty::StrView
        | Ty::Pos(_)
        | Ty::Bound(_)
        | Ty::DomainVal => true,
        Ty::Opt(t) | Ty::List(t) | Ty::Range(t) | Ty::Set(t) | Ty::HashSet(t) => plain(t),
        Ty::ListView(t, Dim::Int) => plain(t),
        Ty::Array { elem, .. } => plain(elem),
        Ty::Tuple(ts) => ts.iter().all(plain),
        Ty::Record(fs) => fs.iter().all(|(_, t)| plain(t)),
        Ty::Map(k, v) | Ty::HashMap(k, v) => plain(k) && plain(v),
        _ => false,
    }
}

/// Whether a value of `ty` is owned data with no fn in it: comparing it in
/// a set or map (`crate::mutation::mutate_cell`) runs no user code.
fn owned(ty: &Ty, program: &Program) -> bool {
    match ty {
        Ty::Struct(_) | Ty::Enum(_) => !ty.has_fn(program),
        Ty::Opt(t) | Ty::List(t) | Ty::Range(t) | Ty::Set(t) | Ty::HashSet(t) => owned(t, program),
        Ty::ListView(t, Dim::Int) => owned(t, program),
        Ty::Array { elem, .. } => owned(elem, program),
        Ty::Tuple(ts) => ts.iter().all(|t| owned(t, program)),
        Ty::Record(fs) => fs.iter().all(|(_, t)| owned(t, program)),
        Ty::Map(k, v) | Ty::HashMap(k, v) => owned(k, program) && owned(v, program),
        _ => plain(ty),
    }
}

/// Whether `place` is reached through a mutable view.
pub(super) fn through_view(place: &Place) -> bool {
    matches!(place.ty, Ty::MutView(..))
        || match &place.kind {
            PlaceKind::Field(base, _) | PlaceKind::Some(base) | PlaceKind::Index(base, _) => {
                through_view(base)
            }
            PlaceKind::Local(_) | PlaceKind::Global(_) => false,
        }
}

impl<'p, 'f> Compiler<'p, 'f> {
    /// `view` by `Op::ApplyV`, its `operands` evaluated before the place's
    /// indices when `operands_first`, as the tree-walker does; `expr` gives
    /// its span. The place's root local goes on the stack around each call
    /// too, where a view of it writes.
    pub(super) fn compile_view(
        &mut self,
        view: ViewOp<'p>,
        place: &'p Place,
        operands: &[&'p Expr],
        operands_first: bool,
        expr: &'p Expr,
    ) -> u16 {
        if let (false, slot) = cpl_ir::walk::root(place) {
            let slot = slot as u16;
            if let Err(at) = self.ref_slots.binary_search(&slot) {
                self.ref_slots.insert(at, slot);
            }
            if matches!(view, ViewOp::Slice(_) | ViewOp::Line(..))
                && !self.view_roots.contains(&slot)
            {
                self.view_roots.push(slot);
            }
        }
        let mut indices = Vec::new();
        crate::vm::helpers::place_indices(place, &mut indices);
        let (first, then) = match operands_first {
            true => (operands, &indices[..]),
            false => (&indices[..], operands),
        };
        let mut regs: Vec<u16> = first.iter().map(|x| self.compile_expr_v(x)).collect();
        regs.extend(then.iter().map(|x| self.compile_expr_v(x)));
        if !operands_first {
            regs.rotate_left(indices.len());
        }
        self.applies_sync = true;
        let idx = self.applies.len() as u32;
        self.applies.push(ApplySpec {
            expr,
            regs: regs.clone(),
            view: Some((view, operands.len())),
        });
        let dst = self.alloc_v();
        self.emit(
            Op::ApplyV {
                idx,
                dst,
                sync: true,
            },
            expr.span,
        );
        for r in regs {
            if r >= self.num_local_v {
                self.emit(Op::ClearV { reg: r }, expr.span);
            }
        }
        dst
    }

    /// `expr` into a fresh value register by `Op::ApplyV`, when it applies
    /// to its operands.
    pub(super) fn try_compile_apply_v(&mut self, expr: &'p Expr) -> Option<u16> {
        if let ExprKind::Mutate(op, place, args) = &expr.kind {
            if through_view(place) {
                let args_ref: Vec<&'p Expr> = args.iter().collect();
                let view = ViewOp::Mutate(*op, place, args);
                return Some(self.compile_view(view, place, &args_ref, false, expr));
            }
            return self.try_compile_mutate_place(expr);
        }
        if let ExprKind::Sort(place, by, desc) = &expr.kind {
            if through_view(place) {
                let desc: Vec<&'p Expr> = desc.iter().map(|d| &**d).collect();
                let view = ViewOp::Sort(place, by);
                return Some(self.compile_view(view, place, &desc, true, expr));
            }
        }
        if !applies(expr) {
            return None;
        }
        // A read evaluates its bounds once, before reading, so they must
        // not depend on when they run.
        if matches!(expr.kind, ExprKind::Read(_) | ExprKind::ReadFrom(..))
            && !operands(expr).into_iter().all(quiet)
        {
            return None;
        }
        let regs: Vec<u16> = operands(expr)
            .into_iter()
            .map(|x| self.compile_expr_v(x))
            .collect();
        let sync = match &expr.kind {
            ExprKind::Cmp(_, lhs, _) => !plain(&lhs.ty),
            ExprKind::Index(base, _) => !plain(&base.ty),
            ExprKind::Builtin(..) | ExprKind::Binary(..) => {
                !operands(expr).iter().all(|a| plain(&a.ty))
            }
            _ => false,
        };
        Some(self.emit_apply_v(expr, regs, sync))
    }

    /// `Op::ApplyV` of `expr` to `regs` into a fresh value register.
    fn emit_apply_v(&mut self, expr: &'p Expr, regs: Vec<u16>, sync: bool) -> u16 {
        self.applies_sync |= sync;
        let idx = self.applies.len() as u32;
        self.applies.push(ApplySpec {
            expr,
            regs: regs.clone(),
            view: None,
        });
        let dst = self.alloc_v();
        self.emit(Op::ApplyV { idx, dst, sync }, expr.span);
        for r in regs {
            if r >= self.num_local_v {
                self.emit(Op::ClearV { reg: r }, expr.span);
            }
        }
        dst
    }

    /// The pop `expr` of `place` (`pop`, `pop_first`, `pop_last`, which
    /// compare nothing) into a fresh value register: by `Op::MutatePlace`
    /// when the VM can walk to the place, else by `Interp::mutated` on the
    /// frame, which resolves any place.
    pub(super) fn compile_pop_v(&mut self, expr: &'p Expr, place: &'p Place) -> u16 {
        if let Some(dst) = self.try_compile_mutate_place(expr) {
            return dst;
        }
        let mut indices = Vec::new();
        place_indices(place, &mut indices);
        let regs = indices
            .into_iter()
            .map(|x| self.compile_expr_v(x))
            .collect();
        self.emit_apply_v(expr, regs, true)
    }

    /// The mutation `expr` into a fresh value register by
    /// `Op::MutatePlace`, when the VM can walk to its place.
    pub(super) fn try_compile_mutate_place(&mut self, expr: &'p Expr) -> Option<u16> {
        let ExprKind::Mutate(op, place, args) = &expr.kind else {
            return None;
        };
        if let Ty::Set(k) | Ty::HashSet(k) | Ty::Map(k, _) | Ty::HashMap(k, _) = &place.ty {
            let ordered = matches!(place.ty, Ty::Set(_) | Ty::Map(..));
            if matches!(op, Mutation::Insert | Mutation::Remove)
                && (!owned(k, self.program)
                    || ordered && crate::value::keys::user_ordered(k, self.program))
            {
                return None;
            }
        }
        let saved = (self.ops.len(), self.spans.len(), self.places.len());
        let Some(path) = self.compile_place_path_to(place, true) else {
            self.ops.truncate(saved.0);
            self.spans.truncate(saved.1);
            self.places.truncate(saved.2);
            return None;
        };
        let regs: Vec<u16> = args.iter().map(|a| self.compile_expr_v(a)).collect();
        let idx = self.applies.len() as u32;
        self.applies.push(ApplySpec {
            expr,
            regs: regs.clone(),
            view: None,
        });
        let dst = self.alloc_v();
        self.emit(Op::MutatePlace { idx, path, dst }, expr.span);
        for r in regs {
            if r >= self.num_local_v {
                self.emit(Op::ClearV { reg: r }, expr.span);
            }
        }
        Some(dst)
    }

    /// `expr` into a fresh scalar register by `Op::ApplyV`.
    pub(super) fn try_compile_apply_i(&mut self, expr: &'p Expr) -> Option<u16> {
        let src_v = self.try_compile_apply_v(expr)?;
        let dst_i = self.alloc_i();
        self.emit(Op::UnboxI { dst_i, src_v }, expr.span);
        self.emit(Op::ClearV { reg: src_v }, expr.span);
        Some(dst_i)
    }
}
