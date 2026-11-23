//! Expressions the VM evaluates by applying `Interp::apply` to operands it
//! computed in registers, rather than handing the whole expression to the
//! tree-walker.

use super::Compiler;
use crate::apply::{applies, operands};
use crate::vm::helpers::quiet;
use crate::vm::ir::*;
use cpl_ir::{Dim, Expr, ExprKind, Ty};

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

    /// `expr` into a fresh scalar register by `Op::ApplyV`.
    pub(super) fn try_compile_apply_i(&mut self, expr: &'p Expr) -> Option<u16> {
        let src_v = self.try_compile_apply_v(expr)?;
        let dst_i = self.alloc_i();
        self.emit(Op::UnboxI { dst_i, src_v }, expr.span);
        self.emit(Op::ClearV { reg: src_v }, expr.span);
        Some(dst_i)
    }
}
