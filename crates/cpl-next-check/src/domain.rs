//! Domain declarations: their ends, factors and named ranges.

use super::*;
use cpl_next_decls::decls::Role;
use cpl_next_syntax::SyntaxToken;
use std::collections::HashMap;

impl Cx<'_> {
    /// The start of a domain `lo..hi` or `lo..=hi`, the integer after its last position, and
    /// what the types of the two ends say every position lies within.
    pub(super) fn domain_ends(
        &mut self,
        lo: &next::Expr,
        hi: &next::Expr,
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
        def: &next::Expr,
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> bool {
        let def_at = self.at(def);
        let (diagnosed, owed) = (self.errors.len(), self.b.obligations.len());
        let v = self.expr_as_typed(def, None);
        let t = self.shallow(&v.ty);
        // A form this checker cannot build yet (`read(1..=100)`, `V + K`) leaves no trace.
        // An owed bound no impl meets (`Range<V>: Add`) is a form not built yet too.
        let unmet = (self.b.obligations[owed..].iter()).any(|(t, r, ..)| {
            let t = self.shallow(t);
            !matches!(t, Ty::Var(_))
                && !(self.impls().impls_of(r.tr)).any(|im| self.impls().fits_impl(im, &t).is_some())
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
            span: def_at,
        };
        let count = match (&t, self.rows.role_trait(Role::Counted)) {
            (t, _) if *t == Ty::Int || t.is_int_range() => Some(v),
            (_, Some(tr)) if self.has_impl(&t, tr) => self.count_of(tr, v, out),
            _ => None,
        };
        let Some(count) = count else {
            let what = format!(
                "a domain is a count, a range of `Int`s, a sequence's positions or another \
                 domain, not `{}`",
                self.show(&t)
            );
            self.err(def_at, Msg::verdict(cpl_diag::codes::TYPE_MISMATCH, what));
            let sym = self.b.syms.intern(self.syms, name);
            // A top-level domain decls set aside is declared here, though it has no positions.
            if let Some(&Ty::Dom(i)) = self.b.doms.get(&sym) {
                self.domains[i].name = name.to_string();
            }
            self.b.doms.insert(sym, Ty::Error);
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
        let &(n, m) = self.methods.trait_fns(tr).first()?;
        let method = self.items.text(n);
        // A variable is read where it is; any other value is held first, so it runs once.
        let recv = match xs.kind {
            TK::Var(_) => xs,
            _ => {
                let v = self.temp(&format!("$counted{}", self.domains.len()), ty.clone());
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
        (self.b.obligations).push((ty.clone(), r.clone(), self.b.item.at(span), Origin::Bound));
        let kind = TK::CallTrait {
            ctx: self.trait_ctx(r.tr, method),
            tr: r,
            method: method.to_string(),
            recv: Some(Box::new(TArg::Value(recv))),
            self_ty: Box::new(ty.clone()),
            targs: Box::new([]),
            sets: Box::default(),
            args: vec![],
        };
        let ty = self.trait_ret(m, vec![ty]);
        Some(TExpr { kind, ty, span })
    }
    pub(super) fn typed_ends(
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
        let sym = self.b.syms.intern(self.syms, name);
        let i = match (self.b.decl, self.doms.top_dom(sym)) {
            (None, Some(&Ty::Dom(i))) if self.domains[i].name.is_empty() => {
                self.domains[i] = info;
                i
            }
            _ => self.add_domain(info, span),
        };
        self.b.doms.insert(sym, Ty::Dom(i));
        self.b.block_doms.push(i);
        i
    }
    /// An array axis's or a list's range: a domain no declaration names, so no name finds it.
    pub(super) fn declare_anonymous(
        &mut self,
        lo: TExpr,
        end: TExpr,
        within: (Option<i64>, Option<i64>),
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> usize {
        let shown = format!("$range{}", self.domains.len());
        let mut info = self.domain_info(&shown, lo, end, within, span, out);
        info.anonymous = true;
        let i = self.add_domain(info, span);
        self.b.block_doms.push(i);
        i
    }
    /// Adds domain `info` with the impls every domain has (`domain_rows`), so a body's domain,
    /// a range's and a binding's own fresh one are domains through the same rows.
    pub(super) fn add_domain(&mut self, info: DomInfo, span: Span) -> usize {
        let i = self.id(super::Table::Domain, self.domains.len());
        super::put(&mut self.domains, i, info);
        // An id names one domain however often its body is checked, as `put` keeps one entry.
        if !self.dom_rows.iter().any(|r| r.for_ == Ty::Dom(i)) {
            let rows = decls::domain_rows(self.rows, i, self.b.item.at(span), self.b.module);
            self.dom_rows.extend(rows);
        }
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
            anonymous: false,
        }
    }
    /// `domain N = e` and `let (domain N, i) = e`, at the top level or in a block: `N`'s
    /// positions, declared in `out`.
    pub(super) fn domain_stmt(&mut self, d: &next::DomainDecl, out: &mut Vec<TStmt>) {
        let span = self.at(d);
        let Some(name) = d.name() else {
            return;
        };
        let name = name.text().to_string();
        let pats: Vec<next::Pattern> = d.patterns().collect();
        let Some(def) = d.size() else {
            return self.refuse_domain(&name, span, "a domain with no size".to_string());
        };
        if !pats.is_empty() {
            return self.domain_let(&name, &def, &pats, span, out);
        }
        match &def {
            next::Expr::Tuple(_) | next::Expr::Record(_) => {
                let Some(p) = self.declare_factors(&name, &def, span, out) else {
                    let at = self.at(&def);
                    let msg = "a product domain's parts are domains";
                    return self.err(at, Msg::verdict(cpl_diag::codes::TYPE_MISMATCH, msg));
                };
                let sym = self.b.syms.intern(self.syms, &name);
                self.b.doms.insert(sym, p);
            }
            // `domain V = [n]` counts `0..n` (R04.2.29).
            next::Expr::List(l) => match <[_; 1]>::try_from(l.elems().collect::<Vec<_>>()) {
                Ok([count]) => {
                    let hi = self.expr_as_typed(&count, None);
                    let lo = TExpr {
                        kind: TK::Int(0),
                        ty: Ty::Int,
                        span: hi.span,
                    };
                    let (lo, end, within) = self.typed_ends(lo, hi, false);
                    let i = self.declare_domain(&name, lo, end, within, span, out);
                    self.domains[i].counted = true;
                }
                Err(_) => self.other_domain(&name, &def, span, out),
            },
            next::Expr::Range(r) if r.step().is_none() && r.walk().is_none() => {
                match (r.lo(), r.hi()) {
                    (Some(lo), Some(hi)) => {
                        let (lo, end, within, chars) = self.domain_ends(&lo, &hi, r.is_inclusive());
                        let i = self.declare_domain(&name, lo, end, within, span, out);
                        self.domains[i].chars = chars;
                    }
                    _ => self.other_domain(&name, &def, span, out),
                }
            }
            _ => self.other_domain(&name, &def, span, out),
        }
    }
    /// `domain V = e` for an `e` that is not a range or a product.
    fn other_domain(&mut self, name: &str, def: &next::Expr, span: Span, out: &mut Vec<TStmt>) {
        if !self.counted_domain(name, def, span, out) {
            self.refuse_domain(name, span, crate::kind_name(def));
        }
    }
    /// A domain form not built yet: `N` names nothing a later use can trip on.
    fn refuse_domain(&mut self, name: &str, span: Span, kind: String) {
        self.err(span, Unbuilt::DomainKind { kind });
        let sym = self.b.syms.intern(self.syms, name);
        self.b.doms.insert(sym, Ty::Error);
        self.b.refused_names.insert(name.to_string());
    }
    /// `let (domain N, i) = e`: `e` is held whole, `N` names its fresh domain afresh, and each
    /// other pattern binds the field in its position (R02.5.5).
    fn domain_let(
        &mut self,
        name: &str,
        def: &next::Expr,
        pats: &[next::Pattern],
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        let v = self.expr(def, None);
        let ty = self.zonk(&v.ty);
        let Some((d, fields)) = self.field_tys(&ty) else {
            self.refuse_domain(name, span, "an untyped domain binding".to_string());
            for p in pats {
                self.bind_unknown(p, false, out);
            }
            return;
        };
        let whole = self.temp(&format!("$tuple{}", span.start), ty.clone());
        out.push(TStmt::Let(whole, v));
        let read = |i: usize, ty: Ty| TExpr {
            kind: TK::Field(
                Box::new(TExpr {
                    kind: TK::Var(whole),
                    ty: ty.clone(),
                    span,
                }),
                i,
            ),
            ty,
            span,
        };
        let first = read(0, fields.first().cloned().unwrap_or(Ty::Error));
        let fresh = self.b.syms.intern(self.syms, name);
        match self.bind_fresh(fresh, &first.ty, span, out) {
            Some((c, i)) => {
                let t = crate::pattern::retag(&ty, c, i);
                let (VarRef::Local(k) | VarRef::Global(k)) = whole;
                match self.b.decl.is_none() {
                    true => self.globals[k].ty = t,
                    false => self.b.locals[k].ty = t,
                }
            }
            None => self.err(span, Unbuilt::DomainPattern),
        }
        let names = match self.items.s(d).record {
            true => self.field_names(d),
            false => Vec::new(),
        };
        let whole_name = self.var_name(whole).to_string();
        for (k, p) in pats.iter().enumerate().map(|(k, p)| (k + 1, p)) {
            let field = names.get(k).cloned().unwrap_or_else(|| k.to_string());
            let part = read(k, fields.get(k).cloned().unwrap_or(Ty::Error));
            self.let_pattern(p, false, part, out);
            // `tree.Pre` is the domain the whole value owns as its `tree.Pre`.
            if let Some(n) = pattern_name(p) {
                let (w, f, n) = (
                    self.b.syms.intern(self.syms, &whole_name),
                    self.b.syms.intern(self.syms, &field),
                    self.b.syms.intern(self.syms, &n),
                );
                self.hand_down(w, f, n);
            }
        }
    }
    /// A product's range factor is a domain of its own, declared here; the product is the
    /// tuple or record of its factors (R04.2.10, R04.2.11).
    fn declare_factors(
        &mut self,
        product: &str,
        def: &next::Expr,
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> Option<Ty> {
        // A punned field `x:` names the domain `x`.
        let (names, xs): (Option<Vec<String>>, Vec<Result<next::Expr, String>>) = match def {
            next::Expr::Tuple(t) => (None, t.elems().map(Ok).collect()),
            next::Expr::Record(r) => {
                let fs: Vec<next::RecordField> = r.fields().collect();
                let names: Option<Vec<String>> = (fs.iter())
                    .map(|f| f.name().map(|n| n.text().to_string()))
                    .collect();
                let names = names?;
                let xs = (fs.iter().zip(&names))
                    .map(|(f, n)| f.value().ok_or_else(|| n.clone()))
                    .collect();
                (Some(names), xs)
            }
            _ => return self.type_path(def),
        };
        let mut parts = Vec::new();
        for (i, x) in xs.iter().enumerate() {
            let x = match x {
                Ok(x) => x,
                Err(n) => {
                    parts.push(self.domain_named(n)?);
                    continue;
                }
            };
            let range = match x {
                next::Expr::Range(r) if r.step().is_none() && r.walk().is_none() => {
                    (r.lo().zip(r.hi())).map(|(lo, hi)| (lo, hi, r.is_inclusive()))
                }
                _ => None,
            };
            let Some((lo, hi, inclusive)) = range else {
                parts.push(self.type_path(x)?);
                continue;
            };
            let name = decls::factor_name(product, i);
            let (lo, end, within, chars) = self.domain_ends(&lo, &hi, inclusive);
            let d = self.declare_domain(&name, lo, end, within, span, out);
            self.domains[d].chars = chars;
            parts.push(Ty::Dom(d));
        }
        let t = match names {
            Some(names) => {
                let fields = names.iter().map(|n| self.syms.get(n));
                let record = self.shapes.record(&fields.collect::<Option<Vec<_>>>()?)?;
                Ty::Struct(record, parts, Vec::new())
            }
            None if parts.len() > 1 => {
                Ty::Struct(self.shapes.tuple(parts.len())?, parts, Vec::new())
            }
            None => return None,
        };
        Some(t).filter(|t| self.is_domain(t))
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
        let bounds = self.rows.role_trait(Role::RangeBounds);
        let fns = bounds
            .map(|tr| self.methods.trait_fns(tr))
            .unwrap_or_default();
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
                let n = cx.items.text(n);
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
                    sets: Box::default(),
                    args: vec![],
                };
                let r = decls::TraitRef {
                    tr,
                    args: args.clone(),
                };
                (cx.b.obligations).push((r_ty.clone(), r, cx.b.item.at(span), Origin::Bound));
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
    /// `[lo..hi]T`, `[n]T` and a range type with value ends: each such axis's domain or range,
    /// declared in `out` and keyed by the axis's item-relative span, for `decls::body_type` to
    /// read in place of the axis. A constant axis or one ending at a domain's count is decls'.
    pub(super) fn name_ranges(
        &mut self,
        t: &next::Type,
        out: &mut Vec<TStmt>,
    ) -> HashMap<Span, Ty> {
        let mut axes = HashMap::new();
        self.axes_in(t, out, &mut axes);
        axes
    }
    fn axes_in(&mut self, t: &next::Type, out: &mut Vec<TStmt>, axes: &mut HashMap<Span, Ty>) {
        match t {
            next::Type::Array(a) => {
                if let Some(e) = a.elem() {
                    self.axes_in(&e, out, axes);
                }
                let dims: Vec<next::GenericArg> =
                    a.dims().map(|d| d.args().collect()).unwrap_or_default();
                for d in dims {
                    let next::GenericArg::Type(ix) = d else {
                        continue;
                    };
                    if let Some(ty) = self.array_axis(&ix, out) {
                        axes.insert(self.at(&ix), ty);
                    }
                }
            }
            next::Type::Path(p) => {
                let args: Vec<next::GenericArg> = p
                    .generic_args()
                    .map(|g| g.args().collect())
                    .unwrap_or_default();
                for a in args {
                    if let next::GenericArg::Type(a) = a {
                        self.axes_in(&a, out, axes);
                    }
                }
            }
            // The ends are values: they are worked out once, here, and the type is their range.
            next::Type::Range(r) => {
                if let Some(next::Expr::Range(re)) = r.expr() {
                    if !self.static_axis(&re) {
                        let ty = self.value_range(&re, out);
                        axes.insert(self.at(t), ty);
                    }
                }
            }
            _ => {}
        }
    }
    pub(super) fn static_axis(&self, re: &next::RangeExpr) -> bool {
        decls::static_axis(
            self.tables,
            self.names,
            &self.b.syms.over(self.syms),
            self.b.module,
            &self.b.generics,
            &self.b.doms,
            re,
        )
    }
    /// An array axis over a range with a value end, or over a count `n`: a domain of its own.
    pub(super) fn array_axis(&mut self, ix: &next::Type, out: &mut Vec<TStmt>) -> Option<Ty> {
        let span = self.at(ix);
        match ix {
            next::Type::Range(r) => match r.expr()? {
                next::Expr::Range(re) if self.static_axis(&re) => None,
                e => self.value_axis(&e, span, out),
            },
            next::Type::Path(p) => {
                // A count `[n]`, or a value's member `[w.keys]`: the path read as a value.
                let segs: Vec<SyntaxToken> = p.segments().collect();
                let (n, members) = segs.split_first()?;
                let n = n.text();
                if p.generic_args().is_some()
                    || self.lookup(n).is_none()
                    || self.domain_named(n).is_some()
                {
                    return None;
                }
                let zero = TExpr {
                    kind: TK::Int(0),
                    ty: Ty::Int,
                    span,
                };
                let mut count = self.name(n, span);
                for m in members {
                    let (text, sym) = (m.text(), self.syms.get(m.text()));
                    let name = super::member::Called { text, sym, span };
                    count = self.member_on(count, name, None, span, None, &[]);
                }
                Some(self.axis_domain(zero, count, false, span, out))
            }
            _ => None,
        }
    }
    /// `[w.keys]`, `w` an `[K]T`: a table over exactly `w`'s keys is keyed by `K` itself and walks
    /// `w.keys` (R04b.4.2), so it needs no count of `K`. The key and that walk.
    pub(super) fn whole_keys(&mut self, ix: &next::Type) -> Option<(Ty, TExpr)> {
        let next::Type::Path(p) = ix else {
            return None;
        };
        let segs: Vec<SyntaxToken> = p.segments().collect();
        let [head, last] = &segs[..] else {
            return None;
        };
        if last.text() != "keys" || p.generic_args().is_some() {
            return None;
        }
        let span = self.at(ix);
        self.keys_of(head.text(), span)
    }
    /// The key `K` of local or `self` field `w` when it is an `[K]T` (a list's is `Int`,
    /// R04.8.11), and `w.keys`.
    pub(super) fn keys_of(&mut self, w: &str, span: Span) -> Option<(Ty, TExpr)> {
        let known = self.lookup(w).is_some() || self.self_field(w).is_some();
        if !known || self.domain_named(w).is_some() {
            return None;
        }
        let recv = self.name(w, span);
        let key = match self.shallow(&recv.ty) {
            Ty::Struct(a, ts, _) if Some(a) == self.array_id() && ts.len() == 2 => ts[0].clone(),
            Ty::Struct(l, ..) if Some(l) == self.role_structs.role_struct(Role::List) => Ty::Int,
            _ => return None,
        };
        let name = super::member::Called {
            text: "keys",
            sym: self.syms.get("keys"),
            span,
        };
        Some((key, self.member_on(recv, name, None, span, None, &[])))
    }
    /// An axis `[lo..hi]` with a value end, or a count `[n]`: a domain of its own.
    pub(super) fn value_axis(
        &mut self,
        e: &next::Expr,
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> Option<Ty> {
        let (lo, hi, inclusive) = match e {
            next::Expr::Range(re) => {
                let (lo, hi) = (re.lo()?, re.hi()?);
                // Its cells count from a number; one from a value is a domain (R04.3.4).
                if decls::literal_int(&lo).is_none() {
                    let msg = "an array over a range that starts at a value: name it as a \
                               domain, `domain D = lo..hi`, then `[D]T`";
                    self.err(span, Msg::verdict(cpl_diag::codes::TYPE_MISMATCH, msg));
                }
                let lo = self.expr_as_typed(&lo, None);
                (lo, self.expr_as_typed(&hi, None), re.is_inclusive())
            }
            e => {
                let zero = TExpr {
                    kind: TK::Int(0),
                    ty: Ty::Int,
                    span,
                };
                (zero, self.expr_as_typed(e, None), false)
            }
        };
        Some(self.axis_domain(lo, hi, inclusive, span, out))
    }
    fn axis_domain(
        &mut self,
        lo: TExpr,
        hi: TExpr,
        inclusive: bool,
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> Ty {
        let (lo, hi) = match (&lo.kind, inclusive) {
            (TK::Int(0), false) if self.int_range(&hi.ty) => self.key_run(hi, out),
            _ => (lo, hi),
        };
        let (lo, end, within) = self.typed_ends(lo, hi, inclusive);
        Ty::Dom(self.declare_anonymous(lo, end, within, span, out))
    }
    /// A range type `lo..hi` with value ends: each end held once in a slot, the type their range.
    fn value_range(&mut self, re: &next::RangeExpr, out: &mut Vec<TStmt>) -> Ty {
        let mut end = |cx: &mut Self, e: Option<next::Expr>, open: bool| {
            e.map(|e| {
                let b = cx.expr(&e, Some(&Ty::Int));
                let known = super::unify::known_range(&cx.b.let_ranges, &|v| cx.var_ty(v), &b);
                let b = cx.coerce(b, &Ty::Int);
                let span = b.span;
                let b = match open {
                    true => {
                        let one = TExpr {
                            kind: TK::Int(1),
                            ty: Ty::Int,
                            span,
                        };
                        TExpr {
                            kind: TK::Bin(BinOp::Sub, Box::new(b), Box::new(one)),
                            ty: Ty::Int,
                            span,
                        }
                    }
                    false => b,
                };
                let v = cx.slot(Ty::Int, "$end");
                let known = known.and_then(|(a, b)| match open {
                    true => Some((a.checked_sub(1)?, b.checked_sub(1)?)),
                    false => Some((a, b)),
                });
                if let Some(k) = known {
                    cx.b.let_ranges.insert(v, k);
                }
                out.push(TStmt::Let(v, b));
                v
            })
        };
        let lo = end(self, re.lo(), false);
        let hi = end(self, re.hi(), !re.is_inclusive());
        let i = self.id(super::Table::Range, self.ranges.len());
        let at = self.at(re);
        super::put(&mut self.ranges, i, RangeInfo { lo, hi, at });
        Ty::RangeOf(i)
    }
}

/// The name a pattern binds when it is a bare name.
fn pattern_name(p: &next::Pattern) -> Option<String> {
    let next::Pattern::Path(pp) = p else {
        return None;
    };
    let segs: Vec<SyntaxToken> = pp.segments().collect();
    match (&segs[..], pp.args()) {
        ([n], None) => Some(n.text().to_string()),
        _ => None,
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
