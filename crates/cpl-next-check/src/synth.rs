//! A made fn's body (`decls::Body::Synth`): each closed variant typed as the source it stands
//! for, through ordinary resolution, so lower builds it as it would that source (R08.1.6, R02.7,
//! R04.2.10, R08.4.1). Every variant over fields is built from one per-field typed expansion,
//! [`Cx::per_field`], which std's `fields(a)` unrolling reuses once derived bodies are std's.
use super::call::CallArg;
use super::lambda::{LamBody, LamParam};
use super::member::Called;
use super::*;
use cpl_next_decls::decls::Role;
use cpl_next_decls::{LawRow, SynthBody};

impl<'a> Cx<'a> {
    /// The body `b` of made fn `f`, its receiver and parameters bound to `params`.
    pub(super) fn synth_body(
        &mut self,
        f: &decls::FnDecl,
        b: &SynthBody,
        params: &[LocalId],
        ret: &Ty,
    ) -> TExpr {
        // Said at the derive that made it.
        self.b.item = f.span.item.clone();
        let span = f.span.span;
        let mut vars: Vec<(TExpr, bool)> = Vec::new();
        if let Some(l) = self.b.self_local {
            vars.push((self.local_value(l, span), false));
        }
        for (&l, p) in params.iter().zip(&f.params) {
            vars.push((self.local_value(l, span), p.mutable));
        }
        let Some((first, _)) = vars.first().cloned() else {
            return self.synth_static(f, b, span);
        };
        // The trait this fn's derived impl is of, whose fns its fields' impls give.
        let tr = match f.owner {
            Some(Owner::Impl(k)) => Some(self.items.imp(k).trait_.tr),
            _ => None,
        };
        let rest: Vec<TArg> = (vars[1..].iter().cloned())
            .map(|(v, m)| if m { TArg::Mut(v) } else { TArg::Value(v) })
            .collect();
        match b {
            SynthBody::EqFields(fs) => {
                let ty = first.ty.clone();
                let eqs = self.per_field(&ty, fs, span, |cx, n, _| {
                    cx.field_cmp(CmpOp::Eq, &vars, n, span)
                });
                let mut out: Option<TExpr> = None;
                for eq in eqs {
                    out = Some(match out {
                        None => eq,
                        Some(a) => self.binary(BinOp::And, a, eq, span),
                    });
                }
                out.unwrap_or_else(|| lit(TK::Bool(true), Ty::Bool, span))
            }
            SynthBody::LtFields(fs) => {
                let ty = first.ty.clone();
                let lts = self.per_field(&ty, fs, span, |cx, n, _| {
                    cx.field_cmp(CmpOp::Lt, &vars, n, span)
                });
                let eqs = self.per_field(&ty, fs, span, |cx, n, _| {
                    cx.field_cmp(CmpOp::Eq, &vars, n, span)
                });
                let mut out: Option<TExpr> = None;
                for (lt, eq) in lts.into_iter().zip(eqs).rev() {
                    out = Some(match out {
                        None => lt,
                        Some(r) => {
                            let tie = self.binary(BinOp::And, eq, r, span);
                            self.binary(BinOp::Or, lt, tie, span)
                        }
                    });
                }
                out.unwrap_or_else(|| lit(TK::Bool(false), Ty::Bool, span))
            }
            SynthBody::EachField { method, fields } => {
                let ty = self.shallow(&first.ty);
                self.made_struct(&ty, fields, span, |cx, n, _| {
                    let x = cx.field_named(&first, n, span);
                    cx.method_named(x, (*method, tr), Vec::new(), span)
                })
            }
            SynthBody::MadeEach { method, fields } => {
                self.made_struct(ret, fields, span, |cx, _, t| {
                    cx.read_as(first.clone(), *method, t, span)
                })
            }
            SynthBody::PackFields { method, fields } => {
                let packed = self.packed(&first, fields, span);
                self.method_named(packed, (*method, tr), rest, span)
            }
            SynthBody::VariantName { method, variants } => {
                let tag = self.enum_tag(first);
                let names = (variants.iter())
                    .map(|(_, token)| self.text_value(token.clone(), span))
                    .collect();
                let name = self.pick(tag, names, span);
                self.method_named(name, (*method, tr), rest, span)
            }
            SynthBody::VariantByTag { method, variants } => {
                self.variant_by_tag(&first, *method, variants, ret, span)
            }
            SynthBody::Getter(n) => self.field_named(&first, *n, span),
            SynthBody::Without { added, kept } => {
                let given: Vec<TExpr> = vars[1..].iter().map(|(v, _)| v.clone()).collect();
                let all: Vec<Sym> = added.iter().chain(kept).copied().collect();
                self.made_struct(ret, &all, span, |cx, n, t| {
                    match added.iter().position(|a| *a == n) {
                        Some(k) => cx.coerce(given[k].clone(), t),
                        None => cx.field_named(&first, n, span),
                    }
                })
            }
            SynthBody::ProductNth { nth, count, fields } => {
                let tys = self.struct_field_tys(ret, span);
                self.made_struct(ret, fields, span, |cx, n, t| {
                    let i = fields.iter().position(|f| *f == n).unwrap_or(0);
                    let after = cx.counts(&tys[i + 1..], *count, span);
                    let size = cx.type_count(t, *count, span);
                    let k = cx.binary(BinOp::Div, first.clone(), after, span);
                    let k = cx.binary(BinOp::Rem, k, size, span);
                    let arg = [CallArg::held(None, k, span)];
                    let at = cx.called(*nth, span);
                    (cx.type_static(t.clone(), at, Some(&arg), span))
                        .unwrap_or_else(|| Self::error_node(span))
                })
            }
            SynthBody::ProductOrd { ord, count, fields } => {
                let ty = first.ty.clone();
                let ords = self.per_field(&ty, fields, span, |cx, n, _| {
                    let x = cx.field_named(&first, n, span);
                    let at = cx.called(*ord, span);
                    cx.member_on(x, at, None, span, Some(&Ty::Int), &[])
                });
                let counts =
                    self.per_field(&ty, fields, span, |cx, _, t| cx.type_count(t, *count, span));
                let mut acc: Option<TExpr> = None;
                for (o, c) in ords.into_iter().zip(counts) {
                    acc = Some(match acc {
                        None => o,
                        Some(a) => {
                            let a = self.binary(BinOp::Mul, a, c, span);
                            self.binary(BinOp::Add, a, o, span)
                        }
                    });
                }
                acc.unwrap_or_else(|| lit(TK::Int(0), Ty::Int, span))
            }
            SynthBody::EnumToInt(_) => {
                let Ty::Struct(e, ..) = self.shallow(&first.ty) else {
                    return Self::error_node(span);
                };
                let values = self.items.s(e).values.clone();
                let tag = self.enum_tag(first);
                let ints = values
                    .iter()
                    .map(|&k| lit(TK::Int(k), Ty::Int, span))
                    .collect();
                self.pick(tag, ints, span)
            }
            SynthBody::IntToEnum { .. } => self.int_to_enum(first, ret, span),
            SynthBody::ProductCount { .. } | SynthBody::EnumCount(_) => {
                self.synth_static(f, b, span)
            }
            SynthBody::Laws { len, laws } => self.laws_list(&first, *len, laws, ret, span),
        }
    }

    /// `laws` of `me`: per law `(name: "..", sizes: s => [pool.len, ..], holds: (s, p) =>
    /// law(me, pool[p[0]], ..))`, each pool the samples `s` or their field, and the law called as
    /// its owner's fn, so a trait's law runs the impl's fns (R08.3.5).
    fn laws_list(&mut self, me: &TExpr, len: Sym, laws: &[LawRow], ret: &Ty, span: Span) -> TExpr {
        let row = match self.shallow(ret) {
            Ty::Struct(_, ts, _) => ts.first().cloned(),
            _ => None,
        };
        let Some((rec, _)) = row.as_ref().and_then(|r| self.field_tys(r)) else {
            return Self::error_node(span);
        };
        let names: Vec<Sym> = self.items.s(rec).fields.iter().map(|(n, _)| *n).collect();
        let (Some(row), [_, _, _]) = (row, &names[..]) else {
            self.err(
                span,
                Msg::internal("`@role(lawful)`'s laws are not (name, sizes, holds)"),
            );
            return Self::error_node(span);
        };
        let mut out = Vec::new();
        for law in laws {
            let v = self.made_struct(&row, &names, span, |cx, n, t| {
                let pools = law.pools.clone();
                let pool = move |cx: &mut Cx<'a>, s: &TExpr, p: &Option<Sym>| match p {
                    Some(f) => cx.field_named(s, *f, span),
                    None => s.clone(),
                };
                let param = || LamParam {
                    pat: None,
                    ty: None,
                    mutable: false,
                };
                if n == names[0] {
                    return cx.text_value(law.name.clone(), span);
                }
                if n == names[1] {
                    let body = move |cx: &mut Cx<'a>, xs: Vec<TExpr>, _: &Ty| {
                        let Some(s) = xs.into_iter().next() else {
                            return Self::error_node(span);
                        };
                        let lens = (pools.iter())
                            .map(|p| {
                                let p = pool(cx, &s, p);
                                cx.member_on(p, cx.called(len, span), None, span, None, &[])
                            })
                            .collect();
                        cx.list_of(lens, span)
                    };
                    let body = LamBody::Built(Box::new(body));
                    return cx.lambda_of(vec![param()], None, body, Some(t), span);
                }
                let (me, f, targs) = (me.clone(), law.law, law.targs.clone());
                // A struct's law takes its sets as the struct's own methods do: the receiver's.
                let sets = match (cx.items.f(f).owner, cx.shallow(&me.ty)) {
                    (Some(Owner::Struct(_)), Ty::Struct(_, _, ss)) => ss,
                    _ => Vec::new(),
                };
                let body = move |cx: &mut Cx<'a>, xs: Vec<TExpr>, _: &Ty| {
                    let [s, picks] = &xs[..] else {
                        return Self::error_node(span);
                    };
                    let args: Vec<CallArg> = (pools.iter().enumerate())
                        .map(|(k, p)| {
                            let p = pool(cx, s, p);
                            let k = lit(TK::Int(k as i64), Ty::Int, span);
                            let k = cx.elem(picks.clone(), k, span);
                            CallArg::held(None, cx.elem(p, k, span), span)
                        })
                        .collect();
                    let callee = cx.fnref(f, targs, sets);
                    let recv = cx.recv_arg(f, me, span);
                    cx.call_method(callee, recv, false, Some(&args), span, None)
                };
                let body = LamBody::Built(Box::new(body));
                cx.lambda_of(vec![param(), param()], None, body, Some(t), span)
            });
            out.push(v);
        }
        let list = self.list_of(out, span);
        self.coerce(list, ret)
    }

    /// A made fn with no receiver or parameter: a domain's size.
    fn synth_static(&mut self, f: &decls::FnDecl, b: &SynthBody, span: Span) -> TExpr {
        match b {
            SynthBody::EnumCount(n) => lit(TK::Int(*n as i64), Ty::Int, span),
            SynthBody::ProductCount { count, .. } => {
                let of = (f.self_ty.clone())
                    .or_else(|| match f.owner {
                        Some(Owner::Impl(k)) => Some(self.items.imp(k).for_.clone()),
                        _ => None,
                    })
                    .unwrap_or(Ty::Error);
                let tys = self.struct_field_tys(&of, span);
                self.counts(&tys, *count, span)
            }
            _ => Self::error_node(span),
        }
    }

    fn local_value(&self, l: LocalId, span: Span) -> TExpr {
        let ty = self.b.locals[l].ty.clone();
        lit(TK::Var(VarRef::Local(l)), ty, span)
    }

    fn called(&self, s: Sym, span: Span) -> Called<'a> {
        let items: &'a dyn Items = self.items;
        Called {
            text: items.text(s),
            sym: Some(s),
            span,
        }
    }

    /// `recv.m(args)`: the `m` of `recv`'s impl of the trait this fn's impl is of, as
    /// `Trait.m(recv, ..)` finds it (R08.1.4), or by bound for a parameter.
    fn method_named(
        &mut self,
        recv: TExpr,
        (m, tr): (Sym, Option<TraitId>),
        args: Vec<TArg>,
        span: Span,
    ) -> TExpr {
        let args: Vec<CallArg> = (args.into_iter())
            .map(|a| match a {
                TArg::Value(v) => CallArg::held(None, v, span),
                TArg::Mut(v) => CallArg {
                    mutable: true,
                    ..CallArg::held(None, v, span)
                },
            })
            .collect();
        let rt = self.shallow(&recv.ty);
        let of_impl = (!matches!(rt, Ty::Param(_) | Ty::Var(_)))
            .then(|| self.impls().impl_member(&rt, m, tr))
            .flatten();
        let Some(decls::Member::Method(id, ts, ss)) = of_impl else {
            let at = self.called(m, span);
            return self.member_on(recv, at, Some(&args), span, None, &[]);
        };
        let f = self.fnref(id, ts, ss);
        let recv = self.recv_arg(id, recv, span);
        self.call_method(f, recv, false, Some(&args), span, None)
    }

    /// `p.m<T>()`: a value of type `t` its maker `p` reads.
    fn read_as(&mut self, p: TExpr, m: Sym, t: &Ty, span: Span) -> TExpr {
        let at = self.called(m, span);
        let v = self.member_on(p, at, Some(&[]), span, Some(t), std::slice::from_ref(t));
        self.coerce(v, t)
    }

    fn struct_field_tys(&mut self, t: &Ty, span: Span) -> Vec<Ty> {
        match self.field_tys(t) {
            Some((_, tys)) => tys,
            None => {
                self.err(span, Msg::internal("a made fn's struct has no fields"));
                Vec::new()
            }
        }
    }

    /// Field `n` of struct value `base`.
    fn field_named(&mut self, base: &TExpr, n: Sym, span: Span) -> TExpr {
        let Some((s, tys)) = self.field_tys(&base.ty) else {
            return Self::error_node(span);
        };
        match self.items.s(s).fields.iter().position(|(f, _)| *f == n) {
            Some(i) => lit(TK::Field(Box::new(base.clone()), i), tys[i].clone(), span),
            None => Self::error_node(span),
        }
    }

    /// `a.n op b.n`, of the first two of `vars`.
    fn field_cmp(&mut self, op: CmpOp, vars: &[(TExpr, bool)], n: Sym, span: Span) -> TExpr {
        let [(a, _), (b, _), ..] = vars else {
            return Self::error_node(span);
        };
        let (x, y) = (self.field_named(a, n, span), self.field_named(b, n, span));
        self.ordered(op, x, y, span)
    }

    /// `each(name, type)` for each of `fields` of struct type `ty`, in `fields`' order: the one
    /// per-field typed expansion the variants over fields share.
    fn per_field(
        &mut self,
        ty: &Ty,
        fields: &[Sym],
        span: Span,
        mut each: impl FnMut(&mut Self, Sym, &Ty) -> TExpr,
    ) -> Vec<TExpr> {
        let Some((s, tys)) = self.field_tys(ty) else {
            return vec![Self::error_node(span)];
        };
        let names: Vec<Sym> = self.items.s(s).fields.iter().map(|(n, _)| *n).collect();
        let mut out = Vec::new();
        for &n in fields {
            match names.iter().position(|f| *f == n) {
                Some(i) => out.push(each(self, n, &tys[i])),
                None => {
                    self.err(
                        span,
                        Msg::internal("a made fn names a field its struct lacks"),
                    );
                    out.push(Self::error_node(span));
                }
            }
        }
        out
    }

    /// Struct `ty` with each of its fields `value(name, type)`; every field must be in `fields`.
    fn made_struct(
        &mut self,
        ty: &Ty,
        fields: &[Sym],
        span: Span,
        mut value: impl FnMut(&mut Self, Sym, &Ty) -> TExpr,
    ) -> TExpr {
        let Some((s, _)) = self.field_tys(ty) else {
            return Self::error_node(span);
        };
        let names: Vec<Sym> = self.items.s(s).fields.iter().map(|(n, _)| *n).collect();
        if names.iter().any(|n| !fields.contains(n)) {
            self.err(span, Msg::internal("a made struct leaves a field out"));
            return Self::error_node(span);
        }
        let vals = self.per_field(ty, &names, span, |cx, n, t| {
            let v = value(cx, n, t);
            cx.coerce(v, t)
        });
        lit(TK::Struct(vals), ty.clone(), span)
    }

    /// `Pack(a.x, Pack(a.y, ()))` over the `@role(pack)` struct.
    fn packed(&mut self, a: &TExpr, fields: &[Sym], span: Span) -> TExpr {
        let Some(pack) = self.role_structs.role_struct(Role::Pack) else {
            self.err(span, Msg::internal("no struct has `@role(pack)`"));
            return Self::error_node(span);
        };
        let ty = a.ty.clone();
        let xs = self.per_field(&ty, fields, span, |cx, n, _| cx.field_named(a, n, span));
        let mut out = lit(TK::Unit, Ty::Unit, span);
        for x in xs.into_iter().rev() {
            let targs = vec![x.ty.clone(), out.ty.clone()];
            let args = [CallArg::held(None, x, span), CallArg::held(None, out, span)];
            out = self.construct(pack, targs, Vec::new(), &args, span, None);
        }
        out
    }

    /// `T.count` of each of `tys`, multiplied; `1` for none.
    fn counts(&mut self, tys: &[Ty], count: Sym, span: Span) -> TExpr {
        let mut out: Option<TExpr> = None;
        for t in tys {
            let c = self.type_count(t, count, span);
            out = Some(match out {
                None => c,
                Some(a) => self.binary(BinOp::Mul, a, c, span),
            });
        }
        out.unwrap_or_else(|| lit(TK::Int(1), Ty::Int, span))
    }

    fn type_count(&mut self, t: &Ty, count: Sym, span: Span) -> TExpr {
        let at = self.called(count, span);
        (self.type_static(t.clone(), at, None, span)).unwrap_or_else(|| Self::error_node(span))
    }

    /// `choices[k]` for the `Int` `k`, the last when `k` is past the others.
    fn pick(&mut self, k: TExpr, mut choices: Vec<TExpr>, span: Span) -> TExpr {
        let Some(mut out) = choices.pop() else {
            return Self::error_node(span);
        };
        if choices.is_empty() {
            return out;
        }
        let t = self.temp("$tag", Ty::Int);
        let ty = out.ty.clone();
        for (i, c) in choices.into_iter().enumerate().rev() {
            let at = lit(TK::Var(t), Ty::Int, span);
            let is = lit(
                TK::Cmp(
                    CmpOp::Eq,
                    Box::new(at),
                    Box::new(lit(TK::Int(i as i64), Ty::Int, span)),
                ),
                Ty::Bool,
                span,
            );
            out = lit(
                TK::If(Box::new(is), Box::new(c), Box::new(out)),
                ty.clone(),
                span,
            );
        }
        lit(
            TK::Block(vec![TStmt::Let(t, k)], Some(Box::new(out))),
            ty,
            span,
        )
    }

    /// `E(x)` for a payload-free enum `ret`: `x` through the `@role(variant_check)` fn, then the
    /// variant whose `Int` it is.
    fn int_to_enum(&mut self, x: TExpr, ret: &Ty, span: Span) -> TExpr {
        let Ty::Struct(e, ..) = self.shallow(ret) else {
            return Self::error_node(span);
        };
        let values = self.items.s(e).values.clone();
        let is = |k: i64| {
            let k = lit(TK::Int(k), Ty::Int, span);
            lit(
                TK::Cmp(CmpOp::Eq, Box::new(x.clone()), Box::new(k)),
                Ty::Bool,
                span,
            )
        };
        let known = (values.iter().map(|&k| is(k)))
            .reduce(|a, b| lit(TK::Bin(BinOp::Or, Box::new(a), Box::new(b)), Ty::Bool, span))
            .unwrap_or_else(|| lit(TK::Bool(false), Ty::Bool, span));
        let checked = self.marked_call(
            self.rows.role_fn(Role::VariantCheck),
            "role(variant_check)",
            vec![known],
            span,
        );
        let n = values.len();
        let mut out = self.variant(e, n.saturating_sub(1), Vec::new(), &[], span);
        for i in (0..n.saturating_sub(1)).rev() {
            let v = self.variant(e, i, Vec::new(), &[], span);
            out = lit(
                TK::If(Box::new(is(values[i])), Box::new(v), Box::new(out)),
                ret.clone(),
                span,
            );
        }
        lit(
            TK::Block(vec![TStmt::Expr(checked)], Some(Box::new(out))),
            ret.clone(),
            span,
        )
    }

    /// `match p.variant([tags]): 0: E.A(p.m<F>(), ..) ..`: the enum `ret` its maker `p` reads by
    /// tag, each payload field at its own type.
    fn variant_by_tag(
        &mut self,
        p: &TExpr,
        m: Sym,
        variants: &[(Sym, String, usize)],
        ret: &Ty,
        span: Span,
    ) -> TExpr {
        let Ty::Struct(e, targs, _) = self.shallow(ret) else {
            return Self::error_node(span);
        };
        let tags = (variants.iter())
            .map(|(_, token, _)| self.text_value(token.clone(), span))
            .collect();
        let tags = self.list_of(tags, span);
        let tag = self.method_call(p.clone(), "variant", vec![TArg::Value(tags)], span, false);
        let made = (0..variants.len())
            .map(|i| {
                let payload = self.items.s(e).variants[i].1;
                let tys =
                    self.struct_field_tys(&Ty::Struct(payload, targs.clone(), Vec::new()), span);
                let args: Vec<CallArg> = (tys.iter())
                    .map(|t| CallArg::held(None, self.read_as(p.clone(), m, t, span), span))
                    .collect();
                self.variant(e, i, targs.clone(), &args, span)
            })
            .collect();
        self.pick(tag, made, span)
    }
}

fn lit(kind: TK, ty: Ty, span: Span) -> TExpr {
    TExpr { kind, ty, span }
}
