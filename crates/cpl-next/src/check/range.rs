//! Ranges and domains as values (R01.8.4, R04.2).
use super::*;

impl Cx<'_> {
    /// A name that is a domain: one declared in a body, or a type parameter used as one.
    pub(super) fn domain_named(&self, n: &str) -> Option<Ty> {
        if let Some(&Ty::Dom(i)) = self.doms.get(n) {
            if self.decl.is_none() && self.domains[i].name.is_empty() {
                self.early.set(Some(i));
            }
            return Some(Ty::Dom(i));
        }
        self.generics.iter().position(|g| g == n).map(Ty::Param)
    }
    /// `lo..hi`, `lo..=hi` and `lo..hi by k` as a value: core's `Range` (R01.8.4).
    pub(super) fn range_value(
        &mut self,
        lo: &ast::Expr,
        hi: &ast::Expr,
        inclusive: bool,
        step: Option<&ast::Expr>,
        span: Span,
    ) -> TExpr {
        let mut end = |e: &ast::Expr| {
            let v = self.expr(e, Some(&Ty::Int));
            if !matches!(self.shallow(&v.ty), Ty::Int | Ty::Range(..) | Ty::Error) {
                let t = self.show(&v.ty);
                self.err(e.span, format!("slice: a range value with `{t}` ends"));
            }
            self.coerce(v, &Ty::Int)
        };
        let lo = end(lo);
        let mut hi = end(hi);
        if inclusive {
            let one = TExpr {
                kind: TK::Int(1),
                ty: Ty::Int,
                span,
            };
            hi = TExpr {
                kind: TK::Bin(BinOp::Add, Box::new(hi), Box::new(one)),
                ty: Ty::Int,
                span,
            };
        }
        let step = match step {
            Some(k) => end(k),
            None => TExpr {
                kind: TK::Int(1),
                ty: Ty::Int,
                span,
            },
        };
        self.range_of(vec![lo, hi, step], Ty::Int, span)
    }
    /// Core's `range_of(lo, hi, step)` over positions of `d`.
    fn range_of(&mut self, args: Vec<TExpr>, d: Ty, span: Span) -> TExpr {
        let r = self.lib_call("range_of", args, span);
        let want = match self.shallow(&r.ty) {
            Ty::Struct(id, _, sets) => Ty::Struct(id, vec![d], sets),
            t => t,
        };
        self.coerce(r, &want)
    }
    /// A domain named as a value: its whole range (R04.2.21).
    pub(super) fn domain_value(&mut self, d: Ty, span: Span) -> TExpr {
        let int = |kind| TExpr {
            kind,
            ty: Ty::Int,
            span,
        };
        let args = vec![int(TK::Int(0)), int(TK::DomLen(d.clone())), int(TK::Int(1))];
        self.range_of(args, d.clone(), span)
    }
}
