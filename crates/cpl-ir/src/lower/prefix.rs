//! `prefix(a)` and the `Prefix` shapes of `std.prefix`.

use super::*;

pub(crate) fn prefix_shape_struct(name: &str, arity: usize) -> Option<&'static str> {
    match (name, arity) {
        ("Prefix", 1) => Some("_PrefixList"),
        ("Prefix", 3) => Some("_Prefix2"),
        ("FillPrefix", 1) => Some("_FillPrefixList"),
        ("FillPrefix", 3) => Some("_FillPrefix2"),
        ("Diff", 3) => Some("_Diff2"),
        _ => None,
    }
}

impl<'a> Lowerer<'a> {
    /// `prefix(a)` dispatches on the shape of `a`: `List<T>` / `[0..n]T`,
    /// `[D]T`, or `[R, C]T`, constructing the matching `Prefix` shape from
    /// `std.prefix`.
    pub(super) fn lower_prefix(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        let Some(exprs) = self.positional_any("prefix", args, 1, span) else {
            return self.err_expr(span);
        };
        let src = self.expr(exprs[0], None);
        if src.ty == Ty::Error {
            return self.err_expr(span);
        }
        let elem_ty = match &src.ty {
            Ty::Array { elem, .. }
            | Ty::List(elem)
            | Ty::ListView(elem, _)
            | Ty::Keyed(elem, _)
            | Ty::MutView(elem) => (**elem).clone(),
            _ => Ty::Error,
        };
        let adds = match &elem_ty {
            Ty::Int | Ty::Float | Ty::I128 | Ty::U64 => true,
            Ty::Struct(sid) => ["add", "sub"]
                .iter()
                .all(|op| self.struct_op_fn(*sid, op).is_some()),
            _ => false,
        };
        let doms: Option<Vec<DomainId>> = match &src.ty {
            Ty::Array { dims, .. } if adds && matches!(dims.len(), 1 | 2) => dims
                .iter()
                .map(|d| match d {
                    Dim::Pos(d) => Some(*d),
                    _ => None,
                })
                .collect(),
            _ => None,
        };
        // A list, or an array indexed by plain `Int`s from 0: `p.sum(l..r)`
        // takes `Int`s.
        let counted = match &src.ty {
            Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(_) => true,
            Ty::Array { dims, .. } => matches!(dims[..], [Dim::Int]),
            _ => false,
        };
        if counted && adds {
            let Some(Sym::GenStruct(gid)) = self.get_sym("_PrefixList") else {
                self.unsupported(span, "`prefix` without `std.prefix.Prefix` in scope");
                return self.err_expr(span);
            };
            let list = Ty::List(Box::new(elem_ty.clone()));
            let src = self.coerce(src, &list, span);
            let sid = self.instantiate_struct(gid, vec![GenArg::Ty(elem_ty)]);
            let Some(&fn_id) = self.structs[sid].methods.get("new") else {
                self.unsupported(span, "`Prefix` has no `new` constructor");
                return self.err_expr(span);
            };
            let ret = self.ret_of(fn_id, span);
            return self.mk(ExprKind::Call(fn_id, vec![Arg::Value(src)]), ret, span);
        }
        if adds
            && matches!(&src.ty, Ty::Array { dims, .. } if matches!(dims[..], [Dim::IntFrom(_)]))
        {
            let Some(Sym::GenStruct(gid)) = self.get_sym("Prefix") else {
                self.unsupported(span, "`prefix` without `std.prefix.Prefix` in scope");
                return self.err_expr(span);
            };
            return self.call_gen_struct(gid, Vec::new(), args, None, span);
        }
        let Some(doms) = doms else {
            if src.ty.mentions_error() {
                return self.err_expr(span);
            }
            let message = format!(
                "`prefix` takes a `[D]T`, `[R, C]T`, or `List<T>` of numbers, found `{}`",
                self.show(&src.ty)
            );
            self.error(codes::TYPE_MISMATCH, span, message);
            return self.err_expr(span);
        };
        let struct_name = if doms.len() == 1 {
            "Prefix"
        } else {
            "_Prefix2"
        };
        let Some(Sym::GenStruct(gid)) = self.get_sym(struct_name) else {
            self.unsupported(span, "`prefix` without `std.prefix.Prefix` in scope");
            return self.err_expr(span);
        };
        let mut gargs: Vec<GenArg> = doms.into_iter().map(GenArg::Domain).collect();
        gargs.push(GenArg::Ty(elem_ty));
        let sid = self.instantiate_struct(gid, gargs);
        let Some(&fn_id) = self.structs[sid].methods.get("new") else {
            self.unsupported(span, "`Prefix` has no `new` constructor");
            return self.err_expr(span);
        };
        let ret = self.ret_of(fn_id, span);
        self.mk(ExprKind::Call(fn_id, vec![Arg::Value(src)]), ret, span)
    }
}
