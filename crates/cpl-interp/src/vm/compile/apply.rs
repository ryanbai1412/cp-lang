//! Expressions the VM evaluates by applying `Interp::apply` to operands it
//! computed in registers, rather than handing the whole expression to the
//! tree-walker.

use super::Compiler;
use crate::apply::{applies, operands};
use crate::vm::helpers::quiet;
use crate::vm::ir::*;
use cpl_ir::{Dim, Expr, ExprKind, Mutation, Ty};

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

impl<'p, 'f> Compiler<'p, 'f> {
    /// `expr` into a fresh value register by `Op::ApplyV`, when it applies
    /// to its operands.
    pub(super) fn try_compile_apply_v(&mut self, expr: &'p Expr) -> Option<u16> {
        if matches!(expr.kind, ExprKind::Mutate(..)) {
            return self.try_compile_mutate_place(expr);
        }
        if !applies(expr) {
            return None;
        }
        // A read evaluates its bounds once, before reading, so they must
        // not depend on when they run.
        if matches!(expr.kind, ExprKind::Read(_)) && !operands(expr).into_iter().all(quiet) {
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
        self.applies_sync |= sync;
        let idx = self.applies.len() as u32;
        self.applies.push(ApplySpec {
            expr,
            regs: regs.clone(),
        });
        let dst = self.alloc_v();
        self.emit(Op::ApplyV { idx, dst, sync }, expr.span);
        for r in regs {
            if r >= self.num_local_v {
                self.emit(Op::ClearV { reg: r }, expr.span);
            }
        }
        Some(dst)
    }

    /// The mutation `expr` into a fresh value register by
    /// `Op::MutatePlace`, when the VM can walk to its place and the change
    /// runs no user code.
    pub(super) fn try_compile_mutate_place(&mut self, expr: &'p Expr) -> Option<u16> {
        let ExprKind::Mutate(op, place, args) = &expr.kind else {
            return None;
        };
        if *op == Mutation::Shuffle || !plain(&place.ty) || !args.iter().all(|a| plain(&a.ty)) {
            return None;
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
