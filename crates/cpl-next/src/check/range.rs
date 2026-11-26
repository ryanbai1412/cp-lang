//! Ranges and domains as values (R01.8.4, R04.2).
use super::expr::block;
use super::*;

impl Cx<'_> {
    /// A fn of a domain named as a type (`D.nth(k)`), or of an array type written `[e]`, as
    /// in `[V].filled(0)`.
    pub(super) fn domain_member(
        &mut self,
        base: &ast::Expr,
        name: &ast::Name,
        call: Option<&[ast::Arg]>,
        span: Span,
        want: Option<&Ty>,
    ) -> Option<TExpr> {
        if let ExprKind::Name(n) = &base.kind {
            if let (Some(dty), true) = (self.domain_named(n), self.lookup(n).is_none()) {
                if let Some(e) = self.domain_fn(dty, name, call, span) {
                    return Some(e);
                }
            }
        }
        // `[e].f(args)`: a fn of the array type `[e]_` with no receiver, as in `[V].filled(0)`.
        if let (ExprKind::List(xs), Some(args)) = (&base.kind, call) {
            if let [index] = &xs[..] {
                let elem = ast::Type {
                    kind: TypeKind::Tuple(Vec::new()),
                    span,
                };
                let t = ast::Type {
                    kind: TypeKind::Array {
                        index: vec![ast::Type::of_index(index)],
                        elem: Box::new(elem),
                    },
                    span,
                };
                let mut pre = Vec::new();
                if let Ty::Struct(d, ts, _) = self.body_ty(&t, span, &mut pre) {
                    let given: Vec<Ty> = (ts.into_iter())
                        .map(|a| if a == Ty::Unit { self.new_ty() } else { a })
                        .collect();
                    if let Some(m) = self.d.s(d).methods.get(&name.text).copied() {
                        let f = self.fnref(m, given, Vec::new());
                        let call = self.call_fn(f, None, args, span, want);
                        return Some(block(pre, call));
                    }
                }
            }
        }
        None
    }
    /// `D.count`, `D.nth(k)` and `D.at(x)`: the core's `Domain` declares them, and each is the
    /// domain operation its `@intrinsic` names (R04.2.8, R04.2.9).
    fn domain_fn(
        &mut self,
        dty: Ty,
        name: &ast::Name,
        call: Option<&[ast::Arg]>,
        span: Span,
    ) -> Option<TExpr> {
        let tr = self.d.domain_trait?;
        let &m = self.d.traits[tr.0].methods.get(&name.text)?;
        let op = self.d.f(m).intrinsic.clone()?;
        let mut int = |a: &ast::Arg| {
            let x = self.expr(&a.value, Some(&Ty::Int));
            self.coerce(x, &Ty::Int)
        };
        Some(match (op.as_str(), call) {
            ("dom_count", None) => TExpr {
                kind: TK::DomLen(dty),
                ty: Ty::Int,
                span,
            },
            ("dom_nth", Some([k])) => {
                let k = int(k);
                TExpr {
                    kind: TK::DomNth(dty.clone(), Box::new(k)),
                    ty: dty,
                    span,
                }
            }
            ("dom_at", Some([x])) => {
                let x = int(x);
                self.domain_at(dty, x, span)
            }
            _ => return None,
        })
    }
    /// `[D] k => e`: one cell per position of `D` in its walk order, which is the list
    /// `[e for k in D]` where a `[D]T` is wanted (R02.8.2).
    pub(super) fn table(
        &mut self,
        dom: &ast::Type,
        pat: &ast::Pattern,
        body: &ast::Expr,
        span: Span,
    ) -> TExpr {
        let mut pre = Vec::new();
        let hole = ast::Type {
            kind: TypeKind::Tuple(Vec::new()),
            span,
        };
        let shape = ast::Type {
            kind: TypeKind::Array {
                index: vec![dom.clone()],
                elem: Box::new(hole),
            },
            span,
        };
        let (array, dty, sets) = match self.body_ty(&shape, span, &mut pre) {
            Ty::Struct(a, ts, sets) => (a, ts[0].clone(), sets),
            _ => return self.refuse(span, "slice: a table over this index"),
        };
        let name = match &dty {
            Ty::Dom(i) if !self.domains[*i].name.is_empty() => self.domains[*i].name.clone(),
            _ => return self.refuse(span, "slice: a table over this index"),
        };
        let walk = ast::Expr {
            kind: ExprKind::Name(name),
            span: dom.span,
        };
        let comp = ast::Expr {
            kind: ExprKind::Comprehension {
                body: Box::new(body.clone()),
                clauses: vec![ast::Clause {
                    pat: pat.clone(),
                    iter: walk,
                    conds: Vec::new(),
                }],
            },
            span,
        };
        let list = self.expr(&comp, None);
        let elem = match self.shallow(&list.ty) {
            Ty::Struct(_, ts, _) if ts.len() == 1 => ts[0].clone(),
            _ => Ty::Error,
        };
        let cells = self.coerce(list, &Ty::Struct(array, vec![dty, elem], sets));
        block(pre, cells)
    }
    fn refuse(&mut self, span: Span, why: &str) -> TExpr {
        self.err(span, why);
        TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
        }
    }
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
    /// `D.at(x)`: `Some(D(x))` when `x` is a position of `D`, else `None`.
    pub(super) fn domain_at(&mut self, d: Ty, x: TExpr, span: Span) -> TExpr {
        let opt = self.optional_ty(d.clone());
        let mk = |kind, ty| TExpr { kind, ty, span };
        let tmp = self.bind("$at", Ty::Int);
        let read = || mk(TK::Var(tmp), Ty::Int);
        let has = mk(TK::DomHas(d.clone(), Box::new(read())), Ty::Bool);
        let pos = mk(TK::DomFrom(d.clone(), Box::new(read())), d.clone());
        let yes = self.some(pos, opt.clone());
        let none = vec![mk(TK::Bool(false), Ty::Bool), mk(TK::Unset, d)];
        let no = mk(TK::Struct(none), opt.clone());
        let pick = mk(
            TK::If(Box::new(has), Box::new(yes), Box::new(no)),
            opt.clone(),
        );
        mk(
            TK::Block(vec![TStmt::Let(tmp, x)], Some(Box::new(pick))),
            opt,
        )
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
