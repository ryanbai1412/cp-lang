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

    /// A call of a field that holds a function (`m.op(a, b)`, `self.lt(a, b)`)
    /// or a bare operator (`(count: +).count(a, b)`); `Err` gives `value` back.
    pub(super) fn call_field(
        &mut self,
        value: Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        span: Span,
        hint: Option<&Ty>,
    ) -> Result<Expr, Expr> {
        let Some((index, fty)) = self.field_of(&value.ty, &name.text) else {
            return Err(value);
        };
        if matches!(fty, Ty::Fn(..)) {
            let fv = self.mk(ExprKind::Field(Box::new(value), index), fty, span);
            return Ok(self.call_fn_val(fv, args, span));
        }
        match self.op_of_type(&fty) {
            Some(op) => self
                .bare_op_call(op, name.span, args, span, hint)
                .ok_or(value),
            None => Err(value),
        }
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

    /// The operator a record field holds (`ops.count` of `(count: +, n: 2)`),
    /// so `ops.count(a, b)` calls it. Only probed once an operator value exists.
    pub(super) fn field_op(&mut self, base: &ast::Expr, field: &str) -> Option<BareOp> {
        if self.op_types.is_empty() {
            return None;
        }
        let Some(Ty::Record(fields)) = self.probe_ty_silent(base) else {
            return None;
        };
        let (_, ty) = fields.iter().find(|(name, _)| name == field)?;
        self.op_of_type(ty)
    }
}
