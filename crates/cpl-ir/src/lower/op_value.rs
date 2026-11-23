//! A bare operator inside a tuple or record (`each(+, max)`, `(n: +, hi: max)`)
//! is a value of its own empty type, one per operator, so the operator stays
//! known through generics and `fields(..)`: `same(m, +)` reads it off the type,
//! as it does for a marker struct (R06.1.5).

use super::mono::BareOp;
use super::*;

impl<'a> Lowerer<'a> {
    /// The empty struct that stands for `op` as a value.
    fn op_type(&mut self, op: BareOp) -> StructId {
        if let Some(&(_, id)) = self.op_types.iter().find(|(o, _)| *o == op) {
            return id;
        }
        let id = self.push_synth_struct(format!("Op_{op:?}"), Vec::new());
        self.op_types.push((op, id));
        id
    }

    /// The operator a value of type `ty` stands for.
    pub(super) fn op_of_type(&self, ty: &Ty) -> Option<BareOp> {
        let Ty::Struct(id) = ty else { return None };
        self.op_types
            .iter()
            .find(|(_, s)| s == id)
            .map(|(op, _)| *op)
    }

    /// A component of a tuple or record literal: a bare operator with no fn
    /// type asked for is its operator value; anything else lowers as usual.
    pub(super) fn component(&mut self, item: &ast::Expr, hint: Option<&Ty>) -> Expr {
        let bare = matches!(
            &item.kind,
            ast::ExprKind::Lambda { .. } | ast::ExprKind::Name(_)
        );
        let wants_op = match hint {
            None | Some(Ty::Unit) => true,
            Some(h) => self.op_of_type(h).is_some(),
        };
        if bare && wants_op {
            if let Some(op) = self.detect_bare_op(item) {
                let id = self.op_type(op);
                return self.mk(ExprKind::Tuple(Vec::new()), Ty::Struct(id), item.span);
            }
        }
        match hint {
            Some(h) => self.expect(item, h),
            None => self.expr(item, None),
        }
    }

    /// The operator `e` names: a bare operator, or a value of an operator type.
    pub(super) fn op_identity(&mut self, e: &ast::Expr) -> Option<BareOp> {
        if let Some(op) = self.detect_bare_op(e) {
            return Some(op);
        }
        let ty = self.probe_ty_silent(e)?;
        self.op_of_type(&ty)
    }
}
