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
            (None, "Char") => Ty::Char,
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
        if ty == Ty::Char && name.text == "try_at" {
            let value = match self.one_arg(&name.text, args, &Ty::Int, span) {
                Some(value) => value,
                None => return Some(self.err_expr(span)),
            };
            return Some(self.char_try_at(value, span));
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

    /// `Char.try_at(i)`: the `i`th of the 128 chars `Char.count` counts, `None` outside them.
    fn char_try_at(&mut self, value: Expr, span: Span) -> Expr {
        let mut stmts = Vec::new();
        let i = self.hold(value, &mut stmts);
        let zero = self.int_lit(0, span);
        let k = self.int_lit(128, span);
        let ge0 = self.mk(
            ExprKind::Cmp(CmpOp::Ge, Box::new(i.clone()), Box::new(zero)),
            Ty::Bool,
            span,
        );
        let lt_k = self.mk(
            ExprKind::Cmp(CmpOp::Lt, Box::new(i.clone()), Box::new(k)),
            Ty::Bool,
            span,
        );
        let ok = self.mk(
            ExprKind::Binary(BinOp::And, Box::new(ge0), Box::new(lt_k)),
            Ty::Bool,
            span,
        );
        let ty = Ty::opt(Ty::Char);
        let ch = self.cast(Cast::IntToChar, i, Ty::Char);
        let some = self.mk(ExprKind::Some(Box::new(ch)), ty.clone(), span);
        let none = self.mk(ExprKind::None, ty.clone(), span);
        let pick = self.mk(
            ExprKind::If(Box::new(ok), Box::new(some), Box::new(none)),
            ty.clone(),
            span,
        );
        stmts.push(Stmt::Give(pick));
        self.mk(ExprKind::Do(Block { stmts }), ty, span)
    }
}
