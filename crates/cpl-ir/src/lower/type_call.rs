//! `T.m(..)` where `T` is a built-in type, named or filling a type parameter.

use super::*;

impl Lowerer<'_> {
    /// `T.gen(r)` is the fn `T`'s `impl` declares; `T.from_int(n)` for a
    /// built-in number is `T(n)`, as every number takes `Int` through
    /// `from_int` (W1).
    pub(super) fn builtin_type_call(
        &mut self,
        base: &ast::Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        span: Span,
    ) -> Option<Expr> {
        let ast::ExprKind::Name(tname) = &base.kind else {
            return None;
        };
        if self.lookup(tname).is_some() {
            return None;
        }
        let ty = match (self.gen_env.get(tname), tname.as_str()) {
            (Some(GenArg::Ty(ty)), _) => ty.clone(),
            (Some(_), _) => return None,
            (None, "Int" | "I64") => Ty::Int,
            (None, "I128") => Ty::I128,
            (None, "U64") => Ty::U64,
            (None, "F64") => Ty::Float,
            _ => return None,
        };
        let found = self
            .prim_methods
            .iter()
            .find(|(t, m, _)| *t == ty && *m == name.text)
            .map(|p| p.2);
        if let Some(fn_id) = found {
            return Some(self.user_call(fn_id, None, args, span));
        }
        let to = match (&ty, name.text.as_str()) {
            (Ty::Int, "from_int") => "Int",
            (Ty::I128, "from_int") => "I128",
            (Ty::U64, "from_int") => "U64",
            (Ty::Float, "from_int") => "F64",
            _ => return None,
        };
        let value = match self.one_arg("from_int", args, &Ty::Int, span) {
            Some(value) => value,
            None => return Some(self.err_expr(span)),
        };
        Some(self.convert(to, value, span))
    }
}
