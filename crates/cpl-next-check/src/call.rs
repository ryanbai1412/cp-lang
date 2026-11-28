//! A call of a name: a fn, a type's constructor, a conversion, or an alias of a type (R02.6.3).
use super::expr::{block, pure, Formal};
use super::*;
use cpl_next_ir::unbuilt::Unbuilt;

/// One argument of a call: written (`f(x)`, `f(k: x)`, `f(mut x)`), or given by a form that
/// calls a member for the program, as `..r without (u, v)` does.
pub(super) struct CallArg {
    pub label: Option<String>,
    pub value: Option<next::Expr>,
    pub mutable: bool,
    pub at: Span,
}

impl CallArg {
    pub(super) fn of(cx: &Cx, a: &next::Arg) -> CallArg {
        CallArg {
            label: a.label().map(|t| t.text().to_string()),
            value: a.value(),
            mutable: a.is_mut(),
            at: cx.at(a),
        }
    }
    /// An argument written without a label: `x` of `f(x)`.
    pub(super) fn plain(cx: &Cx, x: &next::Expr) -> CallArg {
        CallArg {
            label: None,
            value: Some(x.clone()),
            mutable: false,
            at: cx.at(x),
        }
    }
}

impl Cx<'_> {
    /// A call of a declared fn or struct, with the type arguments written at the call.
    pub(super) fn call_named(
        &mut self,
        n: &str,
        written: &[ast::Type],
        args: &[ast::Arg],
        span: Span,
        want: Option<&Ty>,
    ) -> TExpr {
        let Some(named) = self.d.named(self.b.module, n) else {
            self.err(
                span,
                Msg::internal(format!(
                    "`{n}` is called as declared, but its module's scope has no `{n}`"
                )),
            );
            return Self::error_node(span);
        };
        self.call_of(n, named, written, args, span, want)
    }
    /// A call of `named`, which the call writes as `n`.
    pub(super) fn call_of(
        &mut self,
        n: &str,
        mut named: Named,
        written: &[ast::Type],
        args: &[ast::Arg],
        span: Span,
        want: Option<&Ty>,
    ) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        if let Named::Fn(first) = named {
            let given: Vec<&str> = (args.iter())
                .filter_map(|a| Some(a.name.as_ref()?.text.as_str()))
                .collect();
            named = Named::Fn(self.d.overload(first, &given));
        }
        // `Pair<A, B>(..)`: the struct of that name with as many parameters (R02.11.4).
        if let Named::Struct(d) = named {
            named = Named::Struct(self.d.by_arity(d, written.len()));
        }
        let mut pre = Vec::new();
        let mut given: Vec<Ty> = written
            .iter()
            .map(|t| self.body_ty(t, span, &mut pre))
            .collect();
        // `type Z = Mod<7>`: `Z(x)` is a call of the type the alias names, as `Mod<7>(x)` is.
        if let Named::Alias(id) = named {
            // A parameter not written is inferred, as a struct's is: `AliasBox(42)`.
            let missing = (self.d.alias(id).generics.len()).saturating_sub(written.len());
            let holes: Vec<String> = (0..missing).map(|k| format!("$alias{k}")).collect();
            let mut targs = written.to_vec();
            targs.extend(holes.iter().map(|h| ast::Type {
                kind: ast::TypeKind::Named {
                    name: ast::Name {
                        text: h.clone(),
                        span,
                    },
                    args: Vec::new(),
                },
                span,
            }));
            let t = ast::Type {
                kind: ast::TypeKind::Named {
                    name: ast::Name {
                        text: n.to_string(),
                        span,
                    },
                    args: targs,
                },
                span,
            };
            match self.body_ty_open(&t, &holes, span, &mut pre) {
                Ty::Struct(d, targs, _) => {
                    named = Named::Struct(d);
                    given = targs;
                }
                Ty::Error => return mk(TK::Error, Ty::Error),
                to if matches!(args, [a] if a.name.is_none()) && decls::prim_type(n).is_none() => {
                    let v = self.convert(to, &args[0].value, span, Origin::Convert);
                    return block(pre, v);
                }
                _ => {}
            }
        }
        let takes = match named {
            Named::Fn(d) => self.d.f(d).generics.len(),
            Named::Struct(d) => self.d.s(d).generics.len(),
            _ => 0,
        };
        if given.len() > takes {
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::WRONG_ARG_COUNT,
                    format!("`{n}` takes {takes} type arguments, found {}", given.len()),
                ),
            );
        }
        given.truncate(takes);
        if let Named::Struct(d) = named {
            given = self.d.with_defaults_of(d, given);
        }
        let ranged = match named {
            Named::Fn(d) => self.ranged_targ(d, &mut given),
            _ => None,
        };
        let ctor = match named {
            Named::Struct(d) => self.d.s(d).ctor.filter(|_| !match self
                .b
                .decl
                .and_then(|f| self.d.f(f).owner)
            {
                Some(decls::Owner::Struct(o)) => o == d,
                Some(decls::Owner::Impl(i)) => {
                    let im = self.impls().imp(i);
                    im.derived && matches!(im.for_, Ty::Struct(o, ..) if o == d)
                }
                _ => false,
            }),
            _ => None,
        };
        let v = match (ctor, named) {
            // `List(s)`: a `new` that takes nothing builds nothing from `s`, so `T(x)` is `x`'s
            // conversion to `T`, by its impl of the `@role(convert)` trait (R07.8.1, R04.2.28).
            (Some(new), Named::Struct(d))
                if self.d.f(new).params.is_empty() && matches!(args, [a] if a.name.is_none()) =>
            {
                let (n, m) = (self.d.s(d).generics.len(), self.d.s(d).n_sets);
                let mut targs = given;
                while targs.len() < n {
                    targs.push(self.new_ty());
                }
                let sets = (0..m).map(|_| self.new_set()).collect();
                self.convert(
                    Ty::Struct(d, targs, sets),
                    &args[0].value,
                    span,
                    Origin::Construct,
                )
            }
            // R02.6.3: outside its own body, `T(args)` calls `T`'s static `new` when it has one.
            (Some(new), _) => {
                let f = self.fnref(new, given, Vec::new());
                self.call_fn(f, None, args, span, want)
            }
            (None, Named::Fn(d)) => {
                let f = self.fnref(d, given, Vec::new());
                let fd = self.d.f(d);
                let tests;
                let args = match super::lambda::tests_form(fd, args) {
                    Some(a) => {
                        tests = a;
                        &tests[..]
                    }
                    None => args,
                };
                // A free fn's `self` is its first argument when called by name (R08.1.4).
                let first = args.first().filter(|a| a.name.is_none());
                match (fd.owner, fd.self_ty.clone(), first) {
                    (None, Some(st), Some(a)) if fd.self_mode != SelfMode::None => {
                        let sub = Subst {
                            tys: &f.targs,
                            sets: &f.sets,
                        };
                        let st = sub.ty(&st);
                        let st = self.fresh_sets(&st);
                        let e = self.expr(&a.value, Some(&st));
                        let e = self.coerce(e, &st);
                        let recv = self.recv_arg(d, e, span);
                        self.call_fn(f, Some(recv), &args[1..], span, want)
                    }
                    _ => self.call_fn(f, None, args, span, want),
                }
            }
            (None, Named::Struct(d))
                if self
                    .plain_enum(&Ty::Struct(d, Vec::new(), Vec::new()))
                    .is_some()
                    && self.d.s(d).generics.is_empty()
                    && matches!(args, [a] if a.name.is_none()) =>
            {
                self.convert(
                    Ty::Struct(d, Vec::new(), Vec::new()),
                    &args[0].value,
                    span,
                    Origin::Convert,
                )
            }
            (None, Named::Struct(d)) => {
                let s = self.d.s(d);
                let fresh = s.generics.len() - given.len();
                let mut targs = given;
                targs.extend((0..fresh).map(|_| self.new_ty()));
                let sets: Vec<SetTy> = (0..s.n_sets).map(|_| self.new_set()).collect();
                self.construct(d, targs, sets, args, span, want)
            }
            (None, Named::Alias(_)) => {
                let name = n.to_string();
                self.err(span, Unbuilt::AliasCall { name });
                mk(TK::Error, Ty::Error)
            }
            _ => {
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::NOT_CALLABLE,
                        format!("`{n}` cannot be called"),
                    ),
                );
                mk(TK::Error, Ty::Error)
            }
        };
        let v = self.held_to(v, ranged);
        if pre.is_empty() {
            v
        } else {
            let ty = v.ty.clone();
            mk(TK::Block(pre, Some(Box::new(v))), ty)
        }
    }
    /// How a method takes its receiver: a `mut self` (or a cache it fills) needs a place.
    /// A method call; a getter with nothing to pass is read like a field, so `()` after it calls
    /// what it reads (R01.8.7, R02.6.6).
    pub(super) fn call_method(
        &mut self,
        f: FnRef,
        recv: TArg,
        getter: bool,
        call: Option<&[ast::Arg]>,
        span: Span,
        want: Option<&Ty>,
    ) -> TExpr {
        match call {
            Some(args) if getter => {
                let v = self.call_fn(f, Some(recv), &[], span, None);
                self.call_value(v, args, span)
            }
            _ => self.call_fn(f, Some(recv), call.unwrap_or(&[]), span, want),
        }
    }
    /// The receiver of a method that changes it, lent `mut`: never a `let` binding (R05.4.2).
    pub(super) fn mut_recv(&mut self, recv: TExpr, span: Span) -> TArg {
        if let Some(name) = self.let_root(&recv) {
            let msg = Msg::verdict(
                cpl_diag::codes::MUT_OF_IMMUTABLE,
                format!(
                    "`{name}` is bound by `let`, so it cannot be lent `mut`; declare it with `var`"
                ),
            );
            self.err(span, self.with_var_fix(&recv, &name, msg));
        }
        TArg::Mut(recv)
    }
    pub(super) fn recv_arg(&mut self, m: FnId, recv: TExpr, span: Span) -> TArg {
        let place = matches!(recv.kind, TK::Var(_) | TK::Field(..));
        if self.d.f(m).self_mode == SelfMode::Mut {
            if !place && !self.is_error(&recv.ty) {
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::MUT_OF_IMMUTABLE,
                        "a `mut self` method needs a place",
                    ),
                );
            }
            self.mut_recv(recv, span)
        } else if self.d.writes_cache(m) && place {
            TArg::Mut(recv)
        } else {
            TArg::Value(recv)
        }
    }
    /// `@same_keys`: arrays over two different domains are not paired by position (R04b.1.3).
    fn same_keys(&mut self, args: &[TArg], span: Span) {
        let mut first: Option<usize> = None;
        for a in args {
            let (TArg::Value(e) | TArg::Mut(e)) = a;
            let Ty::Struct(d, ts, _) = self.shallow(&e.ty) else {
                continue;
            };
            let index = if Some(d) == self.array_id() {
                self.shallow(&ts[0])
            } else {
                Ty::Error
            };
            let Ty::Dom(i) = index else {
                continue;
            };
            match first {
                None => first = Some(i),
                Some(f) if self.domains[f].name != self.domains[i].name || f != i => {
                    let (a, b) = (self.domains[f].name.clone(), self.domains[i].name.clone());
                    self.err(
                        span,
                        Msg::verdict(cpl_diag::codes::WRONG_DOMAIN, format!("an array over `{b}` is paired with one over `{a}`: their positions are unrelated")),
                    );
                    return;
                }
                Some(_) => {}
            }
        }
    }
    pub(super) fn call_fn(
        &mut self,
        f: FnRef,
        recv: Option<TArg>,
        args: &[ast::Arg],
        span: Span,
        want: Option<&Ty>,
    ) -> TExpr {
        let fd = self.d.f(f.decl);
        // A call of a fn refused as declared is downstream of that refusal: it is the error.
        if (fd.errors.iter()).any(|e| e.refusal.is_some()) {
            return Self::error_node(span);
        }
        let sub = Subst {
            tys: &f.targs,
            sets: &f.sets,
        };
        // The expected type fixes type arguments the arguments cannot: `let xs: List<Int> = List.new()`.
        let ty = self.ret_ty(&f);
        if let Some(w) = want {
            let orig = self.shallow(w);
            // A slot of a range with ends known as the program runs, alone or a tuple's part,
            // takes an `Int` and checks it as it is stored (R03.1.4); one with constant ends is
            // the type argument (`ranged_targ`).
            let made = super::unify::made_param(fd).is_some();
            let w = (made && self.runtime_ranged(w))
                .then(|| self.int_parts(w))
                .flatten()
                .unwrap_or_else(|| w.clone());
            match self.shallow(&w) {
                Ty::Struct(..) | Ty::Raw(_)
                    if matches!(fd.ret, Ty::Struct(..) | Ty::Param(_) | Ty::Raw(_)) =>
                {
                    self.unify(&ty, &w);
                }
                Ty::Range(..) if made => {
                    self.unify(&ty, &w);
                }
                _ if matches!(orig, Ty::Range(..) | Ty::RangeOf(_) | Ty::RangeTo(..))
                    && matches!(fd.ret, Ty::Param(_)) =>
                {
                    self.unify(&ty, &Ty::Int);
                }
                _ => {}
            }
        }
        let mut formals = self.formals(&sub, fd);
        for p in &mut formals {
            p.ty = self.fresh_sets(&p.ty);
        }
        let held = fd.region.is_some() && !std::mem::take(&mut self.b.allow_region);
        let unbuilt =
            held && (fd.region.as_ref().is_some_and(|r| r.0) || self.d.roles().recipe.is_none());
        if unbuilt {
            self.err(
                span,
                Unbuilt::GeneratorAsValue {
                    name: fd.name.clone(),
                },
            );
        }
        self.oblige(&f, span);
        self.solve(false);
        let seen = self.errors.len();
        let (mut pre, args) = self.args(args, &formals, span);
        // A call its arguments do not fit was diagnosed there, and one not built is refused:
        // its value is the error, and the bounds its type arguments would have met are not asked.
        if unbuilt || self.errors.len() > seen {
            self.b.obligations.retain(|o| o.2 != span);
            return Self::error_node(span);
        }
        self.solve(false);
        if fd.same_keys {
            self.same_keys(&args, span);
        }
        let recv = match recv {
            Some(TArg::Value(r)) if !pre.is_empty() && !pure(&r) => {
                let tmp = self.bind("$recv", r.ty.clone());
                let read = TExpr {
                    kind: TK::Var(tmp),
                    ty: r.ty.clone(),
                    span: r.span,
                };
                pre.insert(0, TStmt::Let(tmp, r));
                Some(TArg::Value(read))
            }
            r => r,
        };
        let call = TExpr {
            kind: TK::Call {
                f,
                recv: recv.map(Box::new),
                args,
            },
            ty,
            span,
        };
        if held && self.d.roles().recipe.is_some() {
            return self.recipe(pre, call);
        }
        block(pre, call)
    }
    /// `recv.name(args)` of a trait bound on the type parameter `recv` is.
    pub(super) fn bound_call(
        &mut self,
        p: usize,
        recv: TExpr,
        name: &str,
        args: Vec<TArg>,
        span: Span,
    ) -> TExpr {
        for tr in &self.bound_traits(p) {
            let Some(m) = self.d.member(&self.d.t(tr.tr).methods, name) else {
                continue;
            };
            // A lender is called only where its place is used (`for`, an assignment): elsewhere
            // the bound's reading fn of that name is meant (`IndexedMut.at` vs `Indexed.at`).
            if self.d.f(m).region.is_some() && !self.b.allow_region {
                continue;
            }
            self.reach(m);
            let mut tys = vec![recv.ty.clone()];
            tys.extend(tr.args.iter().cloned());
            let ret = self.trait_ret(m, tys);
            return TExpr {
                kind: TK::CallTrait {
                    ctx: self.trait_ctx(tr.tr, name),
                    tr: tr.clone(),
                    method: name.to_string(),
                    self_ty: Box::new(recv.ty.clone()),
                    targs: Box::default(),
                    recv: Some(Box::new(TArg::Value(recv))),
                    args,
                },
                ty: ret,
                span,
            };
        }
        let shown = self.show(&recv.ty);
        self.err(
            span,
            Msg::verdict(
                cpl_diag::codes::UNKNOWN_FIELD,
                format!("`{shown}` has no `{name}`"),
            ),
        );
        TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
        }
    }
    pub(super) fn call_value(&mut self, callee: TExpr, args: &[ast::Arg], span: Span) -> TExpr {
        let (ps, r) = match self.shallow(&callee.ty) {
            Ty::Fn(ps, r, _) => (ps, *r),
            Ty::Clo(c) => self.clo_sig(&c),
            Ty::Error => {
                return TExpr {
                    kind: TK::Error,
                    ty: Ty::Error,
                    span,
                };
            }
            t => {
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::NOT_CALLABLE,
                        format!("`{}` cannot be called", self.show(&t)),
                    ),
                );
                return TExpr {
                    kind: TK::Error,
                    ty: Ty::Error,
                    span,
                };
            }
        };
        let sig = match self.shallow(&callee.ty) {
            Ty::Clo(c) => match &*c {
                CloRef::Lambda(id, ..) => self.lambdas[id.0].written.clone(),
                CloRef::Item(_) => Vec::new(),
            },
            _ => Vec::new(),
        };
        // A fn item's value is the item: a call through it binds as a direct call does, its
        // defaults and variadic tail included (R06.1.5).
        let item = match self.shallow(&callee.ty) {
            Ty::Clo(c) => match *c {
                CloRef::Item(f) => Some(f),
                CloRef::Lambda(..) => None,
            },
            _ => None,
        };
        let d = self.d;
        let formals: Vec<Formal> = match &item {
            Some(f) if (d.f(f.decl).params.iter()).any(|p| p.variadic || p.default.is_some()) => {
                let sub = Subst {
                    tys: &f.targs,
                    sets: &f.sets,
                };
                self.formals(&sub, d.f(f.decl))
            }
            _ => (ps.into_iter().enumerate())
                .map(|(k, ty)| {
                    let (ty, mutable) = match ty {
                        Ty::Mut(t) => (*t, true),
                        t => (t, false),
                    };
                    Formal {
                        label: "",
                        name: sig.get(k).map_or("", |s| s.0.as_str()),
                        written: sig.get(k).and_then(|s| s.1.as_ref()),
                        ty,
                        mutable,
                        named_only: false,
                        variadic: false,
                        default: None,
                    }
                })
                .collect(),
        };
        let seen = self.errors.len();
        let (pre, args) = self.args(args, &formals, span);
        if self.errors.len() > seen {
            return Self::error_node(span);
        }
        let call = TExpr {
            kind: TK::CallValue {
                callee: Box::new(callee),
                args,
            },
            ty: r,
            span,
        };
        block(pre, call)
    }
}
