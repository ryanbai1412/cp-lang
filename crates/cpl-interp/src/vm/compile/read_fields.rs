//! Reads the VM can't apply in one go: a type's own `read()`, a bound
//! that runs code, or a tuple, record or struct with such a part, read
//! part by part in the order the tree-walker reads them.

use super::Compiler;
use crate::apply::{applies, operands, spec_exprs};
use crate::vm::helpers::quiet;
use crate::vm::ir::*;
use cpl_ir::{Expr, Spec};

/// Whether every expression in `spec` is evaluated once and quietly, so
/// it can be evaluated ahead of the reads around it.
fn hoists(spec: &Spec) -> bool {
    spec_exprs(spec).is_some_and(|xs| xs.into_iter().all(quiet))
}

/// Whether [`Compiler::compile_read_v`] reads `spec`: each part a call, a
/// scalar, or one that [`hoists`].
fn by_parts(spec: &Spec) -> bool {
    match spec {
        Spec::Call(_) | Spec::Plain(_) | Spec::Line | Spec::Int { .. } | Spec::Str { .. } => true,
        Spec::Fields(_, fields) => fields.len() <= 16 && fields.iter().all(by_parts),
        _ => hoists(spec),
    }
}

impl<'p, 'f> Compiler<'p, 'f> {
    /// Whether the read `expr` of `spec` needs [`Self::compile_read_v`]:
    /// `Op::ApplyV` of the whole read can't take it.
    pub(super) fn reads_by_parts(expr: &Expr, spec: &Spec) -> bool {
        !(applies(expr) && operands(expr).into_iter().all(quiet)) && by_parts(spec)
    }

    /// The read `expr` of `spec` into a value register, part by part.
    pub(super) fn compile_read_v(&mut self, spec: &'p Spec, expr: &'p Expr) -> u16 {
        match spec {
            Spec::Call(call) => self.compile_expr_v(call),
            Spec::Fields(_, fields) if !hoists(spec) => {
                let regs = fields.iter().map(|f| self.compile_read_v(f, expr)).collect();
                self.emit_make_tuple(regs, expr.span)
            }
            // Its bounds, then its tokens, as the tree-walker reads it.
            _ => {
                let exprs = spec_exprs(spec).unwrap_or_default();
                let regs: Vec<u16> = exprs.into_iter().map(|x| self.compile_expr_v(x)).collect();
                let n = regs.len();
                self.emit_apply(expr, regs, Some((ViewOp::Read(spec), n)), false)
            }
        }
    }

    /// [`Self::compile_read_v`] into an Int register.
    pub(super) fn compile_read_i(&mut self, spec: &'p Spec, expr: &'p Expr) -> u16 {
        if let Spec::Call(call) = spec {
            return self.compile_expr_i(call);
        }
        let src_v = self.compile_read_v(spec, expr);
        let dst_i = self.alloc_i();
        self.emit(Op::UnboxI { dst_i, src_v }, expr.span);
        self.emit(Op::ClearV { reg: src_v }, expr.span);
        dst_i
    }
}
