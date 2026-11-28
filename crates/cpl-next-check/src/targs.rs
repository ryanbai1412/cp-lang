//! Type arguments written on an expression: `id::<Int>`, `s.parse<Int>()` (R08.2.5).
use super::call::CallArg;
use super::expr::block;
use super::member::{bare_name, written_types, Called};
use super::*;
use cpl_next_ir::unbuilt::Unbuilt;

impl Cx<'_> {
    /// `f<A>` named as a value: the closure that calls `f` with `A`, as `f` alone is.
    pub(super) fn generic_value(&mut self, g: &next::GenericExpr, span: Span) -> TExpr {
        let targs = &written_types(g);
        let err = TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
        };
        let Some((base, n)) = g
            .base()
            .and_then(|b| Some((bare_name(&b)?, b)))
            .map(|(n, b)| (b, n))
        else {
            self.err(
                span,
                Unbuilt::Expression {
                    kind: "Generic".into(),
                },
            );
            return err;
        };
        let n = n.as_str();
        let fd = (self.lookup(n).is_none())
            .then(|| self.fn_named(self.b.module, n))
            .flatten();
        let Some(d) = fd.filter(|d| !self.items.f(*d).generics.is_empty()) else {
            if fd.is_none()
                && self.lookup(n).is_none()
                && self.struct_named(self.b.module, n).is_some()
            {
                // `Pair<V>` named as a value, as `for p in Pair<V>`: the domain of its
                // positions, as a domain's name alone is (R04.2.27).
                let mut pre = Vec::new();
                if let Some(t) = self.struct_at(&base, g.args(), span, &mut pre) {
                    if self.is_domain(&t) {
                        let v = self.domain_value(t, span);
                        return block(pre, v);
                    }
                }
                self.err(span, Unbuilt::TypeArgsValue);
                return err;
            }
            // Anything else is what the name alone is: unknown, the error, or a value that
            // takes no type arguments.
            let alone = self.expr(&base, None);
            if self.is_error(&alone.ty) {
                return err;
            }
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    format!("`{n}` takes no type arguments"),
                ),
            );
            return err;
        };
        let mut pre = Vec::new();
        let given: Vec<Ty> = (targs.iter()).map(|t| self.body_ty(t, &mut pre)).collect();
        let given = self.written_targs(d, Vec::new(), &given, span);
        let f = self.fnref(d, given, Vec::new());
        let v = self.fn_item_value(f, span);
        block(pre, v)
    }

    /// `recv.m<A>(args)`: the written arguments are `m`'s own, after its owner's.
    pub(super) fn generic_method(
        &mut self,
        g: &next::GenericExpr,
        args: &[CallArg],
        span: Span,
        want: Option<&Ty>,
    ) -> TExpr {
        let targs = &written_types(g);
        let field = match g.base() {
            Some(next::Expr::Field(f)) => f.base().zip(f.name()),
            _ => None,
        };
        let Some((base, name)) = field else {
            self.err(
                span,
                Unbuilt::Expression {
                    kind: "Generic".into(),
                },
            );
            return TExpr {
                kind: TK::Error,
                ty: Ty::Error,
                span,
            };
        };
        let mut pre = Vec::new();
        let given: Vec<Ty> = (targs.iter()).map(|t| self.body_ty(t, &mut pre)).collect();
        let name = Called::written(self.syms, name.text(), name.span());
        let v = self.member(&base, name, Some(args), span, want, &given);
        block(pre, v)
    }

    /// The type arguments of `m` with the written ones after those already fixed (`ts`).
    /// Type arguments written on a member that has no type parameters of its own: a field, a
    /// variant, a domain's member.
    pub(super) fn no_targs(&mut self, name: Called<'_>, written: &[Ty], span: Span) {
        if !written.is_empty() {
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    format!("`{}` takes no type arguments", name.text),
                ),
            );
        }
    }
    pub(super) fn written_targs(
        &mut self,
        m: FnId,
        mut ts: Vec<Ty>,
        written: &[Ty],
        span: Span,
    ) -> Vec<Ty> {
        let fd = self.items.f(m);
        let takes = fd.generics.len().saturating_sub(ts.len());
        if written.len() > takes {
            let name = fd.name.clone();
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::WRONG_ARG_COUNT,
                    format!(
                        "`{name}` takes {takes} type arguments, found {}",
                        written.len()
                    ),
                ),
            );
            return ts;
        }
        ts.extend(written.iter().cloned());
        ts
    }
}
