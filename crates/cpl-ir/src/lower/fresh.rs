//! Fns whose result carries a domain their body makes: each call gets its own.

use super::*;

impl<'a> Lowerer<'a> {
    /// The domains made at or after `dom_start` that `ty` mentions, each
    /// once, in the order it first mentions them.
    pub(super) fn fresh_domains_in(
        &self,
        ty: &Ty,
        dom_start: usize,
        seen_structs: &mut HashSet<StructId>,
        seen_enums: &mut HashSet<EnumId>,
        out: &mut Vec<DomainId>,
    ) {
        let note = |d: DomainId, out: &mut Vec<DomainId>| {
            if d >= dom_start && !out.contains(&d) {
                out.push(d);
            }
        };
        match ty {
            Ty::Pos(d) | Ty::Bound(d) => note(*d, out),
            Ty::Array { dims, elem } => {
                for dim in dims {
                    if let Dim::Pos(d) | Dim::Bound(d) | Dim::Sym(d) = dim {
                        note(*d, out);
                    }
                }
                self.fresh_domains_in(elem, dom_start, seen_structs, seen_enums, out);
            }
            Ty::Range(t)
            | Ty::Opt(t)
            | Ty::List(t)
            | Ty::ListView(t, _)
            | Ty::Keyed(t, _)
            | Ty::MutView(t)
            | Ty::Set(t)
            | Ty::HashSet(t)
            | Ty::Mut(t) => self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out),
            Ty::Map(k, v) | Ty::HashMap(k, v) => {
                self.fresh_domains_in(k, dom_start, seen_structs, seen_enums, out);
                self.fresh_domains_in(v, dom_start, seen_structs, seen_enums, out);
            }
            Ty::Tuple(ts) => {
                for t in ts {
                    self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out);
                }
            }
            Ty::Record(fs) => {
                for (_, t) in fs {
                    self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out);
                }
            }
            Ty::Fn(ps, r) => {
                for t in ps {
                    self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out);
                }
                self.fresh_domains_in(r, dom_start, seen_structs, seen_enums, out);
            }
            Ty::Struct(sid) => {
                if !seen_structs.insert(*sid) {
                    return;
                }
                let info = &self.structs[*sid];
                if let Some((_, gargs)) = &info.origin_gen {
                    for a in gargs {
                        match a {
                            GenArg::Domain(d) => note(*d, out),
                            GenArg::Ty(t) => {
                                self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out)
                            }
                            _ => {}
                        }
                    }
                }
                for (_, t) in &info.def.fields {
                    self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out);
                }
            }
            Ty::Enum(eid) => {
                if !seen_enums.insert(*eid) {
                    return;
                }
                for (_, fields) in &self.enums[*eid].variants {
                    for t in fields {
                        self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out);
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn clone_fresh_domain_fn(&mut self, orig: FnId) -> FnId {
        let decl = self.sigs[orig].decl;
        let mod_id = self.sigs[orig].mod_id;
        let offset = self.sigs[orig].offset;
        let owner = self.sigs[orig].owner;
        let env = self.sigs[orig].env.clone();
        let new_id = self.declare_fn_in(decl, mod_id, offset, owner, env);
        self.sigs[new_id].name = format!("{}_{new_id}", self.sigs[new_id].name);
        self.resolve_fn_sig(new_id, None);
        new_id
    }
}

impl<'a> Lowerer<'a> {
    /// The first key and the length of a `Range<Int>` value.
    pub(super) fn range_lo_len(&mut self, range: Expr, span: Span) -> (Expr, Expr) {
        let lo = self.mk(
            ExprKind::Builtin(Builtin::RangeLo, vec![range.clone()]),
            Ty::Int,
            span,
        );
        let hi = self.mk(
            ExprKind::Builtin(Builtin::RangeHi, vec![range]),
            Ty::Int,
            span,
        );
        let len = self.mk(
            ExprKind::Binary(BinOp::Sub, Box::new(hi), Box::new(lo.clone())),
            Ty::Int,
            span,
        );
        (lo, len)
    }

    /// The dimension a probed `[e]T` index names: a domain value's domain,
    /// or, for a `Range<Int>` value such as `w.keys`, a fresh domain over
    /// exactly its keys, indexed by them (R04b.1.5).
    pub(super) fn probed_dim(
        &mut self,
        probed: &Expr,
        e: &ast::Expr,
        values: bool,
    ) -> Option<DimSpec> {
        if let ExprKind::DomainVal(dom_id) = probed.kind {
            return Some(DimSpec::Pos(dom_id));
        }
        if probed.ty != Ty::Range(Box::new(Ty::Int)) {
            return None;
        }
        let id = self.domains.len();
        self.domains.push(format!("keys_{id}"));
        self.range_keyed.insert(id);
        if values {
            let val = self.expr(e, None);
            let (lo, len) = self.range_lo_len(val, e.span);
            self.cx.pending.push(Stmt::DefDomain { id, lo, len });
        }
        Some(DimSpec::Pos(id))
    }

    /// A table builder's walk along one dimension: the binder's type and the
    /// keys it runs over, `lo..hi`.
    pub(super) fn table_axis(&mut self, dim: &DimSpec, span: Span) -> (Ty, Expr, Expr) {
        let zero = self.int_lit(0, span);
        match dim {
            DimSpec::Pos(d) if self.range_keyed.contains(d) => {
                let lo = self.mk(ExprKind::DomainLo(*d), Ty::Int, span);
                let len = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                let hi = self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(lo.clone()), Box::new(len)),
                    Ty::Int,
                    span,
                );
                (Ty::Int, lo, hi)
            }
            DimSpec::Pos(d) => (
                Ty::Pos(*d),
                zero,
                self.mk(ExprKind::DomainLen(*d), Ty::Int, span),
            ),
            DimSpec::Bound(d) => {
                let len = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                (Ty::Bound(*d), zero, self.int_add(len, 1))
            }
            DimSpec::Sym(d) => {
                let n = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                let n1 = self.int_add(n.clone(), 1);
                let prod = self.mk(
                    ExprKind::Binary(BinOp::Mul, Box::new(n), Box::new(n1)),
                    Ty::Int,
                    span,
                );
                let two = self.int_lit(2, span);
                let len = self.mk(
                    ExprKind::Binary(BinOp::Div, Box::new(prod), Box::new(two)),
                    Ty::Int,
                    span,
                );
                (Ty::Int, zero, len)
            }
            DimSpec::Enum(e) => {
                let n = self.int_lit(self.enums[*e].variants.len() as i64, span);
                (Ty::Enum(*e), zero, n)
            }
            DimSpec::Int(len) => (Ty::Int, zero, (**len).clone()),
            // Over `lo..=hi` the builder's `i` runs from `lo`, and cell `i` is the `i - lo`th.
            DimSpec::IntFrom(from, len) => {
                let hi = self.int_add((**len).clone(), *from);
                (Ty::Int, self.int_lit(*from, span), hi)
            }
        }
    }

    /// The cell a table builder's binder `index`, walking `dim` from `lo`, fills.
    pub(super) fn table_cell(&mut self, dim: &DimSpec, index: Expr, lo: Expr) -> Expr {
        match dim {
            DimSpec::Enum(e) => {
                let ord = self.cast(Cast::EnumOrd(*e), index, Ty::Int);
                self.cast(Cast::Raw, ord, Ty::Int)
            }
            DimSpec::Pos(d) if self.range_keyed.contains(d) => {
                let span = index.span;
                let ord = self.mk(
                    ExprKind::Binary(BinOp::Sub, Box::new(index), Box::new(lo)),
                    Ty::Int,
                    span,
                );
                self.cast(Cast::Trust, ord, Ty::Pos(*d))
            }
            DimSpec::IntFrom(from, _) if *from != 0 => self.int_add(index, -from),
            _ => index,
        }
    }
}
