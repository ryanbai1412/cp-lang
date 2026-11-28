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
    ) -> (TExpr, TExpr, (Option<i64>, Option<i64>), bool) {
        // An end keeps the range type it was written with.
        let lo = self.expr_as_typed(lo, None);
        let hi = self.expr_as_typed(hi, None);
        // `'a'..='z'`: a `Char` domain's positions are its characters' codes (R04.2.24).
        let chars = self.shallow(&lo.ty) == Ty::Char;
        let code = |e: TExpr| match chars {
            true => TExpr {
                span: e.span,
                kind: TK::CodeOf(Box::new(e)),
                ty: Ty::Int,
            },
            false => e,
        };
        let hi = match chars {
            true => self.coerce(hi, &Ty::Char),
            false => hi,
        };
        let (lo, hi) = (code(lo), code(hi));
        let (lo, end, within) = self.typed_ends(lo, hi, inclusive);
        (lo, end, within, chars)
    }
    /// `domain V = e` where `e` is not a range: an `Int` count is `0..e`, a value with an impl
    /// of the `@role(counted)` trait has its count's positions, and any other type is E0302
    /// (R04.2.29). False for a form that is not built yet.
    pub(super) fn counted_domain(
        &mut self,
        name: &str,
        def: &ast::Expr,
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> bool {
        let (diagnosed, owed) = (self.errors.len(), self.b.obligations.len());
        let v = self.expr_as_typed(def, None);
        let t = self.shallow(&v.ty);
        // A form this checker cannot build yet (`read(1..=100)`, `V + K`) leaves no trace.
        // An owed bound no impl meets (`Range<V>: Add`) is a form not built yet too.
        let unmet = (self.b.obligations[owed..].iter()).any(|(t, r, ..)| {
            let t = self.shallow(t);
            !matches!(t, Ty::Var(_))
                && !(self.d.impls_of(r.tr)).any(|im| self.d.fits_impl(im, &t).is_some())
        });
        if self.errors.len() > diagnosed
            || unmet
            || matches!(t, Ty::Error | Ty::Var(_) | Ty::Param(_))
        {
            self.errors.truncate(diagnosed);
            self.b.obligations.truncate(owed);
            return false;
        }
        let at = |kind, ty| TExpr {
            kind,
            ty,
            span: def.span,
        };
        let count = match (&t, self.d.roles().counted) {
            (Ty::Int | Ty::Range(..) | Ty::RangeOf(_) | Ty::RangeTo(..), _) => Some(v),
            (_, Some(tr)) if self.has_impl(&t, tr) => self.count_of(tr, v, out),
            _ => None,
        };
        let Some(count) = count else {
            let what = format!(
                "a domain is a count, a range of `Int`s, a sequence's positions or another \
                 domain, not `{}`",
                self.show(&t)
            );
            self.err(def.span, Msg::verdict(cpl_diag::codes::TYPE_MISMATCH, what));
            // A top-level domain decls set aside is declared here, though it has no positions.
            if let Some(&Ty::Dom(i)) = self.b.doms.get(name) {
                self.domains[i].name = name.to_string();
            }
            self.b.doms.insert(name.to_string(), Ty::Error);
            return true;
        };
        let (lo, end, within) = self.typed_ends(at(TK::Int(0), Ty::Int), count, false);
        let i = self.declare_domain(name, lo, end, within, span, out);
        self.domains[i].counted = true;
        true
    }
    /// `xs`'s count through its impl of the `@role(counted)` trait `tr`, `xs` read once in `out`.
    fn count_of(&mut self, tr: TraitId, xs: TExpr, out: &mut Vec<TStmt>) -> Option<TExpr> {
        let (span, ty) = (xs.span, xs.ty.clone());
        let &(n, m) = self.d.trait_fns(tr).first()?;
        let method = self.d.text(n);
        // A variable is read where it is; any other value is held first, so it runs once.
        let recv = match xs.kind {
            TK::Var(_) => xs,
            _ => {
                let v = self.bind(&format!("$counted{}", self.domains.len()), ty.clone());
                out.push(TStmt::Let(v, xs));
                TExpr {
                    kind: TK::Var(v),
                    ty: ty.clone(),
                    span,
                }
            }
        };
        self.reach(m);
        let r = decls::TraitRef { tr, args: vec![] };
        (self.b.obligations).push((ty.clone(), r.clone(), span, Origin::Bound));
        let kind = TK::CallTrait {
            ctx: self.trait_ctx(r.tr, method),
            tr: r,
            method: method.to_string(),
            recv: Some(Box::new(TArg::Value(recv))),
            self_ty: Box::new(ty.clone()),
            targs: Box::new([]),
            args: vec![],
        };
        let ty = self.trait_ret(m, vec![ty]);
        Some(TExpr { kind, ty, span })
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
                kind: TK::Bin(BinOp::Add, Box::new(hi), Box::new(one)),
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
        let info = self.domain_info(name, lo, end, within, span, out);
        let i = match (self.b.decl, self.d.top_doms.get(name)) {
            (None, Some(&Ty::Dom(i))) if self.domains[i].name.is_empty() => {
                self.domains[i] = info;
                i
            }
            _ => {
                let i = self.id(super::Table::Domain, self.domains.len());
                super::put(&mut self.domains, i, info);
                i
            }
        };
        self.b.doms.insert(name.to_string(), Ty::Dom(i));
        self.b.block_doms.push(i);
        i
    }
    /// A domain's slots, set from its ends; `name` is what a message and the trace call it.
    pub(super) fn domain_info(
        &mut self,
        name: &str,
        lo: TExpr,
        end: TExpr,
        within: (Option<i64>, Option<i64>),
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> DomInfo {
        let count = match (&lo.kind, &end.kind) {
            (TK::Int(a), TK::Int(b)) => Some((b - a).max(0)),
            _ => None,
        };
        let first = self.slot(Ty::Int, "$lo");
        out.push(TStmt::Let(first, lo));
        let len = self.slot(Ty::Int, "$len");
        // Spec 4: a range whose start is past its end is empty, never a trap.
        let tmp = self.slot(Ty::Int, "$n");
        let var = |r| TExpr {
            kind: TK::Var(r),
            ty: Ty::Int,
            span,
        };
        let n = TExpr {
            kind: TK::Bin(BinOp::Sub, Box::new(end), Box::new(var(first))),
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
        DomInfo {
            name: name.to_string(),
            len,
            lo: first,
            within,
            owner: self.b.decl,
            count,
            chars: false,
            counted: false,
        }
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
            let (lo, end, within, chars) = self.domain_ends(lo, hi, *inclusive);
            let i = self.declare_domain(&name, lo, end, within, span, out);
            self.domains[i].chars = chars;
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
        let r_ty = r.ty.clone();
        let v = self.bind(&name, r_ty.clone());
        out.push(TStmt::Let(v, r));
        let read = TExpr {
            kind: TK::Var(v),
            ty: r_ty.clone(),
            span,
        };
        let bounds = self.d.roles().bounds;
        let fns = bounds.map(|tr| self.d.trait_fns(tr)).unwrap_or_default();
        let (Some(tr), [lo, hi, ..]) = (bounds, &fns[..]) else {
            self.err(
                span,
                Msg::internal("no `@role(range_bounds)` trait with two fns"),
            );
            let e = || TExpr {
                kind: TK::Error,
                ty: Ty::Error,
                span,
            };
            return (e(), e());
        };
        let args = match self.shallow(&r_ty) {
            Ty::Struct(_, ts, _) => ts,
            _ => Vec::new(),
        };
        // Each end is the bound's fn on `r`, held in a local that `Int(..)` reads (R04b.1.5).
        let end =
            |cx: &mut Self, &(n, m): &(cpl_next_decls::sym::Sym, FnId), out: &mut Vec<TStmt>| {
                let n = cx.d.text(n);
                cx.reach(m);
                let mut tys = vec![r_ty.clone()];
                tys.extend(args.iter().cloned());
                let ty = cx.trait_ret(m, tys);
                let call = TK::CallTrait {
                    ctx: cx.trait_ctx(tr, n),
                    tr: decls::TraitRef {
                        tr,
                        args: args.clone(),
                    },
                    method: n.to_string(),
                    recv: Some(Box::new(TArg::Value(read.clone()))),
                    self_ty: Box::new(r_ty.clone()),
                    targs: Box::new([]),
                    args: vec![],
                };
                let r = decls::TraitRef {
                    tr,
                    args: args.clone(),
                };
                (cx.b.obligations).push((r_ty.clone(), r, span, Origin::Bound));
                let c = cx.slot(ty.clone(), n);
                let at = |kind| TExpr {
                    kind,
                    ty: ty.clone(),
                    span,
                };
                out.push(TStmt::Let(c, at(call)));
                cx.convert_value(Ty::Int, at(TK::Var(c)), span, Origin::Convert)
            };
        let lo = end(self, lo, out);
        (lo, end(self, hi, out))
    }
    /// `lo..D.m + k` with a constant `lo`, `D` a domain in view and `m` its getter: a range type
    /// ending at the domain's count.
    fn count_ended(
        &self,
        lo: &Option<Box<ast::Expr>>,
        hi: &Option<Box<ast::Expr>>,
        inclusive: bool,
    ) -> bool {
        decls::static_range(lo, hi, inclusive).is_none()
            && decls::count_end(lo, hi, inclusive).is_some_and(|(_, d, m, _)| {
                (self.b.generics.iter().any(|g| g == d) || self.b.doms.contains_key(d))
                    && self.d.domain_getter(m).is_some()
            })
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
                    // A constant axis stays a range type: `[0..2]T` is `Array<0..=1, T>`.
                    if decls::const_axis(ix).is_some() {
                        continue;
                    }
                    // So does one ending at a domain's count: `[0..V.count - 1]T` (R04.3.4).
                    if let TypeKind::Range { lo, hi, inclusive } = &ix.kind {
                        if self.count_ended(lo, hi, *inclusive) {
                            *ix = self.name_ranges(ix, span, out);
                            continue;
                        }
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
            // `0..=V.count`: its end is the domain's, so a field of that type holds it.
            TypeKind::Range { lo, hi, inclusive } if self.count_ended(lo, hi, *inclusive) => {
                let (lo, d, m, k) = decls::count_end(lo, hi, *inclusive).unwrap_or_default();
                let Some(m) = self.d.domain_getter(m) else {
                    return t.clone();
                };
                let dt = ast::Type::named(d, t.span);
                let (dt, errs) = decls::body_type(
                    self.d,
                    self.d,
                    self.b.module,
                    &self.b.generics,
                    &self.b.doms,
                    &|n: &str| self.owned_named(n),
                    &dt,
                );
                for e in errs {
                    self.err(span, e);
                }
                let name = format!("$to{}", self.b.doms.len());
                let ty = Ty::RangeTo(Box::new(dt), m, lo, k);
                self.b.doms.insert(name.clone(), ty);
                t.kind = TypeKind::Named {
                    name: ast::Name {
                        text: name,
                        span: t.span,
                    },
                    args: Vec::new(),
                };
            }
            TypeKind::Range { lo, hi, inclusive }
                if decls::static_range(lo, hi, *inclusive).is_none() =>
            {
                let name = format!("$ends{}", self.ranges.len());
                let mut end = |cx: &mut Self, e: &Option<Box<ast::Expr>>, open: bool| {
                    e.as_ref().map(|b| {
                        let b = if open {
                            Self::minus_one(b)
                        } else {
                            (**b).clone()
                        };
                        let b = cx.expr(&b, Some(&Ty::Int));
                        let b = cx.coerce(b, &Ty::Int);
                        let v = cx.slot(Ty::Int, "$end");
                        out.push(TStmt::Let(v, b));
                        v
                    })
                };
                let lo = end(self, lo, false);
                let hi = end(self, hi, !*inclusive);
                // A signature's range ending at a top-level `let` has its slot by where it is
                // written; the top level fills it once, after the `let` (R02.10.3).
                let at = self.b.item.at(t.span);
                let i = match self.d.top_ranges.iter().position(|r| r.at == at) {
                    Some(i) => i,
                    None => self.id(super::Table::Range, self.ranges.len()),
                };
                super::put(&mut self.ranges, i, RangeInfo { lo, hi });
                self.b.doms.insert(name.clone(), Ty::RangeOf(i));
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

/// The domains `t` mentions.
pub(super) fn doms_in(t: &Ty, out: &mut Vec<usize>) {
    match t {
        Ty::Dom(d) => out.push(*d),
        Ty::Raw(e) | Ty::Ref(e) | Ty::Lent(e) | Ty::Mut(e) | Ty::RangeTo(e, ..) => doms_in(e, out),
        Ty::Struct(_, ts, _) => ts.iter().for_each(|t| doms_in(t, out)),
        Ty::Fn(ps, r, _) => ps.iter().chain([&**r]).for_each(|t| doms_in(t, out)),
        _ => {}
    }
}
