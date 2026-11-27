//! Ranges and domains as values (R01.8.4, R04.2).
use super::expr::block;
use super::*;
use cpl_next_ir::unbuilt::Unbuilt;

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
        let mut pre = Vec::new();
        let dty = match self.written_struct(base, span, &mut pre) {
            Some(t) => Some(t),
            None => self.type_path(base),
        };
        if let Some(dty) = dty {
            let found = match &dty {
                Ty::Dom(_) => self.domain_fn(dty.clone(), name, call, span),
                Ty::Struct(..) => self.type_static(dty.clone(), name, call, span),
                _ => None,
            };
            if let Some(e) = found {
                return Some(super::expr::block(pre, e));
            }
            // `D.m` for a `@member(m)` struct, named as a value: the domain of them; `D.m(x)`
            // converts `x` to one (R04.2.5, R04.2.17).
            if let Some(t) = self.member_type(&dty, &name.text) {
                return match call {
                    None => Some(self.domain_value(t, span)),
                    Some([a]) if a.name.is_none() => {
                        Some(self.convert(t, &a.value, span, Origin::Convert))
                    }
                    Some(_) => None,
                };
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
    /// `D.count`, `D.nth(k)`, `D.at(x)` and `D.all`: the core's `Domain` declares them; each
    /// `@intrinsic` one is the domain operation it names (R04.2.8, R04.2.9).
    fn domain_fn(
        &mut self,
        dty: Ty,
        name: &ast::Name,
        call: Option<&[ast::Arg]>,
        span: Span,
    ) -> Option<TExpr> {
        let tr = self.d.domain_trait?;
        let &m = self.d.traits[tr.0].methods.get(&name.text)?;
        let fd = self.d.f(m);
        if !super::member::takes(fd, call) || (call.is_none() && !fd.getter) {
            return None;
        }
        let Some(op) = fd.intrinsic.clone() else {
            let r = decls::TraitRef {
                tr,
                args: Vec::new(),
            };
            return Some(self.static_call(dty, r, m, name, call, span));
        };
        let args = match call {
            None => None,
            Some([a]) => {
                let x = self.expr(&a.value, Some(&Ty::Int));
                Some(self.coerce(x, &Ty::Int))
            }
            Some(_) => return None,
        };
        self.domain_op(dty, &op, args, span)
    }
    /// The domain operation `op` of `D`, with its one `Int` argument if it takes one. The body
    /// of each of `Domain`'s fns that a domain meets without an impl.
    pub(super) fn domain_op(
        &mut self,
        dty: Ty,
        op: &str,
        arg: Option<TExpr>,
        span: Span,
    ) -> Option<TExpr> {
        Some(match (op, arg) {
            ("dom_count", None) => TExpr {
                kind: TK::DomLen(dty),
                ty: Ty::Int,
                span,
            },
            ("dom_nth", Some(k)) => TExpr {
                kind: TK::DomNth(dty.clone(), Box::new(k)),
                ty: dty,
                span,
            },
            ("dom_at", Some(x)) => self.domain_at(dty, x, span),
            ("dom_ord", Some(v)) => TExpr {
                kind: TK::Ordinal(Box::new(v)),
                ty: Ty::Int,
                span,
            },
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
        want: Option<&Ty>,
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
            _ => return self.refuse(span, Unbuilt::TableOverIndex),
        };
        let name = match &dty {
            Ty::Dom(i) if !self.domains[*i].name.is_empty() => self.domains[*i].name.clone(),
            // A type that implements `Domain` (`[V.cuts] c => ..`) walks its positions too.
            Ty::Struct(..) if self.is_domain(&dty) => {
                let all = self.domain_value(dty.clone(), dom.span);
                let tmp = self.bind("$walk", all.ty.clone());
                pre.push(TStmt::Let(tmp, all));
                "$walk".to_string()
            }
            _ => return self.refuse(span, Unbuilt::TableOverIndex),
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
        // The cells' type, when the table is written where an array is expected.
        let cell = match want.map(|w| self.shallow(w)) {
            Some(Ty::Struct(a, ts, _)) if a == array && ts.len() == 2 => Some(ts[1].clone()),
            _ => None,
        };
        let want_list = match cell {
            Some(c) => self
                .list_struct(span)
                .map(|l| Ty::Struct(l, vec![c], Vec::new())),
            None => None,
        };
        let list = self.expr(&comp, want_list.as_ref());
        let elem = match self.shallow(&list.ty) {
            Ty::Struct(_, ts, _) if ts.len() == 1 => ts[0].clone(),
            _ => Ty::Error,
        };
        let cells = self.coerce(list, &Ty::Struct(array, vec![dty, elem], sets));
        block(pre, cells)
    }
    fn refuse(&mut self, span: Span, why: Unbuilt) -> TExpr {
        self.err(span, why);
        TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
        }
    }
    /// A type named in an expression: a domain, a type parameter, or `D.m` for a `@member(m)`
    /// struct, as `V.cuts` is `Cut<V>`.
    pub(super) fn type_path(&self, e: &ast::Expr) -> Option<Ty> {
        match &e.kind {
            ExprKind::Name(n) if self.lookup(n).is_none() => self.domain_named(n),
            ExprKind::Field { base, name } => {
                let b = self.type_path(base)?;
                self.member_type(&b, &name.text)
            }
            // An inline tuple of domains is the product domain (R04.2.11).
            ExprKind::Tuple(xs) if xs.len() > 1 => {
                let &tuple = self.d.tuples.get(&xs.len())?;
                let parts = xs
                    .iter()
                    .map(|x| self.type_path(x))
                    .collect::<Option<Vec<_>>>()?;
                Some(Ty::Struct(tuple, parts, Vec::new())).filter(|t| self.is_domain(t))
            }
            // So is a record of them, its factors named (R04.2.10).
            ExprKind::Record(fs) => {
                let names: Vec<String> = fs.iter().map(|(n, _)| n.text.clone()).collect();
                let record = *self.d.records.get(&names)?;
                let parts = (fs.iter())
                    .map(|(_, x)| self.type_path(x))
                    .collect::<Option<Vec<_>>>()?;
                Some(Ty::Struct(record, parts, Vec::new())).filter(|t| self.is_domain(t))
            }
            _ => None,
        }
    }
    /// `Subset<V>` written as a value: the struct type at the written arguments.
    pub(super) fn written_struct(
        &mut self,
        e: &ast::Expr,
        span: Span,
        pre: &mut Vec<TStmt>,
    ) -> Option<Ty> {
        let ExprKind::Generic { base, args } = &e.kind else {
            return None;
        };
        self.struct_at(base, args, span, pre)
    }
    /// The struct `base` names at the type arguments `args`.
    pub(super) fn struct_at(
        &mut self,
        base: &ast::Expr,
        args: &[ast::Type],
        span: Span,
        pre: &mut Vec<TStmt>,
    ) -> Option<Ty> {
        let ExprKind::Name(n) = &base.kind else {
            return None;
        };
        if self.lookup(n).is_some() || self.d.struct_named(self.module, n).is_none() {
            return None;
        }
        let t = ast::Type {
            kind: TypeKind::Named {
                name: ast::Name {
                    text: n.clone(),
                    span: base.span,
                },
                args: args.to_vec(),
            },
            span: base.span,
        };
        Some(self.body_ty(&t, span, pre)).filter(|t| matches!(t, Ty::Struct(..)))
    }
    /// `D.m`: the `@member(m)` struct at `D` (R06).
    fn member_type(&self, d: &Ty, m: &str) -> Option<Ty> {
        let &id = self.d.member_types.get(m)?;
        (self.d.s(id).n_sets == 0).then(|| Ty::Struct(id, vec![d.clone()], Vec::new()))
    }
    /// `D(x)`: the position of `x`, checked; a library domain (`Cell((0, 2))` of a product)
    /// converts by its `Into` impl.
    pub(super) fn domain_from(&mut self, dty: Ty, x: &ast::Expr, span: Span) -> TExpr {
        if let Ty::Struct(..) = dty {
            return self.convert(dty, x, span, Origin::Convert);
        }
        let v = self.expr(x, Some(&Ty::Int));
        let v = self.coerce(v, &Ty::Int);
        TExpr {
            kind: TK::DomFrom(dty.clone(), Box::new(v)),
            ty: dty,
            span,
        }
    }
    /// A name that is a domain: one declared in a body, or a type parameter used as one.
    pub(super) fn domain_named(&self, n: &str) -> Option<Ty> {
        // A fn's own type parameter is nearer than any domain declared outside it.
        if let Some(p) = self.generics.iter().position(|g| g == n) {
            return Some(Ty::Param(p));
        }
        if let Some(t @ Ty::Struct(..)) = self.doms.get(n) {
            return Some(t.clone());
        }
        // `type Cell = (Row, Col)` names the product domain too (R04.2.10).
        if let Some(decls::Named::Alias(_)) = self.d.views[self.module.0].get(n) {
            let written = ast::Type {
                kind: ast::TypeKind::Named {
                    name: ast::Name {
                        text: n.to_string(),
                        span: Span::default(),
                    },
                    args: Vec::new(),
                },
                span: Span::default(),
            };
            let (gs, doms) = (&self.generics, &self.doms);
            let t = decls::body_type(self.d, self.module, gs, doms, &written, &mut Vec::new());
            return Some(t).filter(|t| self.is_domain(t));
        }
        if let Some(&Ty::Dom(i)) = self.doms.get(n) {
            if self.decl.is_none() && self.domains[i].name.is_empty() {
                self.early.set(Some(i));
            }
            return Some(Ty::Dom(i));
        }
        None
    }
    /// `lo..hi`, `lo..=hi` and `lo..hi by k` as a value: core's `Range` (R01.8.4).
    pub(super) fn range_value(
        &mut self,
        lo: Option<&ast::Expr>,
        hi: Option<&ast::Expr>,
        inclusive: bool,
        step: Option<&ast::Expr>,
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let diagnosed = self.errors.len();
        let (args, d) = match (lo, hi) {
            (Some(lo), Some(hi)) => {
                let first = self.expr(lo, Some(&Ty::Int));
                let first = match self.whole_of(want) {
                    Some(d @ Ty::Dom(_)) if step.is_none() && self.shallow(&first.ty) == Ty::Int => {
                        self.coerce(first, &d)
                    }
                    _ => first,
                };
                return self.range_from(first, hi, inclusive, step, want, span);
            }
            _ => match self.open_ends(lo, hi, inclusive, step, want, span) {
                Ok(ends) => ends,
                Err(e) => return *e,
            },
        };
        self.range_built(args, d, diagnosed, span)
    }
    /// `lo..hi` whose `lo` is checked already, as `first`.
    pub(super) fn range_from(
        &mut self,
        first: TExpr,
        hi: &ast::Expr,
        inclusive: bool,
        step: Option<&ast::Expr>,
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let diagnosed = self.errors.len();
        let (args, d) = match self.ord_ends(&first.ty, step) {
            Some(ord) => self.domain_range(first, hi, inclusive, want, ord, span),
            None => (self.int_ends(first, hi, inclusive, step, span), Ty::Int),
        };
        self.range_built(args, d, diagnosed, span)
    }
    /// The range of positions `args` of `d`, unless checking its ends found errors.
    fn range_built(&mut self, args: Vec<TExpr>, d: Ty, diagnosed: usize, span: Span) -> TExpr {
        if self.errors.len() > diagnosed {
            return Self::error_node(span);
        }
        self.range_of(args, d, span)
    }
    /// `lo..` or `..hi`: the open end runs to `Int.MAX` or the end of the given end's domain, or
    /// starts at 0 or the domain's first value (R01.8.4, R04.4.8, R04.4.10).
    fn open_ends(
        &mut self,
        lo: Option<&ast::Expr>,
        hi: Option<&ast::Expr>,
        inclusive: bool,
        step: Option<&ast::Expr>,
        want: Option<&Ty>,
        span: Span,
    ) -> Result<(Vec<TExpr>, Ty), Box<TExpr>> {
        let diagnosed = self.errors.len();
        let from = lo.is_some();
        let Some(given) = lo.or(hi) else {
            return Err(Box::new(Self::error_node(span)));
        };
        let v = self.expr(given, Some(&Ty::Int));
        if self.errors.len() > diagnosed {
            return Err(Box::new(Self::error_node(span)));
        }
        let t = self.shallow(&v.ty);
        let int = |kind| TExpr {
            kind,
            ty: Ty::Int,
            span,
        };
        let past = |e: TExpr| {
            let one = Box::new(int(TK::Int(1)));
            int(TK::Bin(BinOp::Add, Box::new(e), one, Overflow::Traps))
        };
        if let Some(ord) = self.ord_ends(&t, None) {
            if step.is_some() {
                self.err(span, Unbuilt::ByOnWalk);
                return Err(Box::new(Self::error_node(span)));
            }
            let cuts = self.cut_domain(&t);
            let piece = match want.map(|w| self.shallow(w)) {
                Some(Ty::Struct(_, a, _)) if a.len() == 1 => {
                    cuts.clone().filter(|d| self.shallow(&a[0]) == *d)
                }
                _ => None,
            };
            if let Some(d) = piece {
                let (l, h) = if from {
                    (Some(v), None)
                } else {
                    (None, Some(v))
                };
                return Err(Box::new(
                    (self.cut_piece(l, h, inclusive, d, span)).unwrap_or(Self::error_node(span)),
                ));
            }
            let at = self.end_position(v, &ord);
            let (l, h) = match (from, cuts) {
                (true, Some(d)) => (at, past(self.domain_len(d, span))),
                (true, None) => (at, self.domain_len(t.clone(), span)),
                (false, _) => (int(TK::Int(0)), if inclusive { past(at) } else { at }),
            };
            return Ok((vec![l, h, int(TK::Int(1))], t));
        }
        if !matches!(t, Ty::Int | Ty::Range(..)) {
            let shown = self.show(&t);
            let code = cpl_diag::codes::TYPE_MISMATCH;
            let msg = format!("an open range's end is an `Int` or a domain value, not `{shown}`");
            self.err(given.span, Msg::verdict(code, msg));
            return Err(Box::new(Self::error_node(span)));
        }
        let v = self.coerce(v, &Ty::Int);
        let (l, h) = match from {
            true => (v, int(TK::Int(i64::MAX))),
            false => (int(TK::Int(0)), if inclusive { past(v) } else { v }),
        };
        let step = match step {
            Some(k) => {
                let k = self.expr(k, Some(&Ty::Int));
                self.coerce(k, &Ty::Int)
            }
            None => int(TK::Int(1)),
        };
        Ok((vec![l, h, step], Ty::Int))
    }
    /// How a range's ends give their positions, when they are values of a domain: a declared
    /// domain's value is its ordinal, and any other type's comes from its `Domain` impl or bound.
    pub(super) fn ord_ends(&self, t: &Ty, step: Option<&ast::Expr>) -> Option<Ends> {
        let tr = self.d.domain_trait?;
        if step.is_some() {
            return None;
        }
        match self.shallow(t) {
            Ty::Dom(_) => Some(Ends::Dom),
            t @ Ty::Struct(..) if self.d.has_impl(&t, tr) => {
                self.domain_method("dom_ord").map(Ends::Ord)
            }
            Ty::Param(p) if self.bound_traits(p).iter().any(|b| b.tr == tr) => {
                self.domain_method("dom_ord").map(Ends::Ord)
            }
            _ => None,
        }
    }
    /// The position of a range end `v`.
    fn end_position(&mut self, v: TExpr, ends: &Ends) -> TExpr {
        match ends {
            Ends::Ord(ord) => self.ordinal(v, ord),
            Ends::Dom => TExpr {
                span: v.span,
                kind: TK::Ordinal(Box::new(v)),
                ty: Ty::Int,
            },
        }
    }
    /// The `lo, hi, step` integers of an `Int` range.
    fn int_ends(
        &mut self,
        first: TExpr,
        hi: &ast::Expr,
        inclusive: bool,
        step: Option<&ast::Expr>,
        span: Span,
    ) -> Vec<TExpr> {
        let lo = self.int_end(first);
        let hi = self.expr(hi, Some(&Ty::Int));
        let mut hi = self.int_end(hi);
        if inclusive {
            let one = TExpr {
                kind: TK::Int(1),
                ty: Ty::Int,
                span,
            };
            hi = TExpr {
                kind: TK::Bin(BinOp::Add, Box::new(hi), Box::new(one), Overflow::Traps),
                ty: Ty::Int,
                span,
            };
        }
        let step = match step {
            Some(k) => {
                let k = self.expr(k, Some(&Ty::Int));
                self.int_end(k)
            }
            None => TExpr {
                kind: TK::Int(1),
                ty: Ty::Int,
                span,
            },
        };
        vec![lo, hi, step]
    }
    /// An `Int` range's end, step or bound.
    fn int_end(&mut self, v: TExpr) -> TExpr {
        if !matches!(self.shallow(&v.ty), Ty::Int | Ty::Range(..) | Ty::Error) {
            let t = self.show(&v.ty);
            self.err(v.span, Unbuilt::RangeOfEnds { ty: t });
            return Self::error_node(v.span);
        }
        self.coerce(v, &Ty::Int)
    }
    /// Std's `@range_literal` fn `(lo, hi, step)` over positions of `d`.
    fn range_of(&mut self, args: Vec<TExpr>, d: Ty, span: Span) -> TExpr {
        let r = self.marked_call(self.d.sugar.range_literal, "range_literal", args, span);
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
    /// `D.count`.
    fn domain_len(&mut self, d: Ty, span: Span) -> TExpr {
        // A type parameter's count is its instance's: a product or library domain has its own.
        match (&d, self.domain_method("dom_count")) {
            (Ty::Struct(..) | Ty::Param(_), Some(count)) => {
                self.domain_count(d.clone(), count, span)
            }
            _ => TExpr {
                kind: TK::DomLen(d.clone()),
                ty: Ty::Int,
                span,
            },
        }
    }
    /// A domain named as a value: its whole range (R04.2.21).
    pub(super) fn domain_value(&mut self, d: Ty, span: Span) -> TExpr {
        self.ord_piece(None, None, d, span)
    }
    /// `j..i` with ends of a type that implements `Domain`, as cuts: the range of them, or, for
    /// a half-open range of cuts where a `Range<D>` is wanted, the piece of `D`'s positions
    /// between them (R04.4.1, R04.4.4).
    fn domain_range(
        &mut self,
        lo: TExpr,
        hi: &ast::Expr,
        inclusive: bool,
        want: Option<&Ty>,
        ord: Ends,
        span: Span,
    ) -> (Vec<TExpr>, Ty) {
        let t = self.shallow(&lo.ty);
        let hi = self.expr(hi, Some(&t));
        let hi = self.coerce(hi, &t);
        let cuts = self.cut_domain(&t);
        let piece = match want.map(|w| self.shallow(w)) {
            Some(Ty::Struct(_, a, _)) if a.len() == 1 && cuts == Some(self.shallow(&a[0])) => {
                Some(a[0].clone())
            }
            _ => None,
        };
        if piece.is_some() && inclusive {
            self.closed_cuts(&t, span);
        }
        let lo = self.end_position(lo, &ord);
        let mut hi = self.end_position(hi, &ord);
        let int = |kind| TExpr {
            kind,
            ty: Ty::Int,
            span,
        };
        if inclusive {
            hi = int(TK::Bin(
                BinOp::Add,
                Box::new(hi),
                Box::new(int(TK::Int(1))),
                Overflow::Traps,
            ));
        }
        (vec![lo, hi, int(TK::Int(1))], piece.unwrap_or(t))
    }
    /// `D`, when `t` is core's `@cuts` struct of `D`.
    pub(super) fn cut_domain(&self, t: &Ty) -> Option<Ty> {
        match self.shallow(t) {
            Ty::Struct(c, of, _) if Some(c) == self.d.cut_struct => {
                of.first().map(|d| self.shallow(d))
            }
            _ => None,
        }
    }
    fn closed_cuts(&mut self, t: &Ty, span: Span) {
        let code = cpl_diag::codes::TYPE_MISMATCH;
        let shown = self.show(t);
        self.err(
            span,
            Msg::verdict(
                code,
                format!(
                    "a closed range of `{shown}` is no piece of positions: write `j..i` (R04.4.6)"
                ),
            ),
        );
    }
    /// `..c` or `c..` with `c` a cut of `d`: the piece of `d`'s positions before or after it
    /// (R04.4.8), as a `Range<D>`.
    pub(super) fn cut_piece(
        &mut self,
        lo: Option<TExpr>,
        hi: Option<TExpr>,
        through: bool,
        d: Ty,
        span: Span,
    ) -> Option<TExpr> {
        let ord = self.domain_method("dom_ord")?;
        if through {
            let t = (lo.iter().chain(&hi)).next().map(|e| e.ty.clone())?;
            self.closed_cuts(&t, span);
        }
        let lo = lo.map(|e| self.ordinal(e, &ord));
        let hi = hi.map(|e| self.ordinal(e, &ord));
        Some(self.ord_piece(lo, hi, d, span))
    }
    /// The positions of `d` with ordinals `lo..hi`, from the first or to the last when left out.
    fn ord_piece(&mut self, lo: Option<TExpr>, hi: Option<TExpr>, d: Ty, span: Span) -> TExpr {
        let int = |kind| TExpr {
            kind,
            ty: Ty::Int,
            span,
        };
        let lo = lo.unwrap_or(int(TK::Int(0)));
        let hi = hi.unwrap_or_else(|| self.domain_len(d.clone(), span));
        self.range_of(vec![lo, hi, int(TK::Int(1))], d, span)
    }
    /// The fn of core's `Domain` whose body is the intrinsic `op`.
    fn domain_method(&self, op: &str) -> Option<DomainFn> {
        let tr = self.d.domain_trait?;
        let ms = &self.d.traits[tr.0].methods;
        let (n, &m) = ms
            .iter()
            .find(|(_, &m)| self.d.f(m).intrinsic.as_deref() == Some(op))?;
        let r = decls::TraitRef {
            tr,
            args: Vec::new(),
        };
        Some((r, n.clone(), m))
    }
    /// `v.ord` of a value whose type implements `Domain`, through the trait.
    fn ordinal(&mut self, v: TExpr, ord: &DomainFn) -> TExpr {
        let span = v.span;
        let (r, method, m) = ord.clone();
        self.reach(m);
        self.obligations
            .push((v.ty.clone(), r.clone(), span, Origin::Bound));
        TExpr {
            kind: TK::CallTrait {
                tr: r,
                method,
                self_ty: Box::new(v.ty.clone()),
                recv: Some(Box::new(TArg::Value(v))),
                args: Vec::new(),
            },
            ty: Ty::Int,
            span,
        }
    }
    /// `D.count` of a type that implements `Domain`: its impl's, through the trait.
    fn domain_count(&mut self, d: Ty, count: DomainFn, span: Span) -> TExpr {
        let (r, n, m) = count;
        self.obligations
            .push((d.clone(), r.clone(), span, Origin::Bound));
        let name = ast::Name { text: n, span };
        self.static_call(d, r, m, &name, None, span)
    }
}
/// A fn of core's `Domain`: the trait, the fn's name in it, and the fn.
type DomainFn = (decls::TraitRef, String, FnId);

impl Cx<'_> {
    /// A position of a domain: of a declared one, or of a type parameter used as one.
    pub(super) fn is_position(&self, t: &Ty) -> bool {
        match self.shallow(t) {
            Ty::Dom(_) => true,
            Ty::Param(i) => {
                (self.bound_traits(i).iter()).any(|b| Some(b.tr) == self.d.domain_trait)
            }
            _ => false,
        }
    }
    /// `v + k`, `v - k` and `v - w` on positions are the `Int` arithmetic of their integers; any
    /// other operator takes `Int(v)` first (R04.3.3). Not a position's operator: `l` and `r` back.
    pub(super) fn position_arith(
        &mut self,
        op: BinOp,
        l: TExpr,
        r: TExpr,
        span: Span,
    ) -> Result<TExpr, Box<(TExpr, TExpr)>> {
        let (lp, rp) = (self.is_position(&l.ty), self.is_position(&r.ty));
        if !(lp || rp) || matches!(op, BinOp::And | BinOp::Or | BinOp::Coalesce) {
            return Err(Box::new((l, r)));
        }
        let int = |e: TExpr| TExpr {
            span: e.span,
            kind: TK::DomOrd(Box::new(e)),
            ty: Ty::Int,
        };
        let same = rp && self.shallow(&l.ty) == self.shallow(&r.ty);
        match op {
            BinOp::Add | BinOp::Sub if lp && (!rp || op == BinOp::Sub && same) => {
                let r = if rp { int(r) } else { r };
                Ok(self.binary(op, int(l), r, span))
            }
            BinOp::Add | BinOp::Sub if lp && rp => Err(Box::new((l, r))),
            _ => {
                let v = if lp { &l } else { &r };
                let name = self.show(&v.ty);
                let text =
                    cpl_diag::msgs::check::BAD_OPERANDS_ON_A_POSITION.say(&[&op.text(), &name]);
                self.err(span, Msg::verdict(cpl_diag::codes::BAD_OPERANDS, text));
                Ok(Self::error_node(span))
            }
        }
    }
}
/// How a range's end values give their positions.
pub(super) enum Ends {
    Ord(DomainFn),
    Dom,
}
