//! The 2-D shapes of a family (`Prefix(a)` on `a: [R, C]T`) and arity keys.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// The key of the struct `name` with `arity` type arguments, when it shares
    /// its name with one of another arity (R02.11.4), else `name`.
    pub(super) fn arity_name(&self, name: &str, arity: usize) -> String {
        let key = cpl_check::arity_key(name, arity);
        match self.get_sym(&key) {
            Some(_) => key,
            None => name.to_string(),
        }
    }

    /// Whether the struct `shape` is built by a `new(a: List<T>)`.
    pub(super) fn takes_list(&self, shape: &str) -> bool {
        let Some(Sym::GenStruct(gid)) = self.get_sym(shape) else {
            return false;
        };
        self.gen_structs[gid].decl.methods.iter().any(|m| {
            matches!(&m.kind, ast::StmtKind::Fn(f) if f.name.text == "new"
                && matches!(f.params.first().and_then(|p| p.ty.as_ref()).map(|t| &t.kind),
                    Some(ast::TypeKind::Named { name, .. }) if name.text == "List"))
        })
    }

    /// A list, or an array indexed by plain `Int`s from 0.
    pub(super) fn counted_ty(ty: &Ty) -> bool {
        match ty {
            Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(..) => true,
            Ty::Array { dims, .. } => matches!(dims[..], [Dim::Int]),
            _ => false,
        }
    }

    /// `Prefix(a)` on a `List<T>` or a `[0..n]T`: the shape `new(a: List<T>)`.
    pub(super) fn list_shape_ctor(&mut self, shape: &str, args: &[ast::Arg], span: Span) -> Expr {
        let src = self.expr(&args[0].value, None);
        let elem = match &src.ty {
            Ty::Array { elem, .. }
            | Ty::List(elem)
            | Ty::ListView(elem, _)
            | Ty::Keyed(elem, _)
            | Ty::MutView(elem, _) => (**elem).clone(),
            _ => return self.err_expr(span),
        };
        let Some(Sym::GenStruct(gid)) = self.get_sym(shape) else {
            return self.err_expr(span);
        };
        let src = self.coerce(src, &Ty::list(elem.clone()), span);
        let sid = self.instantiate_struct(gid, vec![GenArg::Ty(elem)]);
        let Some(fn_id) = self.own_method(sid, "new") else {
            return self.err_expr(span);
        };
        let ret = self.ret_of(fn_id, span);
        self.mk(ExprKind::Call(fn_id, vec![Arg::Value(src)]), ret, span)
    }

    /// `Prefix(a)` on `a: [R, C]T`: the family's 2-D shape `new(a)`.
    pub(super) fn grid_shape_ctor(&mut self, shape: &str, args: &[ast::Arg], span: Span) -> Expr {
        let src = self.expr(&args[0].value, None);
        let Ty::Array { dims, elem } = &src.ty else {
            return self.err_expr(span);
        };
        let (Dim::Pos(r), Dim::Pos(c)) = (&dims[0], &dims[1]) else {
            self.report_str(span, msgs::ir::TYPE_MISMATCH_TAKES_AN_ARRAY_OVER, shape);
            return self.err_expr(span);
        };
        let gargs = vec![
            GenArg::Domain(*r),
            GenArg::Domain(*c),
            GenArg::Ty((**elem).clone()),
        ];
        let Some(Sym::GenStruct(gid)) = self.get_sym(shape) else {
            return self.err_expr(span);
        };
        let sid = self.instantiate_struct(gid, gargs);
        let Some(fn_id) = self.own_method(sid, "new") else {
            let written = shape.split('\'').next().unwrap_or(shape);
            self.report(
                span,
                msgs::check::NO_METHOD_ON,
                &[&"struct", &written, &"new"],
            );
            return self.err_expr(span);
        };
        let ret = self.ret_of(fn_id, span);
        self.mk(ExprKind::Call(fn_id, vec![Arg::Value(src)]), ret, span)
    }
}
