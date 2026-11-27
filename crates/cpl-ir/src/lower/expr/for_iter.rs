//! What a `for` walks over: ranges, domains, sequences, maps and
//! generators, and the type of each value.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// What a `for` walks over, and the type of each value.
    /// What `e` is where a sequence is wanted and `e` is something a
    /// `for` walks without being one: a range, a domain, `Bool`. It is
    /// the values the walk gives, `x for x in e`, so `sorted(0..n)`,
    /// `sum(lo..hi)` and `max(D)` need no rule of their own.
    pub(in crate::lower) fn walked(&mut self, e: &ast::Expr) -> Option<ast::Expr> {
        let mut inner = e;
        while let ast::ExprKind::Paren(next) = &inner.kind {
            inner = next;
        }
        let walks = if self.is_sorted_range_expr(inner) {
            true
        } else {
            match &inner.kind {
                ast::ExprKind::Range { .. } => true,
                ast::ExprKind::Name(name) if self.lookup(name).is_none() => {
                    name == "Bool"
                        || matches!(self.gen_env.get(name), Some(GenArg::Domain(_)))
                        || matches!(self.get_sym(name), Some(Sym::Domain(_)))
                        || self.index_struct(name).is_some()
                }
                ast::ExprKind::Generic { .. } => self.index_struct_of(inner).is_some(),
                ast::ExprKind::Field { base, name } => {
                    let (field_base, field_name, rev) = if name.text == "rev" {
                        match &base.kind {
                            ast::ExprKind::Field { base: b, name: n } => {
                                (b.as_ref(), n.text.as_str(), true)
                            }
                            _ => (base.as_ref(), "", false),
                        }
                    } else {
                        (base.as_ref(), name.text.as_str(), false)
                    };
                    if matches!(field_name, "keys" | "values") {
                        match self.probe_ty_silent(field_base) {
                            Some(Ty::Map(..)) => true,
                            Some(Ty::HashMap(..)) if !rev => true,
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            }
        };
        let item = "item";
        walks.then(|| ast::Expr {
            span: e.span,
            kind: ast::ExprKind::Comprehension {
                body: Box::new(ast::Expr {
                    kind: ast::ExprKind::Name(item.to_string()),
                    span: e.span,
                }),
                clauses: vec![ast::Clause {
                    pat: ast::Pattern {
                        kind: ast::PatternKind::Name(item.to_string()),
                        span: e.span,
                    },
                    iter: e.clone(),
                    conds: Vec::new(),
                }],
            },
        })
    }

    pub(in crate::lower) fn iter(&mut self, e: &ast::Expr) -> (Head, Ty) {
        let span = e.span;
        if let Some(alias) = self.subsets_alias(e) {
            return self.iter(&alias);
        }
        if let Some(res) = self
            .sorted_view_iter(e, span)
            .or_else(|| self.stepped_range_iter(e))
        {
            return lift(res);
        }
        // Direct zero-overhead range iteration for `V.after(pos)` and `V.before(pos)`.
        if let ast::ExprKind::Call { callee, args } = &e.kind {
            if let ast::ExprKind::Field { base, name } = &callee.kind {
                if let Some(Sym::Domain(id)) = self.sym(base) {
                    if matches!(name.text.as_str(), "after" | "before") && args.len() == 1 {
                        let pos = self.expr_as(&args[0].value, &Ty::Pos(id));
                        let (lo, hi) = if name.text == "after" {
                            (
                                self.int_add(self.cast(Cast::Ord, pos, Ty::Int), 1),
                                self.mk(ExprKind::DomainLen(id), Ty::Int, span),
                            )
                        } else {
                            (self.int_lit(0, span), self.cast(Cast::Ord, pos, Ty::Int))
                        };
                        return lift((
                            Iter::Range {
                                lo,
                                hi,
                                step: None,
                                rev: false,
                                closed: false,
                            },
                            Ty::Pos(id),
                        ));
                    }
                }
            }
        }
        // `for b in Bool` walks `false`, then `true`.
        if matches!(&e.kind, ast::ExprKind::Name(name) if name == "Bool" && self.lookup(name).is_none())
        {
            let both = [false, true].map(|b| ast::Expr {
                kind: ast::ExprKind::Bool(b),
                span,
            });
            return self.iter(&ast::Expr {
                kind: ast::ExprKind::List(both.to_vec()),
                span,
            });
        }
        // `for (k, v) in m.items` goes over the map itself.
        if let ast::ExprKind::Field { base, name } = &e.kind {
            if name.text == "items"
                && matches!(
                    self.probe_ty_silent(base),
                    Some(Ty::Map(..) | Ty::HashMap(..))
                )
            {
                return self.iter(base);
            }
        }
        let error = |lw: &mut Self| {
            let zero = lw.int_lit(0, span);
            lift((
                Iter::Range {
                    lo: zero.clone(),
                    hi: zero,
                    step: None,
                    rev: false,
                    closed: false,
                },
                Ty::Error,
            ))
        };
        match &e.kind {
            ast::ExprKind::Paren(inner) => self.iter(inner),
            ast::ExprKind::Name(n) if let Some(factors) = self.alias_factors(n) => {
                lift(self.product_walk(factors, span))
            }
            ast::ExprKind::Tuple(items) if !items.is_empty() => {
                let factors: Option<Vec<(Option<String>, ProductFactor)>> = items
                    .iter()
                    .map(|it| self.expr_to_product_factor(it).map(|f| (None, f)))
                    .collect();
                if let Some(factors) = factors {
                    return lift(self.product_walk(factors, span));
                }
                let value = self.expr(e, None);
                self.report(
                    span,
                    msgs::ir::TYPE_MISMATCH_A_FOR_CAN_T,
                    &[&self.show(&value.ty)],
                );
                error(self)
            }
            ast::ExprKind::Record(fields) if !fields.is_empty() => {
                let factors: Option<Vec<(Option<String>, ProductFactor)>> = fields
                    .iter()
                    .map(|(n, it)| {
                        self.expr_to_product_factor(it)
                            .map(|f| (Some(n.text.clone()), f))
                    })
                    .collect();
                if let Some(factors) = factors {
                    return lift(self.product_walk(factors, span));
                }
                let value = self.expr(e, None);
                self.report(
                    span,
                    msgs::ir::TYPE_MISMATCH_A_FOR_CAN_T,
                    &[&self.show(&value.ty)],
                );
                error(self)
            }
            // `for p in ?Col:` walks `none` and then every `Col`.
            ast::ExprKind::Generic { base, args }
                if matches!(&base.kind, ast::ExprKind::Name(n) if n == "Opt")
                    && args.len() == 1 =>
            {
                let domain = match self.resolve_ty(&args[0]) {
                    Ty::Pos(domain) => domain,
                    Ty::Error => return error(self),
                    other => {
                        self.report(
                            span,
                            msgs::ir::TYPE_MISMATCH_A_FOR_CAN_WALK,
                            &[&self.show(&other)],
                        );
                        return error(self);
                    }
                };
                let elem = Ty::opt(Ty::Pos(domain));
                let none = self.mk(ExprKind::None, elem.clone(), span);
                let some_ty = elem.clone();
                let all = self.listed(domain, elem.clone(), Some(none), span, |lw, ord| {
                    let pos = lw.cast(Cast::Trust, ord, Ty::Pos(domain));
                    (None, lw.mk_some(pos, some_ty, span))
                });
                lift((Iter::Seq(all, false), elem))
            }
            ast::ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step,
            } => {
                let lo = self.expr(lo, None);
                let ty = lo.ty.clone();
                if matches!(ty, Ty::I128 | Ty::U64) && step.is_none() {
                    let hi = self.expr_as(hi, &ty);
                    let range = self.wide_range(lo, hi, *inclusive, ty.clone(), span);
                    return (Head::Iter(Iter::RangeVal(range, false)), ty);
                }
                if !matches!(ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_) | Ty::Error) {
                    self.report(
                        span,
                        msgs::ir::TYPE_MISMATCH_A_RANGE_RUNS_OVER,
                        &[&self.show(&ty)],
                    );
                    return error(self);
                }
                let mut hi = self.expr_as(hi, &ty);
                let mut lo = lo;
                if ty != Ty::Int {
                    lo = self.cast(Cast::Ord, lo, Ty::Int);
                    hi = self.cast(Cast::Ord, hi, Ty::Int);
                }
                // Positions have a next one; the last `Int` does not.
                let closed = *inclusive && ty == Ty::Int;
                if *inclusive && !closed {
                    hi = self.int_add(hi, 1);
                }
                let step = step.as_ref().map(|s| self.expr_as(s, &Ty::Int));
                lift((
                    Iter::Range {
                        lo,
                        hi,
                        step,
                        rev: false,
                        closed,
                    },
                    ty,
                ))
            }
            ast::ExprKind::Range {
                lo: Some(lo),
                hi: None,
                step,
                ..
            } => {
                let lo = self.expr(lo, None);
                match lo.ty {
                    Ty::Pos(d) | Ty::Bound(d) => {
                        let cuts = Self::walks_cuts(&lo.ty);
                        let lo_ord = self.cast(Cast::Ord, lo, Ty::Int);
                        let mut hi = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        if cuts {
                            hi = self.int_add(hi, 1);
                        }
                        let step = step.as_ref().map(|s| self.expr_as(s, &Ty::Int));
                        lift((
                            Iter::Range {
                                lo: lo_ord,
                                hi,
                                step,
                                rev: false,
                                closed: false,
                            },
                            if cuts { Ty::Bound(d) } else { Ty::Pos(d) },
                        ))
                    }
                    // `for i in 3..:` counts up until the body leaves.
                    Ty::Int => {
                        let hi = self.int_lit(INF, span);
                        let step = step.as_ref().map(|s| self.expr_as(s, &Ty::Int));
                        lift((
                            Iter::Range {
                                lo,
                                hi,
                                step,
                                rev: false,
                                closed: false,
                            },
                            Ty::Int,
                        ))
                    }
                    Ty::Error => error(self),
                    _ => {
                        self.report(
                            span,
                            msgs::ir::TYPE_MISMATCH_A_FOR_OVER_LO,
                            &[&self.show(&lo.ty)],
                        );
                        error(self)
                    }
                }
            }
            ast::ExprKind::Range {
                lo: None,
                hi: Some(hi),
                inclusive,
                step,
            } => {
                let hi = self.expr(hi, None);
                let step = step.as_ref().map(|s| self.expr_as(s, &Ty::Int));
                match hi.ty {
                    Ty::Pos(d) | Ty::Bound(d) => {
                        let cuts = Self::walks_cuts(&hi.ty);
                        let lo_ord = self.int_lit(0, span);
                        let mut hi_ord = self.cast(Cast::Ord, hi, Ty::Int);
                        if *inclusive {
                            hi_ord = self.int_add(hi_ord, 1);
                        }
                        lift((
                            Iter::Range {
                                lo: lo_ord,
                                hi: hi_ord,
                                step,
                                rev: false,
                                closed: false,
                            },
                            if cuts { Ty::Bound(d) } else { Ty::Pos(d) },
                        ))
                    }
                    // `for k in ..3:` counts 0, 1, 2.
                    Ty::Int => {
                        let lo = self.int_lit(0, span);
                        lift((
                            Iter::Range {
                                lo,
                                hi,
                                step,
                                rev: false,
                                closed: *inclusive,
                            },
                            Ty::Int,
                        ))
                    }
                    Ty::Error => error(self),
                    _ => {
                        let end = self.show(&hi.ty);
                        self.report(span, msgs::check::TYPE_MISMATCH_OPEN_RANGE_END, &[&end]);
                        error(self)
                    }
                }
            }
            ast::ExprKind::Range { .. } => {
                self.report(span, msgs::check::TYPE_MISMATCH_FULL_RANGE, &[]);
                error(self)
            }
            ast::ExprKind::Name(name)
                if self.lookup(name).is_none()
                    && let Some(id) = self.resolve_domain_name(name) =>
            {
                let lo = self.int_lit(0, span);
                let hi = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                lift((
                    Iter::Range {
                        lo,
                        hi,
                        step: None,
                        rev: false,
                        closed: false,
                    },
                    Ty::Pos(id),
                ))
            }
            ast::ExprKind::Name(name)
                if self.lookup(name).is_none()
                    && let Some(GenArg::Ty(Ty::Bound(id))) = self.gen_env.get(name).cloned() =>
            {
                let lo = self.int_lit(0, span);
                let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                let hi = self.int_add(len, 1);
                lift((
                    Iter::Range {
                        lo,
                        hi,
                        step: None,
                        rev: false,
                        closed: false,
                    },
                    Ty::Bound(id),
                ))
            }
            ast::ExprKind::Name(name)
                if self.lookup(name).is_none()
                    && let Some(Sym::Enum(eid)) = self.get_sym(name)
                    && self.enums[eid].variants.iter().all(|(_, f)| f.is_empty()) =>
            {
                let lo = self.int_lit(0, span);
                let hi = self.int_lit(self.enums[eid].variants.len() as i64, span);
                lift((
                    Iter::Range {
                        lo,
                        hi,
                        step: None,
                        rev: false,
                        closed: false,
                    },
                    Ty::Enum(eid),
                ))
            }
            ast::ExprKind::Name(_) | ast::ExprKind::Generic { .. }
                if let Some(sid) = self.index_struct_of(e)
                    && let Some(dom) = self.structs[sid].index_domain =>
            {
                if let Some(nth) = self.own_method(sid, "nth") {
                    return (self.walk(dom, nth, span), Ty::Struct(sid));
                }
                error(self)
            }
            // `(lo..hi).rev by step` counts down from `hi - 1`: the values
            // of `lo'..hi by step` backward, where `lo'` is the last one.
            ast::ExprKind::By { walk, step } => {
                let (iter, ty) = self.iter(walk);
                let step = self.expr_as(step, &Ty::Int);
                let Head::Iter(Iter::Range {
                    lo,
                    hi,
                    step: None,
                    rev,
                    closed,
                }) = iter
                else {
                    self.report(span, cpl_diag::msgs::ir::STEPS_RANGE_NUMBERS_WRITTEN, &[]);
                    return (iter, ty);
                };
                // `for v in V by 2` steps along the positions of `V`.
                if !matches!(ty, Ty::Int | Ty::Pos(_)) {
                    self.report(
                        span,
                        msgs::ir::TYPE_MISMATCH_BY_STEPS_A_RANGE,
                        &[&self.show(&ty)],
                    );
                    return lift((
                        Iter::Range {
                            lo,
                            hi,
                            step: None,
                            rev,
                            closed,
                        },
                        ty,
                    ));
                }
                if !rev {
                    let iter = Iter::Range {
                        lo,
                        hi,
                        step: Some(step),
                        rev,
                        closed,
                    };
                    return lift((iter, ty));
                }
                let down = |lw: &mut Self, lo: Expr, hi: Expr, step: Expr| {
                    let bin =
                        |lw: &mut Self, op, l: Expr, r: Expr| lw.mk_binary(op, l, r, Ty::Int, span);
                    let one = lw.int_lit(1, span);
                    let top = if closed {
                        hi.clone()
                    } else {
                        bin(lw, BinOp::Sub, hi.clone(), one)
                    };
                    let room = bin(lw, BinOp::Sub, top.clone(), lo);
                    let steps = bin(lw, BinOp::Div, room, step.clone());
                    let back = bin(lw, BinOp::Mul, steps, step.clone());
                    let lo = bin(lw, BinOp::Sub, top, back);
                    Iter::Range {
                        lo,
                        hi,
                        step: Some(step),
                        rev: true,
                        closed,
                    }
                };
                if [&lo, &hi, &step]
                    .into_iter()
                    .all(crate::lower::stmt::settled)
                {
                    return lift((down(self, lo, hi, step), ty));
                }
                // Ends that do work are worked out once, ahead of the walk.
                let list = self.collected(Ty::Int, span, |lw, out, push| {
                    let (lo, hi, step) = (lw.hold(lo, out), lw.hold(hi, out), lw.hold(step, out));
                    let iter = down(lw, lo, hi, step);
                    let item = lw.temp(Ty::Int);
                    let lid = lw.fresh_loop();
                    let at = lw.local(item, span);
                    let body = Block {
                        stmts: vec![push(lw, at)],
                    };
                    out.push(Stmt::For(lid, item, iter, body));
                });
                lift((Iter::Seq(list, false), ty))
            }
            // `xs.rev()` walks the same way as `xs.rev`.
            ast::ExprKind::Call { callee, args }
                if args.is_empty()
                    && matches!(&callee.kind, ast::ExprKind::Field { name, .. } if name.text == "rev") =>
            {
                self.iter(callee)
            }
            ast::ExprKind::Field { base, name } if name.text == "rev" => {
                let (iter, ty) = self.iter(base);
                let iter = match iter {
                    Head::Walk(walk) => Head::Walk(Walk {
                        rev: !walk.rev,
                        ..walk
                    }),
                    Head::Iter(iter) => Head::Iter(match iter {
                        Iter::Range {
                            lo,
                            hi,
                            step,
                            rev,
                            closed,
                        } => Iter::Range {
                            lo,
                            hi,
                            step,
                            rev: !rev,
                            closed,
                        },
                        Iter::RangeVal(value, rev) => Iter::RangeVal(value, !rev),
                        Iter::Seq(value, rev) => Iter::Seq(value, !rev),
                        Iter::TreeRange {
                            container,
                            lo,
                            hi,
                            rev,
                        } => Iter::TreeRange {
                            container,
                            lo,
                            hi,
                            rev: !rev,
                        },
                    }),
                };
                (iter, ty)
            }
            ast::ExprKind::Field { base, name } if cpl_check::is_cuts(&name.text) => {
                let domain = match &base.kind {
                    ast::ExprKind::Name(n) if self.lookup(n).is_none() => {
                        self.resolve_domain_name(n)
                    }
                    _ => None,
                };
                let Some(id) = domain else {
                    self.report(name.span, msgs::ir::UNKNOWN_FIELD_CUTS_BELONGS_TO_A, &[]);
                    return error(self);
                };
                let lo = self.int_lit(0, span);
                let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                let hi = self.int_add(len, 1);
                lift((
                    Iter::Range {
                        lo,
                        hi,
                        step: None,
                        rev: false,
                        closed: false,
                    },
                    Ty::Bound(id),
                ))
            }
            _ => {
                if let ast::ExprKind::Field { base, name } = &e.kind {
                    if let ast::ExprKind::Generic {
                        base: gbase,
                        args: gargs,
                    } = &base.kind
                    {
                        if let Some(Sym::GenStruct(gid)) = self.sym(gbase) {
                            let resolved = self.resolve_gen_args_for_struct(gid, gargs);
                            let sid = self.instantiate_struct(gid, resolved);
                            if let Some(&dom_id) = self.structs[sid].owned_domains.get(&name.text) {
                                let lo = self.int_lit(0, span);
                                let hi = self.mk(ExprKind::DomainLen(dom_id), Ty::Int, span);
                                return lift((
                                    Iter::Range {
                                        lo,
                                        hi,
                                        step: None,
                                        rev: false,
                                        closed: false,
                                    },
                                    Ty::Pos(dom_id),
                                ));
                            }
                        }
                    }
                }
                let value = self.expr(e, None);
                if let ExprKind::DomainVal(dom_id) = value.kind {
                    let lo = self.int_lit(0, span);
                    let hi = self.mk(ExprKind::DomainLen(dom_id), Ty::Int, span);
                    return lift((
                        Iter::Range {
                            lo,
                            hi,
                            step: None,
                            rev: false,
                            closed: false,
                        },
                        Ty::Pos(dom_id),
                    ));
                }
                if let Ty::Range(inner) = value.ty.clone() {
                    return lift((Iter::RangeVal(value, false), *inner));
                }
                // A struct with a zero-arg `each(self)` (such as `Bitset<D>`), or the
                // `iter(self)` of its `impl Seq`, walks what that call gives.
                if let Ty::Struct(sid) = value.ty {
                    if let Some((each_fn, elem)) = self.struct_walk(sid, span) {
                        let call = self.mk(
                            ExprKind::Call(each_fn, vec![Arg::Value(value)]),
                            Ty::list(elem.clone()),
                            span,
                        );
                        return lift((Iter::Seq(call, false), elem));
                    }
                    // A struct with `len` and `nth(self, j)` (an `Indexed` view such
                    // as `Stepped<S>`) walks `nth(j)` for `j` in `0..len`.
                    let len_fn = self.own_method(sid, "len");
                    let nth_fn = self.own_method(sid, "nth").or_else(|| {
                        // `nth<T>(self, j) -> T` over a sequence parameter: `T` is its element.
                        let gfid = *self.structs[sid].gen_methods.get("nth")?;
                        if self.gen_fns[gfid].decl.generics.len() != 1 {
                            return None;
                        }
                        let elem = self.structs[sid].env.values().find_map(|a| match a {
                            GenArg::Ty(t) => t.seq_elem(),
                            _ => None,
                        })?;
                        Some(self.instantiate_fn(
                            gfid,
                            vec![GenArg::Ty(elem)],
                            &[],
                            &[],
                            Vec::new(),
                            &[],
                        ))
                    });
                    if let (Some(len_fn), Some(nth_fn)) = (len_fn, nth_fn) {
                        if self.sigs[len_fn].params.is_empty()
                            && self.sigs[nth_fn].params.len() == 1
                        {
                            self.queue_fn(len_fn);
                            self.queue_fn(nth_fn);
                            let elem = self.ret_of(nth_fn, span);
                            let recv = self.temp(value.ty.clone());
                            let at = self.mk(ExprKind::Local(recv), value.ty.clone(), span);
                            self.cx.pending.push_let(recv, value);
                            let len = self.mk(
                                ExprKind::Call(len_fn, vec![Arg::Value(at.clone())]),
                                Ty::Int,
                                span,
                            );
                            let iter = Head::Walk(Walk {
                                k: self.temp(Ty::Int),
                                n: self.temp(Ty::Int),
                                len,
                                nth: nth_fn,
                                recv: Some(at),
                                rev: false,
                            });
                            return (iter, elem);
                        }
                    }
                }
                let elem = match &value.ty {
                    Ty::Array { elem, .. }
                    | Ty::List(elem)
                    | Ty::ListView(elem, _)
                    | Ty::Keyed(elem, _)
                    | Ty::MutView(elem, _)
                    | Ty::Set(elem)
                    | Ty::HashSet(elem) => (**elem).clone(),
                    Ty::Str | Ty::StrView => Ty::Char,
                    Ty::Map(key, val) | Ty::HashMap(key, val) => {
                        Ty::Tuple(vec![(**key).clone(), (**val).clone()])
                    }
                    Ty::Error => return error(self),
                    Ty::Int => {
                        self.report(span, msgs::ir::TYPE_MISMATCH_A_FOR_CAN_T_2, &[]);
                        self.help("count with a range: `for i in 0..n`");
                        return error(self);
                    }
                    other => {
                        self.report(
                            span,
                            msgs::ir::TYPE_MISMATCH_A_FOR_CAN_T,
                            &[&self.show(other)],
                        );
                        return error(self);
                    }
                };
                lift((Iter::Seq(value, false), elem))
            }
        }
    }
}

/// An `Iter` and the type of each value, as a [`Head`].
fn lift((iter, ty): (Iter, Ty)) -> (Head, Ty) {
    (Head::Iter(iter), ty)
}
