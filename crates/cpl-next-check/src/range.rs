//! Ranges and domains as values (R01.8.4, R04.2).
use super::call::{ArgValue, CallArg};
use super::expr::block;
use super::member::Called;
use super::*;
use cpl_next_ir::ops::{DomainOp, Intrinsic};
use cpl_next_ir::unbuilt::Unbuilt;

impl Cx<'_> {
    /// A fn of a domain named as a type (`D.nth(k)`), or of an array type written `[e]`, as
    /// in `[V].filled(0)`.
    pub(super) fn domain_member(
        &mut self,
        base: &next::Expr,
        name: Called,
        call: Option<&[CallArg]>,
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
            if let Some(t) = self.member_type(&dty, name.text) {
                return match call {
                    None => Some(self.domain_value(t, span)),
                    Some(
                        [CallArg {
                            label: None,
                            value: ArgValue::Written(v),
                            ..
                        }],
                    ) => Some(self.convert(t, v, span, Origin::Convert)),
                    Some(_) => None,
                };
            }
        }
        // `[e].f(args)`: a fn of the array type `[e]_` with no receiver, as in `[V].filled(0)`.
        let (next::Expr::List(l), Some(args)) = (base, call) else {
            return None;
        };
        let [index] = &l.elems().collect::<Vec<_>>()[..] else {
            return None;
        };
        let array = self.d.sugar().array?;
        let mut pre = Vec::new();
        let axis = self.axis_of(index, span, &mut pre)?;
        let m = self.d.member(&self.d.s(array).methods, name.text)?;
        let given = vec![axis, self.new_ty()];
        let f = self.fnref(m, given, Vec::new());
        let call = self.call_fn(f, None, args, span, want);
        Some(block(pre, call))
    }
    /// The axis `[e]` keys an array by: a domain, a constant range, or a domain of its own.
    fn axis_of(&mut self, e: &next::Expr, span: Span, pre: &mut Vec<TStmt>) -> Option<Ty> {
        if let Some(t) = self.type_path(e) {
            return Some(t);
        }
        if let next::Expr::Range(re) = e {
            if self.static_axis(re) {
                let scoped = |n| self.owned_named(n);
                let at = decls::BodyScope {
                    sigs: self.d,
                    scope: self.d,
                    m: self.b.module,
                    generics: &self.b.generics,
                    doms: &self.b.doms,
                    scoped: &scoped,
                };
                let (t, errs) = decls::range_type(&at, re);
                for e in errs {
                    self.err(span, e);
                }
                return Some(t);
            }
        }
        self.value_axis(e, span, pre)
    }
    /// `D.count`, `D.nth(k)`, `D.at(x)` and `D.all`: the core's `Domain` declares them; each
    /// `@intrinsic` one is the domain operation it names (R04.2.8, R04.2.9).
    pub(super) fn domain_fn(
        &mut self,
        dty: Ty,
        name: Called,
        call: Option<&[CallArg]>,
        span: Span,
    ) -> Option<TExpr> {
        let core = [self.d.roles().domain, self.d.roles().order];
        let m = (core.iter().flatten())
            .find_map(|tr| self.d.member(&self.d.t(*tr).methods, name.text))?;
        self.domain_fn_of(dty, m, name, call, span)
    }
    /// `D.m`, `m` being one of `Domain`'s or `Ordered`'s fns, which the program calls `name`.
    pub(super) fn domain_fn_of(
        &mut self,
        dty: Ty,
        m: FnId,
        name: Called,
        call: Option<&[CallArg]>,
        span: Span,
    ) -> Option<TExpr> {
        let core = [self.d.roles().domain, self.d.roles().order];
        let tr = (core.into_iter().flatten())
            .find(|tr| self.d.t(*tr).methods.values().any(|&f| f == m))?;
        let fd = self.d.f(m);
        if !super::member::takes(fd, call) || (call.is_none() && !fd.getter) {
            return None;
        }
        let Some(op) = fd.intrinsic else {
            let r = decls::TraitRef {
                tr,
                args: Vec::new(),
            };
            return Some(self.static_call(dty, r, m, name, call, span));
        };
        let args = match call {
            None => None,
            Some(
                [CallArg {
                    label: None,
                    value: ArgValue::Written(v),
                    ..
                }],
            ) => {
                let x = self.expr(v, Some(&Ty::Int));
                Some(self.coerce(x, &Ty::Int))
            }
            Some(_) => return None,
        };
        let Intrinsic::Domain(op) = op else {
            return None;
        };
        self.domain_op(dty, op, args, span)
    }
    /// The domain operation `op` of `D`, with its one `Int` argument if it takes one. The body
    /// of each of `Domain`'s fns that a domain meets without an impl.
    pub(super) fn domain_op(
        &mut self,
        dty: Ty,
        op: DomainOp,
        arg: Option<TExpr>,
        span: Span,
    ) -> Option<TExpr> {
        Some(match (op, arg) {
            (DomainOp::Count, None) => TExpr {
                kind: TK::DomLen(dty),
                ty: Ty::Int,
                span,
            },
            (DomainOp::Nth, Some(k)) => TExpr {
                kind: TK::DomNth(dty.clone(), Box::new(k)),
                ty: dty,
                span,
            },
            (DomainOp::At, Some(x)) => self.domain_at(dty, x, span),
            (DomainOp::Counted, None) => TExpr {
                kind: TK::DomCounted(dty),
                ty: Ty::Bool,
                span,
            },
            (DomainOp::Ord, Some(v)) => TExpr {
                kind: TK::Ordinal(Box::new(v)),
                ty: Ty::Int,
                span,
            },
            _ => return None,
        })
    }
    /// Whether `t` is a type parameter bound by `Domain`.
    fn domain_param(&self, t: &Ty) -> bool {
        let Ty::Param(i) = t else {
            return false;
        };
        (self.bound_traits(*i).iter()).any(|r| Some(r.tr) == self.d.roles().domain)
    }
    /// `[D] k => e`: one cell per position of `D` in its walk order, which is the list
    /// `[e for k in D]` where a `[D]T` is wanted (R02.8.2).
    pub(super) fn table(&mut self, t: &next::TableExpr, span: Span, want: Option<&Ty>) -> TExpr {
        let mut pre = Vec::new();
        let dims: Vec<next::GenericArg> = t.dims().map(|d| d.args().collect()).unwrap_or_default();
        let mut parts = Vec::new();
        for d in &dims {
            let next::GenericArg::Type(ix) = d else {
                return self.refuse(span, Unbuilt::TableOverIndex);
            };
            let ty = match self.array_axis(ix, &mut pre) {
                Some(ty) => ty,
                None => self.body_ty(ix, &mut pre),
            };
            parts.push(ty);
        }
        // Several domains are their product, the patterns naming its parts (R04.2.11).
        let dty = match &parts[..] {
            [d] => d.clone(),
            ds if ds.len() > 1 => match self.d.tuple(ds.len()) {
                Some(tu) => Ty::Struct(tu, parts.clone(), Vec::new()),
                None => return self.refuse(span, Unbuilt::TableOverIndex),
            },
            _ => return self.refuse(span, Unbuilt::TableOverIndex),
        };
        let Some(array) = self.d.sugar().array else {
            return self.refuse(span, Unbuilt::TableOverIndex);
        };
        let sets = (0..self.d.n_sets(array)).map(|_| self.new_set()).collect();
        let pats: Vec<next::Pattern> = (t
            .params()
            .map(|p| p.lambda_params())
            .unwrap_or_default()
            .into_iter())
        .map(|p| p.pattern)
        .collect();
        let Some(body) = t.body() else {
            return self.refuse(span, Unbuilt::TableOverIndex);
        };
        let int = |v: i64| TExpr {
            kind: TK::Int(v),
            ty: Ty::Int,
            span,
        };
        let written_range = match &dims[..] {
            [next::GenericArg::Type(next::Type::Range(r))] => match r.expr() {
                Some(next::Expr::Range(re)) => Some(re),
                _ => None,
            },
            _ => None,
        };
        let walk = match &dty {
            // An anonymous range's cells take any `Int` (R04.3.4), so its keys walk as `Int`s.
            Ty::Dom(i) if self.domains[*i].name.starts_with('$') => {
                let (lo, len) = (self.domains[*i].lo, self.domains[*i].len);
                let var = |r| TExpr {
                    kind: TK::Var(r),
                    ty: Ty::Int,
                    span,
                };
                let hi = TExpr {
                    kind: TK::Bin(BinOp::Add, Box::new(var(lo)), Box::new(var(len))),
                    ty: Ty::Int,
                    span,
                };
                self.range_of(vec![var(lo), hi, int(1)], Ty::Int, span)
            }
            // And one ending at a domain's count: `[0..V.count - 1] i => ..` walks its written ends.
            Ty::RangeTo(..) => match written_range {
                Some(re) => {
                    let (lo, hi) = (re.lo(), re.hi());
                    let inclusive = re.is_inclusive();
                    self.range_value(lo.as_ref(), hi.as_ref(), inclusive, None, None, span)
                }
                None => return self.refuse(span, Unbuilt::TableOverIndex),
            },
            // So do a constant range's: `[0..3] i => ..`.
            Ty::Range(lo, hi) => self.range_of(vec![int(*lo), int(*hi + 1), int(1)], Ty::Int, span),
            // A domain, a type that implements `Domain` (`[V.cuts] c => ..`), or a parameter
            // bound by it, walks its positions.
            Ty::Dom(_) => self.domain_value(dty.clone(), span),
            Ty::Struct(..) | Ty::Param(_) if self.is_domain(&dty) || self.domain_param(&dty) => {
                self.domain_value(dty.clone(), span)
            }
            _ => return self.refuse(span, Unbuilt::TableOverIndex),
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
        let list = self.fill_over(&pats, walk, &body, want_list.as_ref(), span);
        let elem = match self.shallow(&list.ty) {
            Ty::Struct(_, ts, _) if ts.len() == 1 => ts[0].clone(),
            _ => Ty::Error,
        };
        // Into an array over another anonymous range the cells convert as a list does, their
        // count checked, since such an array takes any `Int` (R04.3.4).
        let dty = match want.map(|w| self.shallow(w)) {
            Some(Ty::Struct(a, ts, _)) if a == array && self.converts(&dty, &ts[0]) => {
                ts[0].clone()
            }
            _ => dty,
        };
        let cells = self.coerce(list, &Ty::Struct(array, vec![dty, elem], sets));
        block(pre, cells)
    }
    /// Whether an array over the anonymous range `from` converts to one over `to`: both are
    /// anonymous, and two written counts that differ stay a type mismatch (R04.8.10).
    fn converts(&self, from: &Ty, to: &Ty) -> bool {
        let count = |t: &Ty| match self.shallow(t) {
            Ty::Dom(i) if self.domains[i].name.starts_with('$') => Some(self.domains[i].count),
            Ty::Range(lo, hi) => Some(Some(hi - lo + 1)),
            _ => None,
        };
        match (count(from), count(to)) {
            (Some(f), Some(t)) => !matches!((f, t), (Some(x), Some(y)) if x != y),
            _ => false,
        }
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
    pub(super) fn type_path(&self, e: &next::Expr) -> Option<Ty> {
        match e {
            next::Expr::Name(n) => {
                let n = n.ident()?;
                match self.lookup(n.text()) {
                    None => self.domain_named(n.text()),
                    Some(_) => None,
                }
            }
            next::Expr::Paren(p) => self.type_path(&p.inner()?),
            next::Expr::Field(f) => {
                let b = self.type_path(&f.base()?)?;
                self.member_type(&b, f.name()?.text())
            }
            // An inline tuple of domains is the product domain (R04.2.11).
            next::Expr::Tuple(t) => {
                let xs: Vec<next::Expr> = t.elems().collect();
                if xs.len() < 2 {
                    return None;
                }
                let tuple = self.d.tuple(xs.len())?;
                let parts = (xs.iter())
                    .map(|x| self.type_path(x))
                    .collect::<Option<Vec<_>>>()?;
                Some(Ty::Struct(tuple, parts, Vec::new())).filter(|t| self.is_domain(t))
            }
            // So is a record of them, its factors named (R04.2.10).
            next::Expr::Record(r) => {
                let fs: Vec<next::RecordField> = r.fields().collect();
                let names = (fs.iter())
                    .map(|f| f.name().map(|n| n.text().to_string()))
                    .collect::<Option<Vec<String>>>()?;
                let record = self.d.record(&names)?;
                let parts = (fs.iter().zip(&names))
                    .map(|(f, n)| match f.value() {
                        Some(x) => self.type_path(&x),
                        None => self.domain_named(n),
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(Ty::Struct(record, parts, Vec::new())).filter(|t| self.is_domain(t))
            }
            _ => None,
        }
    }
    /// `Subset<V>` written as a value: the struct type at the written arguments.
    pub(super) fn written_struct(
        &mut self,
        e: &next::Expr,
        span: Span,
        pre: &mut Vec<TStmt>,
    ) -> Option<Ty> {
        let next::Expr::Generic(g) = e else {
            return None;
        };
        self.struct_at(&g.base()?, g.args(), span, pre)
    }
    /// The struct `base` names at the type arguments `args`, as decls gives it when written.
    pub(super) fn struct_at(
        &mut self,
        base: &next::Expr,
        args: Option<next::GenericArgs>,
        _span: Span,
        pre: &mut Vec<TStmt>,
    ) -> Option<Ty> {
        let next::Expr::Name(n) = base else {
            return None;
        };
        let n = n.ident()?;
        if self.lookup(n.text()).is_some() {
            return None;
        }
        let d = self.d.struct_named(self.b.module, n.text())?;
        let written: Vec<next::GenericArg> = args.map(|g| g.args().collect()).unwrap_or_default();
        let mut a = Vec::new();
        for t in written {
            let next::GenericArg::Type(t) = t else {
                return None;
            };
            a.push(self.body_ty(&t, pre));
        }
        let (d, a) = decls::struct_args(self.d, d, a);
        let sets = (0..self.d.n_sets(d)).map(|_| self.new_set()).collect();
        Some(Ty::Struct(d, a, sets))
    }
    /// A struct type whose constructor takes `n` fields.
    pub(super) fn built_from(&self, t: &Ty, n: usize) -> bool {
        matches!(t, Ty::Struct(s, ..) if self.d.s(*s).fields.len() == n)
    }
    /// `D.m`: the `@member(m)` struct at `D` (R06).
    fn member_type(&self, d: &Ty, m: &str) -> Option<Ty> {
        let &id = self.d.member_types.get(&self.d.sym(m)?)?;
        (self.d.s(id).n_sets == 0).then(|| Ty::Struct(id, vec![d.clone()], Vec::new()))
    }
    /// `D(x)`: the position of `x`, checked; a library domain (`Cell((0, 2))` of a product)
    /// converts by its `Into` impl.
    pub(super) fn domain_from(&mut self, dty: Ty, x: &next::Expr, span: Span) -> TExpr {
        if let Ty::Struct(..) = dty {
            return self.convert(dty, x, span, Origin::Convert);
        }
        let v = match dty {
            Ty::Dom(i) if self.domains[i].chars => {
                let c = self.expr(x, Some(&Ty::Char));
                let c = self.coerce(c, &Ty::Char);
                TExpr {
                    kind: TK::CodeOf(Box::new(c)),
                    ty: Ty::Int,
                    span,
                }
            }
            _ => {
                let v = self.expr(x, Some(&Ty::Int));
                self.coerce(v, &Ty::Int)
            }
        };
        TExpr {
            kind: TK::DomFrom(dty.clone(), Box::new(v), CheckKind::Domain),
            ty: dty,
            span,
        }
    }
    /// A name that is a domain: one declared in a body, or a type parameter used as one.
    pub(super) fn domain_named(&self, n: &str) -> Option<Ty> {
        let sym = self.sym_of(n);
        // A fn's own type parameter is nearer than any domain declared outside it.
        if let Some(p) = sym.and_then(|s| self.b.generics.iter().position(|g| *g == s)) {
            return Some(Ty::Param(p));
        }
        if let Some(t @ Ty::Struct(..)) = sym.and_then(|s| self.b.doms.get(&s)) {
            return Some(t.clone());
        }
        // `type Cell = (Row, Col)` names the product domain too (R04.2.10).
        if let Some(decls::Named::Alias(id)) = self.d.named(self.b.module, n) {
            let scoped = |n| self.owned_named(n);
            let at = decls::BodyScope {
                sigs: self.d,
                scope: self.d,
                m: self.b.module,
                generics: &self.b.generics,
                doms: &self.b.doms,
                scoped: &scoped,
            };
            let (t, _) = decls::alias_type(&at, id, Vec::new());
            return Some(t).filter(|t| self.is_domain(t));
        }
        // So does a struct that is a domain by its impl: a payload-free enum (R04.2.22).
        if let Some(decls::Named::Struct(s)) = self.d.named(self.b.module, n) {
            let t = Ty::Struct(s, Vec::new(), Vec::new());
            if self.d.s(s).generics.is_empty() && self.is_domain(&t) {
                return Some(t);
            }
        }
        if let Some(&Ty::Dom(i)) = sym.and_then(|s| self.b.doms.get(&s)) {
            if self.b.decl.is_none() && self.domains[i].name.is_empty() {
                self.early.set(Some(i));
            }
            return Some(Ty::Dom(i));
        }
        None
    }
    /// `lo..hi`, `lo..=hi` and `lo..hi by k` as a value: core's `Range` (R01.8.4).
    pub(super) fn range_value(
        &mut self,
        lo: Option<&next::Expr>,
        hi: Option<&next::Expr>,
        inclusive: bool,
        step: Option<&next::Expr>,
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let diagnosed = self.errors.len();
        let (args, d) = match (lo, hi) {
            (Some(lo), Some(hi)) => {
                let first = self.expr(lo, Some(&Ty::Int));
                let first = match self.whole_of(want) {
                    Some(d @ Ty::Dom(_))
                        if step.is_none() && self.shallow(&first.ty) == Ty::Int =>
                    {
                        self.coerce(first, &d)
                    }
                    // A range type's positions are its `Int`s: `u[1..3]` on `[0..4]T`.
                    Some(d @ Ty::Range(..))
                        if step.is_none() && self.shallow(&first.ty) == Ty::Int =>
                    {
                        let first = self.coerce(first, &d);
                        let (args, d) =
                            self.domain_range(first, hi, inclusive, want, Ends::Dom, span);
                        return self.range_built(args, d, diagnosed, span);
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
        hi: &next::Expr,
        inclusive: bool,
        step: Option<&next::Expr>,
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let diagnosed = self.errors.len();
        let (args, d) = match self.ord_ends(&first.ty, step) {
            Some(ord) => self.domain_range(first, hi, inclusive, want, ord, span),
            None => match self.endpoint_struct(&first.ty) {
                Some(Ty::Struct(d, targs, sets)) => {
                    let ends = (first, hi);
                    return self.endpoint_range(ends, inclusive, step, (d, targs, sets), span);
                }
                _ => (self.int_ends(first, hi, inclusive, step, span), Ty::Int),
            },
        };
        self.range_built(args, d, diagnosed, span)
    }
    /// The struct `lo..hi` builds from ends of type `t`: the associated type of `t`'s
    /// `@role(endpoint)` impl (decision 456). `None` for `Int` ends and a domain's positions,
    /// which take `range_literal`.
    fn endpoint_struct(&self, t: &Ty) -> Option<Ty> {
        let tr = self.d.roles().endpoint?;
        let t = self.shallow(t);
        let int_like = matches!(t, Ty::Int | Ty::Range(..) | Ty::Dom(_));
        if int_like || matches!(t, Ty::Error | Ty::Var(_) | Ty::Param(_)) {
            return None;
        }
        let impls = self.impls();
        let (im, targs) = (impls.ranked_impls(tr))
            .find_map(|(_, im)| impls.fits_impl(im, &t).map(|targs| (im, targs)))?;
        let sub = Subst {
            tys: &targs,
            sets: &[],
        };
        Some(sub.ty(im.trait_.args.last()?))
    }
    /// `lo..hi` over an endpoint whose range has no positions: its struct from the two ends. Such
    /// a range has no excluded end to include and nothing to step, so `..=` and `by` are E0302.
    fn endpoint_range(
        &mut self,
        (lo, hi): (TExpr, &next::Expr),
        inclusive: bool,
        step: Option<&next::Expr>,
        (d, targs, sets): (StructId, Vec<Ty>, Vec<SetTy>),
        span: Span,
    ) -> TExpr {
        let t = lo.ty.clone();
        let hi = self.expr(hi, Some(&t));
        let hi = self.coerce(hi, &t);
        if inclusive || step.is_some() {
            let shown = self.show(&t);
            let what = if inclusive { "`..=`" } else { "`by`" };
            let code = cpl_diag::codes::TYPE_MISMATCH;
            let msg =
                format!("a range of `{shown}` ends has no positions, so it cannot take {what}");
            self.err(span, Msg::verdict(code, msg));
            return Self::error_node(span);
        }
        let args = [CallArg::held(None, lo, span), CallArg::held(None, hi, span)];
        self.construct(d, targs, sets, &args, span, None)
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
        lo: Option<&next::Expr>,
        hi: Option<&next::Expr>,
        inclusive: bool,
        step: Option<&next::Expr>,
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
        // An `Int` end where positions are wanted is a position, as `lo..hi`'s start is; an
        // exclusive literal end is a cut, so it may be the domain's end (R04.4.25).
        let whole = self.whole_of(want);
        if let Some(d @ Ty::Dom(_)) = &whole {
            if !from && !inclusive && step.is_none() && matches!(v.kind, TK::Int(_)) {
                let int = |kind| TExpr {
                    kind,
                    ty: Ty::Int,
                    span,
                };
                let hi = self.cut_ordinal(d.clone(), v);
                return Ok((vec![int(TK::Int(0)), hi, int(TK::Int(1))], d.clone()));
            }
        }
        let v = match whole {
            Some(d @ Ty::Dom(_)) if step.is_none() && self.shallow(&v.ty) == Ty::Int => {
                self.coerce(v, &d)
            }
            _ => v,
        };
        let t = self.shallow(&v.ty);
        let int = |kind| TExpr {
            kind,
            ty: Ty::Int,
            span,
        };
        let past = |e: TExpr| {
            let one = Box::new(int(TK::Int(1)));
            int(TK::Bin(BinOp::Add, Box::new(e), one))
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
            let at = self.at(given);
            self.err(at, Msg::verdict(code, msg));
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
    pub(super) fn ord_ends(&self, t: &Ty, step: Option<&next::Expr>) -> Option<Ends> {
        let tr = self.d.roles().domain?;
        if step.is_some() {
            return None;
        }
        match self.shallow(t) {
            Ty::Dom(_) => Some(Ends::Dom),
            t @ Ty::Struct(..) if self.impls().has_impl(&t, tr) => {
                self.domain_method(DomainOp::Ord).map(Ends::Ord)
            }
            Ty::Param(p) if self.bound_traits(p).iter().any(|b| b.tr == tr) => {
                self.domain_method(DomainOp::Ord).map(Ends::Ord)
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
        hi: &next::Expr,
        inclusive: bool,
        step: Option<&next::Expr>,
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
                kind: TK::Bin(BinOp::Add, Box::new(hi), Box::new(one)),
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
            match self.d.roles().endpoint {
                // Ends whose type has no `@role(endpoint)` impl make no range.
                Some(_) => {
                    let code = cpl_diag::codes::TYPE_MISMATCH;
                    let msg = format!("a range's ends cannot be `{t}`: it has no `Endpoint` impl");
                    self.err(v.span, Msg::verdict(code, msg));
                }
                None => self.err(v.span, Unbuilt::RangeOfEnds { ty: t }),
            }
            return Self::error_node(v.span);
        }
        self.coerce(v, &Ty::Int)
    }
    /// Std's `@role(range_literal)` fn `(lo, hi, step)` over positions of `d`.
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
        let pos = mk(
            TK::DomFrom(d.clone(), Box::new(read()), CheckKind::Domain),
            d.clone(),
        );
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
        match (&d, self.domain_method(DomainOp::Count)) {
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
        hi: &next::Expr,
        inclusive: bool,
        want: Option<&Ty>,
        ord: Ends,
        span: Span,
    ) -> (Vec<TExpr>, Ty) {
        let t = self.shallow(&lo.ty);
        let hi = self.expr(hi, Some(&t));
        // An exclusive literal end is a cut, so it may be the domain's end (R04.4.25).
        let cut =
            !inclusive && matches!(t, Ty::Dom(_) | Ty::Range(..)) && matches!(hi.kind, TK::Int(_));
        let hi = match cut {
            true => self.cut_ordinal(t.clone(), hi),
            false => self.coerce(hi, &t),
        };
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
        let mut hi = match cut {
            true => hi,
            false => self.end_position(hi, &ord),
        };
        let int = |kind| TExpr {
            kind,
            ty: Ty::Int,
            span,
        };
        if inclusive {
            hi = int(TK::Bin(BinOp::Add, Box::new(hi), Box::new(int(TK::Int(1)))));
        }
        (vec![lo, hi, int(TK::Int(1))], piece.unwrap_or(t))
    }
    /// `D`, when `t` is the `@role(cuts)` struct of `D`.
    pub(super) fn cut_domain(&self, t: &Ty) -> Option<Ty> {
        match self.shallow(t) {
            Ty::Struct(c, of, _) if Some(c) == self.d.roles().cut => {
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
        let ord = self.domain_method(DomainOp::Ord)?;
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
    fn domain_method(&self, op: DomainOp) -> Option<DomainFn> {
        let tr = self.d.roles().domain?;
        let ms = &self.d.t(tr).methods;
        let (n, &m) = ms
            .iter()
            .find(|(_, &m)| self.d.f(m).intrinsic == Some(Intrinsic::Domain(op)))?;
        let r = decls::TraitRef {
            tr,
            args: Vec::new(),
        };
        Some((r, self.d.text(*n).to_string(), m))
    }
    /// `v.ord` of a value whose type implements `Domain`, through the trait.
    fn ordinal(&mut self, v: TExpr, ord: &DomainFn) -> TExpr {
        let span = v.span;
        let (r, method, m) = ord.clone();
        self.reach(m);
        self.b
            .obligations
            .push((v.ty.clone(), r.clone(), span, Origin::Bound));
        TExpr {
            kind: TK::CallTrait {
                ctx: self.trait_ctx(r.tr, &method),
                tr: r,
                method,
                self_ty: Box::new(v.ty.clone()),
                targs: Box::default(),
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
        self.b
            .obligations
            .push((d.clone(), r.clone(), span, Origin::Bound));
        let name = Called { text: &n, span };
        self.static_call(d, r, m, name, None, span)
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
                (self.bound_traits(i).iter()).any(|b| Some(b.tr) == self.d.roles().domain)
            }
            _ => false,
        }
    }
    /// A position's integer. On a type parameter that is its `Domain` impl's `dom_ord` member,
    /// reached here so the impl an instance runs is checked (R04.2.15).
    pub(super) fn position_int(&mut self, e: TExpr) -> TExpr {
        if matches!(self.shallow(&e.ty), Ty::Param(_)) {
            let d = self.d;
            let ord = (d.roles().domain.iter())
                .flat_map(|tr| d.t(*tr).methods.values())
                .find(|&&m| d.f(m).intrinsic == Some(Intrinsic::Domain(DomainOp::Ord)));
            if let Some(&m) = ord {
                self.reach(m);
            }
        }
        TExpr {
            span: e.span,
            kind: TK::DomOrd(Box::new(e)),
            ty: Ty::Int,
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
        let same = rp && self.shallow(&l.ty) == self.shallow(&r.ty);
        match op {
            BinOp::Add | BinOp::Sub if lp && (!rp || op == BinOp::Sub && same) => {
                let r = if rp { self.position_int(r) } else { r };
                let l = self.position_int(l);
                Ok(self.binary(op, l, r, span))
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
