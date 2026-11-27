//! Structs and enums as values: building them, their variants, and `match` (R02.6, R02.7).
use super::expr::{block, Formal};
use super::*;
use cpl_next_ir::unbuilt::Unbuilt;

impl Cx<'_> {
    /// `T(x)` for a core type or a payload-free enum `T`: `x`'s impl of core's `@convert` trait at
    /// `T` (R04.2.28, R02.7); a position's `Int` is its integer.
    pub(super) fn convert(&mut self, to: Ty, x: &ast::Expr, span: Span, why: Origin) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let v = self.expr(x, None);
        if to == Ty::Int {
            // Until `Domain` is a trait type parameters meet by bounds, a parameter is a domain.
            let bounded = match self.shallow(&v.ty) {
                Ty::Param(i) => {
                    (self.bound_traits(i).iter()).any(|b| Some(b.tr) == self.d.convert_trait)
                }
                _ => false,
            };
            if !bounded && matches!(self.shallow(&v.ty), Ty::Dom(_) | Ty::Param(_)) {
                return mk(TK::DomOrd(Box::new(v)), Ty::Int);
            }
        }
        let v = match self.shallow(&v.ty) {
            Ty::Range(..) | Ty::RangeOf(_) => self.coerce(v, &Ty::Int),
            _ => v,
        };
        let conv = self.d.convert_trait.and_then(|tr| {
            let (m, f) = self.d.traits[tr.0].methods.iter().next()?;
            Some((tr, m.clone(), *f))
        });
        let Some((tr, method, f)) = conv else {
            self.err(span, Unbuilt::ConversionWithoutConvert);
            return mk(TK::Error, Ty::Error);
        };
        self.reach(f);
        let r = decls::TraitRef {
            tr,
            args: vec![to.clone()],
        };
        self.obligations.push((v.ty.clone(), r.clone(), span, why));
        mk(
            TK::CallTrait {
                tr: r,
                method,
                self_ty: Box::new(v.ty.clone()),
                targs: Box::default(),
                recv: Some(Box::new(TArg::Value(v))),
                args: Vec::new(),
            },
            to,
        )
    }
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
        want: Option<&Ty>,
    ) -> TExpr {
        // The expected type fixes what the fields cannot: `let r: (u: V, v: V) = (u: 0, v: 2)`.
        // Only the type arguments: the functions a field can hold are still added by the arguments.
        // Where `?S` is wanted the literal is an `S` made present, so it takes `S`'s fields too.
        let is_d = |cx: &Self, t: &Ty| matches!(cx.shallow(t), Ty::Struct(o, ..) if o == d);
        let want = want.and_then(|w| match is_d(self, w) {
            true => Some(w.clone()),
            false => self.optional_of(w).filter(|t| is_d(self, t)),
        });
        if let Some(w) = want {
            let open = (0..sets.len()).map(|_| self.new_set()).collect();
            self.unify(&Ty::Struct(d, targs.clone(), open), &w);
        }
        let s = self.d.s(d);
        let sub = Subst {
            tys: &targs,
            sets: &sets,
        };
        let named = super::expr::named_params(&s.generics, &sub);
        let formals: Vec<Formal> = (s.fields.iter().zip(&s.defaults))
            .map(|((n, t), e)| Formal {
                label: n,
                ty: sub.ty(t),
                mutable: false,
                named_only: false,
                variadic: false,
                default: e
                    .as_ref()
                    .map(|e| (e, s.module, s.item.clone(), named.clone())),
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

    /// `e with f: v` and `(..e, f: v)`: a copy of `e` with those fields replaced (R02.5.3). `e`
    /// is read once and the new values in the order written; each other field is read from `e`,
    /// so its heap fields move (R12.11.4).
    /// `(to: x, ..r without (u, v))`: a call of the member of its form's trait that makes the
    /// rest, met by each record with those fields (R08.4.1).
    pub(super) fn without(
        &mut self,
        prefix: &[(ast::Name, ast::Expr)],
        base: &ast::Expr,
        removed: &[ast::Name],
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let form = (
            prefix.iter().map(|(n, _)| n.text.clone()).collect(),
            removed.iter().map(|n| n.text.clone()).collect(),
        );
        let name = ast::Name {
            text: cpl_next_decls::records::without_member(&form),
            span,
        };
        let callee = ast::Expr {
            kind: ast::ExprKind::Field {
                base: Box::new(base.clone()),
                name,
            },
            span,
        };
        let args = (prefix.iter())
            .map(|(_, e)| ast::Arg {
                name: None,
                mutable: false,
                value: e.clone(),
            })
            .collect();
        let kind = ast::ExprKind::Call {
            callee: Box::new(callee),
            args,
        };
        self.expr(&ast::Expr { kind, span }, want)
    }

    pub(super) fn record_update(
        &mut self,
        base: &ast::Expr,
        updates: &[(ast::Name, ast::Expr)],
        span: Span,
    ) -> TExpr {
        let error = TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
        };
        let b = self.expr(base, None);
        if self.shallow(&b.ty) == Ty::Error {
            return error;
        }
        let Some((s, tys)) = self.field_tys(&b.ty) else {
            let code = cpl_diag::codes::TYPE_MISMATCH;
            let t = self.show(&b.ty);
            self.err(
                span,
                Msg::verdict(code, format!("`with` updates a struct, not a `{t}`")),
            );
            return error;
        };
        let names: Vec<String> = self.d.s(s).fields.iter().map(|(n, _)| n.clone()).collect();
        let ty = b.ty.clone();
        let (mut pre, read) = self.held(b);
        let mut given: Vec<Option<TExpr>> = vec![None; names.len()];
        for (n, e) in updates {
            let Some(k) = names.iter().position(|m| *m == n.text) else {
                let code = cpl_diag::codes::UNKNOWN_FIELD;
                let t = self.show(&ty);
                self.err(
                    n.span,
                    Msg::verdict(code, format!("`{t}` has no field `{}`", n.text)),
                );
                continue;
            };
            let v = self.expr(e, Some(&tys[k]));
            let v = self.coerce(v, &tys[k]);
            let tmp = self.bind("$with", tys[k].clone());
            pre.push(TStmt::Let(tmp, v));
            let v = TExpr {
                kind: TK::Var(tmp),
                ty: tys[k].clone(),
                span,
            };
            if given[k].replace(v).is_some() {
                let code = cpl_diag::codes::DUPLICATE_DEFINITION;
                self.err(
                    n.span,
                    Msg::verdict(code, format!("field `{}` is given twice", n.text)),
                );
            }
        }
        let fields = (given.into_iter().zip(tys).enumerate())
            .map(|(k, (v, t))| {
                v.unwrap_or_else(|| TExpr {
                    kind: TK::Field(Box::new(read.clone()), k),
                    ty: t,
                    span,
                })
            })
            .collect();
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
            self.err(span, Unbuilt::FnValueInVariant);
        }
        let payload = self.construct(p, targs.clone(), Vec::new(), args, span, None);
        TExpr {
            kind: TK::Variant(i, Box::new(payload)),
            ty: Ty::Struct(e, targs, Vec::new()),
            span,
        }
    }

    /// A bare variant name `n` as a value: the expected enum's variant, or else the variant of the
    /// one enum in scope that has it, where nothing else takes the name (R02.7.1).
    pub(super) fn bare_variant(&self, n: &str, want: Option<&Ty>) -> Option<(StructId, usize)> {
        self.expected_variant(n, want)
            .or_else(|| self.scoped_variant(n))
    }
    fn nearer(&self, n: &str) -> bool {
        !n.starts_with(|c: char| c.is_ascii_uppercase())
            || self.lookup(n).is_some()
            || self.itself(n, Span::new(0, 0)).is_some()
            || self.self_field(n).is_some()
    }
    pub(super) fn expected_variant(&self, n: &str, want: Option<&Ty>) -> Option<(StructId, usize)> {
        let Some(Ty::Struct(e, ..)) = want.map(|t| self.shallow(t)) else {
            return None;
        };
        let i = (self.d.s(e).variants.iter()).position(|(v, _)| v == n)?;
        (!self.nearer(n)).then_some((e, i))
    }
    pub(super) fn scoped_variant(&self, n: &str) -> Option<(StructId, usize)> {
        if self.nearer(n)
            || self.domain_named(n).is_some()
            || matches!(
                self.d.named(self.module, n),
                Some(decls::Named::Fn(_) | decls::Named::Struct(_) | decls::Named::Const(_))
            )
        {
            return None;
        }
        let mut found: Vec<(StructId, usize)> = (self.d.views[self.module.0].values())
            .filter_map(|x| match x {
                decls::Named::Struct(e) => Some((
                    *e,
                    (self.d.s(*e).variants.iter()).position(|(v, _)| v == n)?,
                )),
                _ => None,
            })
            .collect();
        found.sort_by_key(|f| f.0 .0);
        found.dedup();
        match found[..] {
            [one] => Some(one),
            _ => None,
        }
    }

    /// `match x: arms`: the first arm whose pattern `x` matches and whose guard holds, with the
    /// names it binds in scope; every value is met by an arm (R02.7.2, R02.7.4, R01.10).
    pub(super) fn match_(
        &mut self,
        scrut: &ast::Expr,
        arms: &[ast::Arm],
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        use super::pattern::Space;
        let mk = |kind, ty| TExpr { kind, ty, span };
        let x = self.expr(scrut, None);
        let xt = self.shallow(&x.ty);
        self.scopes.push(HashMap::new());
        let (stmts, whole) = self.held(x);
        // A value of no known type has no arms to count.
        let known = !matches!(xt, Ty::Error | Ty::Var(_));
        let mut counted = known;
        let mut ty = want.cloned();
        let was_joining = self.joining;
        let mut rows: Vec<Vec<Space>> = Vec::new();
        let mut built: Vec<(Option<TExpr>, bool, TExpr)> = Vec::new();
        for arm in arms {
            self.scopes.push(HashMap::new());
            let mut binds = Vec::new();
            let seen = self.errors.len();
            let test = self.refute(&arm.pat, &whole, false, &mut binds);
            // A pattern in error says nothing of which values the arms meet.
            counted &= self.errors.len() == seen;
            let space = self.space(&arm.pat, &xt);
            if known
                && self
                    .useful(
                        &rows,
                        std::slice::from_ref(&space),
                        std::slice::from_ref(&xt),
                    )
                    .is_none()
            {
                self.err(
                    arm.pat.span,
                    Msg::verdict(
                        cpl_diag::codes::UNREACHABLE_ARM,
                        "this arm never runs: the arms before it match everything it does",
                    ),
                );
            }
            let guard = arm.guard.as_ref().map(|g| self.bool_cond(g));
            let body = self.block(&arm.body, ty.as_ref());
            self.scopes.pop();
            // An arm that leaves gives no value, so it says nothing of the match's type.
            let body = match &ty {
                _ if arm.body.leaves() => body,
                Some(t) if self.shallow(t) != Ty::Unit => self.coerce(body, t),
                Some(_) => body,
                // With no expected type, the arms meet in a type of their own, as an `if`'s do.
                None => {
                    let j = self.join_ty(&body.ty.clone());
                    self.joining = true;
                    ty = Some(j.clone());
                    if self.shallow(&j) != Ty::Unit {
                        self.coerce(body, &j)
                    } else {
                        body
                    }
                }
            };
            let bt = body.ty.clone();
            // A guard sees the names, so they are bound before it, in the test.
            built.push(match guard {
                None => {
                    rows.push(vec![space]);
                    (test, false, mk(TK::Block(binds, Some(Box::new(body))), bt))
                }
                Some(g) => {
                    let g = mk(TK::Block(binds, Some(Box::new(g))), Ty::Bool);
                    (Self::both(test, Some(g)), true, body)
                }
            });
        }
        self.joining = was_joining;
        let ty = ty.unwrap_or(Ty::Unit);
        let missing = counted
            .then(|| self.useful(&rows, &[Space::Any], std::slice::from_ref(&xt)))
            .flatten();
        let mut tail = match missing {
            Some(w) => {
                let w = self.show_space(&w[0], &xt);
                let code = cpl_diag::codes::NON_EXHAUSTIVE_MATCH;
                self.err(span, Msg::verdict(code, format!("`match` misses {w}")));
                mk(TK::Error, ty.clone())
            }
            // Every value is met, so the last arm needs no test when it has no guard.
            None => match built.pop() {
                Some((_, false, body)) => body,
                Some((test, true, body)) => {
                    let test = test.unwrap_or_else(|| mk(TK::Bool(true), Ty::Bool));
                    let none = mk(TK::Error, ty.clone());
                    mk(
                        TK::If(Box::new(test), Box::new(body), Box::new(none)),
                        ty.clone(),
                    )
                }
                None => mk(TK::Error, ty.clone()),
            },
        };
        for (test, _, body) in built.into_iter().rev() {
            tail = match test {
                Some(t) => mk(
                    TK::If(Box::new(t), Box::new(body), Box::new(tail)),
                    ty.clone(),
                ),
                None => body,
            };
        }
        self.scopes.pop();
        mk(TK::Block(stmts, Some(Box::new(tail))), ty)
    }
}
