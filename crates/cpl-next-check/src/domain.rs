//! Domain declarations: their ends, factors and named ranges.

use super::*;

impl Cx<'_> {
    /// The start of a domain `lo..hi` or `lo..=hi`, the integer after its last position, and
    /// what the types of the two ends say every position lies within.
    pub(super) fn domain_ends(
        &mut self,
        lo: &ast::Expr,
        hi: &ast::Expr,
        inclusive: bool,
    ) -> (TExpr, TExpr, (Option<i64>, Option<i64>)) {
        // An end keeps the range type it was written with.
        let lo = self.expr_as_typed(lo, None);
        let hi = self.expr_as_typed(hi, None);
        self.typed_ends(lo, hi, inclusive)
    }
    fn typed_ends(
        &mut self,
        lo: TExpr,
        hi: TExpr,
        inclusive: bool,
    ) -> (TExpr, TExpr, (Option<i64>, Option<i64>)) {
        let ends = |e: &TExpr, ty: Ty| match (&e.kind, ty) {
            (TK::Int(k), _) => (Some(*k), Some(*k)),
            (_, Ty::Range(a, b)) => (Some(a), Some(b)),
            _ => (None, None),
        };
        let least = ends(&lo, self.shallow(&lo.ty)).0;
        let most = ends(&hi, self.shallow(&hi.ty)).1;
        let most = if inclusive {
            most
        } else {
            most.map(|m| m.saturating_sub(1))
        };
        let lo = self.coerce(lo, &Ty::Int);
        let hi = self.coerce(hi, &Ty::Int);
        let end = if let (true, TK::Int(k)) = (inclusive, &hi.kind) {
            let k = k.checked_add(1).map_or(TK::Error, TK::Int);
            TExpr { kind: k, ..hi }
        } else if inclusive {
            let span = hi.span;
            let one = TExpr {
                kind: TK::Int(1),
                ty: Ty::Int,
                span,
            };
            TExpr {
                kind: TK::Bin(BinOp::Add, Box::new(hi), Box::new(one), Overflow::Traps),
                ty: Ty::Int,
                span,
            }
        } else {
            hi
        };
        (lo, end, (least, most))
    }
    pub(super) fn declare_domain(
        &mut self,
        name: &str,
        lo: TExpr,
        end: TExpr,
        within: (Option<i64>, Option<i64>),
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> usize {
        let count = match (&lo.kind, &end.kind) {
            (TK::Int(a), TK::Int(b)) => Some((b - a).max(0)),
            _ => None,
        };
        let first = self.bind(&format!("{name}$lo"), Ty::Int);
        out.push(TStmt::Let(first, lo));
        let len = self.bind(&format!("{name}$len"), Ty::Int);
        // Spec 4: a range whose start is past its end is empty, never a trap.
        let tmp = self.bind(&format!("{name}$n"), Ty::Int);
        let var = |r| TExpr {
            kind: TK::Var(r),
            ty: Ty::Int,
            span,
        };
        let n = TExpr {
            kind: TK::Bin(
                BinOp::Sub,
                Box::new(end),
                Box::new(var(first)),
                Overflow::Traps,
            ),
            ty: Ty::Int,
            span,
        };
        out.push(TStmt::Let(tmp, n));
        let v = |k| TExpr {
            kind: k,
            ty: Ty::Int,
            span,
        };
        let neg = TExpr {
            kind: TK::Cmp(
                CmpOp::Lt,
                Box::new(v(TK::Var(tmp))),
                Box::new(v(TK::Int(0))),
            ),
            ty: Ty::Bool,
            span,
        };
        out.push(TStmt::Let(
            len,
            v(TK::If(
                Box::new(neg),
                Box::new(v(TK::Int(0))),
                Box::new(v(TK::Var(tmp))),
            )),
        ));
        let info = DomInfo {
            name: name.to_string(),
            len,
            lo: first,
            within,
            owner: self.decl,
            count,
        };
        if let (None, Some(&Ty::Dom(i))) = (self.decl, self.d.top_doms.get(name)) {
            if self.domains[i].name.is_empty() {
                self.domains[i] = info;
                self.doms.insert(name.to_string(), Ty::Dom(i));
                return i;
            }
        }
        self.domains.push(info);
        self.doms
            .insert(name.to_string(), Ty::Dom(self.domains.len() - 1));
        self.domains.len() - 1
    }
    /// A product's range factor is a domain of its own, declared here; the product then names it.
    pub(super) fn declare_factors(
        &mut self,
        product: &str,
        def: &ast::Expr,
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> ast::Expr {
        let xs: Vec<&ast::Expr> = match &def.kind {
            ExprKind::Tuple(xs) => xs.iter().collect(),
            ExprKind::Record(fs) => fs.iter().map(|(_, x)| x).collect(),
            _ => return def.clone(),
        };
        let mut parts = Vec::new();
        for (i, &x) in xs.iter().enumerate() {
            let ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step: None,
            } = &x.kind
            else {
                parts.push(x.clone());
                continue;
            };
            let name = decls::factor_name(product, i);
            let (lo, end, within) = self.domain_ends(lo, hi, *inclusive);
            self.declare_domain(&name, lo, end, within, span, out);
            parts.push(ast::Expr {
                kind: ExprKind::Name(name),
                span: x.span,
            });
        }
        let kind = match &def.kind {
            ExprKind::Record(fs) => {
                ExprKind::Record(fs.iter().map(|(n, _)| n.clone()).zip(parts).collect())
            }
            _ => ExprKind::Tuple(parts),
        };
        ast::Expr {
            kind,
            span: def.span,
        }
    }
    pub(super) fn plus_one(e: &ast::Expr) -> ast::Expr {
        Self::and_one(e, BinOp::Add)
    }
    pub(super) fn minus_one(e: &ast::Expr) -> ast::Expr {
        Self::and_one(e, BinOp::Sub)
    }
    fn and_one(e: &ast::Expr, op: BinOp) -> ast::Expr {
        let one = ast::Expr {
            kind: ExprKind::Int(1),
            span: e.span,
        };
        ast::Expr {
            kind: ExprKind::Binary {
                op,
                op_span: e.span,
                lhs: Box::new(e.clone()),
                rhs: Box::new(one),
            },
            span: e.span,
        }
    }
    fn int_range(&mut self, t: &Ty) -> bool {
        self.is_range(t)
            && matches!(self.shallow(t), Ty::Struct(_, ts, _) if self.shallow(&ts[0]) == Ty::Int)
    }
    /// `[r]T` for a range `r` of `Int`s covers just `r`'s keys (R04b.1.5): the ends of its domain
    /// are `r`'s, with `r` read once, in `out`.
    fn key_run(&mut self, r: TExpr, out: &mut Vec<TStmt>) -> (TExpr, TExpr) {
        let span = r.span;
        let name = format!("$keys{}", self.domains.len());
        let v = self.bind(&name, r.ty.clone());
        out.push(TStmt::Let(v, r));
        let end = |cx: &mut Self, which: &str| {
            let cut = ast::Expr {
                kind: ExprKind::Field {
                    base: Box::new(ast::Expr {
                        kind: ExprKind::Name(name.clone()),
                        span,
                    }),
                    name: ast::Name {
                        text: which.to_string(),
                        span,
                    },
                },
                span,
            };
            let int = ExprKind::Call {
                callee: Box::new(ast::Expr {
                    kind: ExprKind::Name("Int".to_string()),
                    span,
                }),
                args: vec![ast::Arg {
                    name: None,
                    mutable: false,
                    value: cut,
                }],
            };
            cx.expr(&ast::Expr { kind: int, span }, Some(&Ty::Int))
        };
        (end(self, "start"), end(self, "end"))
    }
    /// `[lo..hi]T` and `[n]T` index by a range with no name: each gets a domain of its own, declared in `out`.
    pub(super) fn name_ranges(
        &mut self,
        t: &ast::Type,
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> ast::Type {
        let mut t = t.clone();
        match &mut t.kind {
            TypeKind::Array { index, elem } => {
                **elem = self.name_ranges(elem, span, out);
                for ix in index.iter_mut() {
                    let key = decls::const_axis(ix).map(|(lo, hi)| decls::anon_key(lo, hi));
                    if let Some(text) = key.filter(|k| self.d.top_doms.contains_key(k)) {
                        let name = ast::Name {
                            text,
                            span: ix.span,
                        };
                        *ix = ast::Type {
                            kind: TypeKind::Named {
                                name,
                                args: Vec::new(),
                            },
                            span: ix.span,
                        };
                        continue;
                    }
                    let zero = || ast::Expr {
                        kind: ExprKind::Int(0),
                        span,
                    };
                    let ends = match &ix.kind {
                        TypeKind::Range {
                            lo: Some(lo),
                            hi: Some(hi),
                            inclusive,
                        } => {
                            // Its cells count from a number; one from a value is a domain (R04.3.4).
                            if lo.int_literal().is_none() {
                                self.err(
                                    span,
                                    Msg::verdict(cpl_diag::codes::TYPE_MISMATCH, "an array over a range that starts at a value: name it as a domain, `domain D = lo..hi`, then `[D]T`"),
                                );
                            }
                            Some(((**lo).clone(), (**hi).clone(), *inclusive))
                        }
                        TypeKind::Named { name, args }
                            if args.is_empty()
                                && self.lookup(&name.text).is_some()
                                && self.domain_named(&name.text).is_none() =>
                        {
                            Some((
                                zero(),
                                ast::Expr {
                                    kind: ExprKind::Name(name.text.clone()),
                                    span: name.span,
                                },
                                false,
                            ))
                        }
                        _ => None,
                    };
                    if let Some((lo, hi, inclusive)) = ends {
                        let (lo, hi) =
                            (self.expr_as_typed(&lo, None), self.expr_as_typed(&hi, None));
                        let (lo, hi) = match (&lo.kind, inclusive) {
                            (TK::Int(0), false) if self.int_range(&hi.ty) => self.key_run(hi, out),
                            _ => (lo, hi),
                        };
                        let (lo, end, within) = self.typed_ends(lo, hi, inclusive);
                        let name = format!("$range{}", self.domains.len());
                        self.declare_domain(&name, lo, end, within, span, out);
                        *ix = ast::Type {
                            kind: TypeKind::Named {
                                name: ast::Name {
                                    text: name,
                                    span: ix.span,
                                },
                                args: Vec::new(),
                            },
                            span: ix.span,
                        };
                    }
                }
            }
            TypeKind::Named { args, .. } => {
                for a in args.iter_mut() {
                    *a = self.name_ranges(a, span, out);
                }
            }
            // The ends are values: they are worked out once, here, and the range gets a name.
            TypeKind::Range { lo, hi, inclusive }
                if decls::static_range(lo, hi, *inclusive).is_none() =>
            {
                let name = format!("$ends{}", self.ranges.len());
                let mut end =
                    |cx: &mut Self, e: &Option<Box<ast::Expr>>, which: &str, open: bool| {
                        e.as_ref().map(|b| {
                            let b = if open {
                                Self::minus_one(b)
                            } else {
                                (**b).clone()
                            };
                            let b = cx.expr(&b, Some(&Ty::Int));
                            let b = cx.coerce(b, &Ty::Int);
                            let v = cx.bind(&format!("{name}${which}"), Ty::Int);
                            out.push(TStmt::Let(v, b));
                            v
                        })
                    };
                let lo = end(self, lo, "lo", false);
                let hi = end(self, hi, "hi", !*inclusive);
                self.ranges.push(RangeInfo { lo, hi });
                self.doms
                    .insert(name.clone(), Ty::RangeOf(self.ranges.len() - 1));
                t.kind = TypeKind::Named {
                    name: ast::Name {
                        text: name,
                        span: t.span,
                    },
                    args: Vec::new(),
                };
            }
            _ => {}
        }
        t
    }
}
