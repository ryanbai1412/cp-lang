//! Structs and enums as values: building them, their variants, and `match` (R02.6, R02.7).
use super::expr::{block, Formal};
use super::*;

impl Cx<'_> {
    /// An enum whose variants carry nothing: a value is which variant it is.
    pub(super) fn plain_enum(&self, t: &Ty) -> Option<StructId> {
        match self.shallow(t) {
            Ty::Struct(id, ..) => {
                let vs = &self.d.s(id).variants;
                let plain = vs.iter().all(|(_, p)| self.d.s(*p).fields.is_empty());
                (!vs.is_empty() && plain).then_some(id)
            }
            _ => None,
        }
    }
    /// `E(x)` of an enum whose variants carry nothing: the variant whose value is `x`; any
    /// other `x` traps (R02.7.3).
    pub(super) fn enum_of_int(&mut self, e: StructId, x: &ast::Expr, span: Span) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let x = self.expr(x, Some(&Ty::Int));
        let x = self.coerce(x, &Ty::Int);
        let t = self.bind("$value", Ty::Int);
        let read = || mk(TK::Var(t), Ty::Int);
        let is = |k: i64| {
            let k = mk(TK::Int(k), Ty::Int);
            mk(TK::Cmp(CmpOp::Eq, Box::new(read()), Box::new(k)), Ty::Bool)
        };
        let values = self.d.s(e).values.clone();
        let name = self.d.s(e).name.clone();
        let known = (values.iter().map(|&k| is(k)))
            .reduce(|a, b| mk(TK::Bin(BinOp::Or, Box::new(a), Box::new(b)), Ty::Bool));
        let check = TStmt::Assert(
            known.unwrap_or_else(|| mk(TK::Bool(false), Ty::Bool)),
            format!("no variant of `{name}` has this value"),
            span,
            CheckKind::Domain,
        );
        let last = values.len().saturating_sub(1);
        let mut pick = self.variant(e, last, Vec::new(), &[], span);
        for i in (0..last).rev() {
            let v = self.variant(e, i, Vec::new(), &[], span);
            let ty = v.ty.clone();
            pick = mk(
                TK::If(Box::new(is(values[i])), Box::new(v), Box::new(pick)),
                ty,
            );
        }
        let ty = pick.ty.clone();
        mk(
            TK::Block(vec![TStmt::Let(t, x), check], Some(Box::new(pick))),
            ty,
        )
    }
    /// Which variant an enum value is, from 0 in declaration order.
    pub(super) fn enum_tag(&mut self, x: TExpr) -> TExpr {
        let (span, ty) = (x.span, x.ty.clone());
        let tmp = self.bind("$enum", ty.clone());
        let var = TExpr {
            kind: TK::Var(tmp),
            ty,
            span,
        };
        let tag = TExpr {
            kind: TK::Tag(Box::new(var)),
            ty: Ty::Int,
            span,
        };
        TExpr {
            kind: TK::Block(vec![TStmt::Let(tmp, x)], Some(Box::new(tag))),
            ty: Ty::Int,
            span,
        }
    }
    /// `S(args)`: the fields of struct `d` from `args`, given its type and set arguments.
    pub(super) fn construct(
        &mut self,
        d: StructId,
        targs: Vec<Ty>,
        sets: Vec<SetTy>,
        args: &[ast::Arg],
        span: Span,
    ) -> TExpr {
        let s = self.d.s(d);
        let sub = Subst {
            tys: &targs,
            sets: &sets,
        };
        let formals: Vec<Formal> = (s.fields.iter().zip(&s.defaults))
            .map(|((n, t), e)| Formal {
                label: n,
                ty: sub.ty(t),
                mutable: false,
                named_only: false,
                default: e.as_ref().map(|e| (e, s.module, s.item)),
            })
            .collect();
        let (pre, fields) = self.args(args, &formals, span);
        let fields = (fields.into_iter())
            .map(|a| match a {
                TArg::Value(v) | TArg::Mut(v) => v,
            })
            .collect();
        let ty = Ty::Struct(d, targs, sets);
        block(
            pre,
            TExpr {
                kind: TK::Struct(fields),
                ty,
                span,
            },
        )
    }

    /// Variant `i` of enum `e`, its payload built from `args` (R02.7.1).
    pub(super) fn variant(
        &mut self,
        e: StructId,
        i: usize,
        mut targs: Vec<Ty>,
        args: &[ast::Arg],
        span: Span,
    ) -> TExpr {
        let s = self.d.s(e);
        let fresh = s.generics.len().saturating_sub(targs.len());
        targs.extend((0..fresh).map(|_| self.new_ty()));
        let p = s.variants[i].1;
        if self.d.s(p).n_sets > 0 {
            self.err(span, "slice: a function value in a variant's payload");
        }
        let payload = self.construct(p, targs.clone(), Vec::new(), args, span);
        TExpr {
            kind: TK::Variant(i, Box::new(payload)),
            ty: Ty::Struct(e, targs, Vec::new()),
            span,
        }
    }

    /// `match x: arms`: a test of `x`'s variant per arm, with the payload bound in its arm, and every
    /// variant met by an arm or a default (R02.7.2, R02.7.4).
    pub(super) fn match_(
        &mut self,
        scrut: &ast::Expr,
        arms: &[ast::Arm],
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let x = self.expr(scrut, None);
        let xt = self.shallow(&x.ty);
        let (e, targs) = match &xt {
            Ty::Struct(e, targs, _) if !self.d.s(*e).variants.is_empty() => (*e, targs.clone()),
            _ => {
                self.err(span, "slice: a match on a value that is not an enum");
                return mk(TK::Error, Ty::Error);
            }
        };
        let variants = self.d.s(e).variants.clone();
        self.scopes.push(HashMap::new());
        let tmp = self.bind("$match", xt.clone());
        let stmts = vec![TStmt::Let(tmp, x)];
        let whole = || TExpr {
            kind: TK::Var(tmp),
            ty: xt.clone(),
            span,
        };
        let mut ty = want.cloned();
        let mut cases: Vec<Option<TExpr>> = vec![None; variants.len()];
        let mut default: Option<TExpr> = None;
        for arm in arms {
            if arm.guard.is_some() {
                self.err(arm.pat.span, "slice: a match guard");
            }
            self.scopes.push(HashMap::new());
            let mut binds = Vec::new();
            let which = match &arm.pat.kind {
                PatternKind::Wildcard => None,
                PatternKind::Name(v) if variants.iter().any(|(n, _)| n == v) => {
                    variants.iter().position(|(n, _)| n == v)
                }
                PatternKind::Name(b) => {
                    let l = self.bind(b, xt.clone());
                    binds.push(TStmt::Let(l, whole()));
                    None
                }
                PatternKind::Variant { path, args } => {
                    let v = &path[path.len() - 1].text;
                    match variants.iter().position(|(n, _)| n == v) {
                        Some(i) => {
                            let p = variants[i].1;
                            let fields = self.d.s(p).fields.clone();
                            if args.len() != fields.len() {
                                self.err(
                                    arm.pat.span,
                                    format!(
                                        "{}: `{v}` has {} fields, found {}",
                                        cpl_diag::codes::WRONG_ARG_COUNT.0,
                                        fields.len(),
                                        args.len()
                                    ),
                                );
                            }
                            let sub = Subst {
                                tys: &targs,
                                sets: &[],
                            };
                            let payload = mk(
                                TK::Payload(Box::new(whole()), i),
                                Ty::Struct(p, targs.clone(), Vec::new()),
                            );
                            for (j, (a, (_, t))) in args.iter().zip(&fields).enumerate() {
                                match &a.kind {
                                    PatternKind::Wildcard => {}
                                    PatternKind::Name(b) => {
                                        let t = sub.ty(t);
                                        let l = self.bind(b, t.clone());
                                        let field = TK::Field(Box::new(payload.clone()), j);
                                        binds.push(TStmt::Let(l, mk(field, t)));
                                    }
                                    _ => self.err(a.span, "slice: a nested pattern"),
                                }
                            }
                            Some(i)
                        }
                        None => {
                            let en = self.d.s(e).name.clone();
                            self.err(
                                arm.pat.span,
                                format!(
                                    "{}: `{en}` has no variant `{v}`",
                                    cpl_diag::codes::UNKNOWN_FIELD.0
                                ),
                            );
                            None
                        }
                    }
                }
                _ => {
                    self.err(
                        arm.pat.span,
                        "slice: a pattern other than a variant or a name",
                    );
                    None
                }
            };
            let covered = which.map_or(default.is_some(), |i| cases[i].is_some());
            if default.is_some() || covered {
                self.err(
                    arm.pat.span,
                    format!(
                        "{}: this arm never runs: the arms before it match everything it does",
                        cpl_diag::codes::UNREACHABLE_ARM.0
                    ),
                );
            }
            let body = self.block(&arm.body, ty.as_ref());
            self.scopes.pop();
            let body = match &ty {
                Some(t) if self.shallow(t) != Ty::Unit => self.coerce(body, t),
                Some(_) => body,
                None => {
                    ty = Some(body.ty.clone());
                    body
                }
            };
            let bt = body.ty.clone();
            let body = mk(TK::Block(binds, Some(Box::new(body))), bt);
            match which {
                Some(i) if cases[i].is_none() => cases[i] = Some(body),
                Some(_) => {}
                None => default = default.or(Some(body)),
            }
        }
        let ty = ty.unwrap_or(Ty::Unit);
        let missing: Vec<&str> = (variants.iter().zip(&cases))
            .filter(|(_, c)| c.is_none())
            .map(|((n, _), _)| n.as_str())
            .collect();
        let mut tail = match (default, missing.is_empty()) {
            (Some(d), _) => d,
            (None, true) => match cases.pop() {
                Some(Some(last)) => last,
                _ => mk(TK::Error, ty.clone()),
            },
            (None, false) => {
                self.err(
                    span,
                    format!(
                        "{}: `match` misses {}",
                        cpl_diag::codes::NON_EXHAUSTIVE_MATCH.0,
                        missing.join(", ")
                    ),
                );
                mk(TK::Error, ty.clone())
            }
        };
        for (i, c) in cases.into_iter().enumerate().rev() {
            if let Some(c) = c {
                let tag = mk(TK::Tag(Box::new(whole())), Ty::Int);
                let test = mk(
                    TK::Cmp(
                        CmpOp::Eq,
                        Box::new(tag),
                        Box::new(mk(TK::Int(i as i64), Ty::Int)),
                    ),
                    Ty::Bool,
                );
                tail = mk(
                    TK::If(Box::new(test), Box::new(c), Box::new(tail)),
                    ty.clone(),
                );
            }
        }
        self.scopes.pop();
        mk(TK::Block(stmts, Some(Box::new(tail))), ty)
    }
}
