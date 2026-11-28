//! A call of a name: a fn, a type's constructor, a conversion, or an alias of a type (R02.6.3).
use super::expr::{block, named_params, pure};
use super::*;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_syntax::TokenKind;

/// One argument of a call: written (`f(x)`, `f(k: x)`, `f(mut x)`), given by a form that
/// calls a member for the program, as `..r without (u, v)` does, or checked already, as
/// `g[i].m(x)` checks `x` before it lends the cell.
#[derive(Clone)]
pub(super) struct CallArg {
    pub label: Option<String>,
    pub value: ArgValue,
    pub mutable: bool,
    pub at: Span,
}

/// What an argument gives.
#[derive(Clone)]
pub(super) enum ArgValue {
    Written(next::Expr),
    Checked(TExpr),
    /// The parser found it missing and has said so.
    Missing,
}

impl CallArg {
    pub(super) fn of(a: &next::Arg) -> CallArg {
        let label = a.label().map(|t| t.text().to_string());
        CallArg {
            mutable: a.is_mut(),
            ..CallArg::labelled(label, a.value(), a.syntax().span())
        }
    }
    /// An argument written without a label: `x` of `f(x)`.
    pub(super) fn plain(x: &next::Expr) -> CallArg {
        CallArg::labelled(None, Some(x.clone()), x.syntax().span())
    }
    /// An argument a form gives with `label`, the parser's missing value as `None`.
    pub(super) fn labelled(label: Option<String>, value: Option<next::Expr>, at: Span) -> CallArg {
        let value = value.map_or(ArgValue::Missing, ArgValue::Written);
        CallArg {
            label,
            value,
            mutable: false,
            at,
        }
    }
    /// An argument checked already, which the callee coerces as it does any other.
    pub(super) fn held(label: Option<String>, v: TExpr, at: Span) -> CallArg {
        CallArg {
            label,
            value: ArgValue::Checked(v),
            mutable: false,
            at,
        }
    }
    /// Where its value is written: the value, else the whole argument.
    fn value_at(&self) -> Span {
        self.written().map_or(self.at, |x| x.syntax().span())
    }
    /// Its expression, when it was written: what a test of an argument's shape reads.
    pub(super) fn written(&self) -> Option<&next::Expr> {
        match &self.value {
            ArgValue::Written(x) => Some(x),
            ArgValue::Checked(_) | ArgValue::Missing => None,
        }
    }
}

impl Cx<'_> {
    /// The value an argument gives; one the parser found missing is already its error.
    pub(super) fn arg_expr(&mut self, a: &CallArg, want: Option<&Ty>) -> TExpr {
        match &a.value {
            ArgValue::Written(x) => self.expr(x, want),
            ArgValue::Checked(v) => v.clone(),
            ArgValue::Missing => Self::error_node(a.at),
        }
    }
    /// A call of a declared fn or struct, with the type arguments written at the call.
    pub(super) fn call_named(
        &mut self,
        n: &str,
        written: &[next::Type],
        args: &[CallArg],
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
        written: &[next::Type],
        args: &[CallArg],
        span: Span,
        want: Option<&Ty>,
    ) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        if let Named::Fn(first) = named {
            let given: Vec<&str> = (args.iter()).filter_map(|a| a.label.as_deref()).collect();
            named = Named::Fn(self.d.overload(first, &given));
        }
        // `Pair<A, B>(..)`: the struct of that name with as many parameters (R02.11.4).
        if let Named::Struct(d) = named {
            named = Named::Struct(self.d.by_arity(d, written.len()));
        }
        let mut pre = Vec::new();
        let mut given: Vec<Ty> = written.iter().map(|t| self.body_ty(t, &mut pre)).collect();
        // `type Z = Mod<7>`: `Z(x)` is a call of the type the alias names, as `Mod<7>(x)` is.
        if let Named::Alias(id) = named {
            // A parameter not written is inferred, as a struct's is: `AliasBox(42)`.
            let missing = (self.d.alias(id).generics.len()).saturating_sub(written.len());
            let mut targs = given.clone();
            targs.extend((0..missing).map(|_| self.new_ty()));
            let scoped = |n| self.owned_named(n);
            let at = decls::BodyScope {
                sigs: self.d,
                scope: self.d,
                m: self.b.module,
                generics: &self.b.generics,
                doms: &self.b.doms,
                scoped: &scoped,
            };
            let (t, errs) = decls::alias_type(&at, id, targs);
            for e in errs {
                self.err(span, e);
            }
            match self.fresh_sets(&t) {
                Ty::Struct(d, targs, _) => {
                    named = Named::Struct(d);
                    given = targs;
                }
                Ty::Error => return mk(TK::Error, Ty::Error),
                to if matches!(args, [a] if a.label.is_none()) && decls::prim_type(n).is_none() => {
                    let v = self.arg_expr(&args[0], None);
                    let v = self.convert_value(to, v, span, Origin::Convert);
                    return block(pre, v);
                }
                _ => {}
            }
        }
        let takes = match named {
            Named::Fn(d) => self.items.f(d).generics.len(),
            Named::Struct(d) => self.items.s(d).generics.len(),
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
            Named::Struct(d) => self.items.s(d).ctor.filter(|_| !match self
                .b
                .decl
                .and_then(|f| self.items.f(f).owner)
            {
                Some(decls::Owner::Struct(o)) => o == d,
                Some(decls::Owner::Impl(i)) => {
                    let im = self.items.imp(i);
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
                if self.items.f(new).params.is_empty()
                    && matches!(args, [a] if a.label.is_none()) =>
            {
                let (n, m) = (self.items.s(d).generics.len(), self.items.s(d).n_sets);
                let mut targs = given;
                while targs.len() < n {
                    targs.push(self.new_ty());
                }
                let sets = (0..m).map(|_| self.new_set()).collect();
                {
                    let v = self.arg_expr(&args[0], None);
                    self.convert_value(Ty::Struct(d, targs, sets), v, span, Origin::Construct)
                }
            }
            // R02.6.3: outside its own body, `T(args)` calls `T`'s static `new` when it has one.
            (Some(new), _) => {
                let f = self.fnref(new, given, Vec::new());
                self.call_fn(f, None, args, span, want)
            }
            (None, Named::Fn(d)) => {
                let f = self.fnref(d, given, Vec::new());
                let fd = self.items.f(d);
                let tests;
                let args = match tests_form(fd, args) {
                    Some(t) => {
                        tests = self.tests_args(&f, fd, t, span);
                        &tests[..]
                    }
                    None => args,
                };
                // A free fn's `self` is its first argument when called by name (R08.1.4).
                let first = args.first().filter(|a| a.label.is_none());
                match (fd.owner, fd.self_ty.clone(), first) {
                    (None, Some(st), Some(a)) if fd.self_mode != SelfMode::None => {
                        let sub = Subst {
                            tys: &f.targs,
                            sets: &f.sets,
                        };
                        let st = sub.ty(&st);
                        let st = self.fresh_sets(&st);
                        let e = self.arg_expr(a, Some(&st));
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
                    && self.items.s(d).generics.is_empty()
                    && matches!(args, [a] if a.label.is_none()) =>
            {
                let v = self.arg_expr(&args[0], None);
                self.convert_value(
                    Ty::Struct(d, Vec::new(), Vec::new()),
                    v,
                    span,
                    Origin::Convert,
                )
            }
            (None, Named::Struct(d)) => {
                let s = self.items.s(d);
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
        call: Option<&[CallArg]>,
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
        if self.items.f(m).self_mode == SelfMode::Mut {
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
        args: &[CallArg],
        span: Span,
        want: Option<&Ty>,
    ) -> TExpr {
        let fd = self.items.f(f.decl);
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
                _ if orig.is_int_range() && matches!(fd.ret, Ty::Param(_)) => {
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
                let tmp = self.temp("$recv", r.ty.clone());
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
            let Some(m) = self.d.member(&self.items.t(tr.tr).methods, name) else {
                continue;
            };
            // A lender is called only where its place is used (`for`, an assignment): elsewhere
            // the bound's reading fn of that name is meant (`IndexedMut.at` vs `Indexed.at`).
            if self.items.f(m).region.is_some() && !self.b.allow_region {
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
    pub(super) fn call_value(&mut self, callee: TExpr, args: &[CallArg], span: Span) -> TExpr {
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
                        written: sig.get(k).and_then(|s| s.1.clone()),
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

/// A parameter as a call binds it.
pub(super) struct Formal<'d> {
    pub label: &'d str,
    /// What its fn's body calls it.
    pub name: &'d str,
    /// A jagged parameter's written range (`k: 0..=n`), read with each earlier parameter's name
    /// bound to its argument.
    pub written: Option<next::Type>,
    pub ty: Ty,
    pub mutable: bool,
    pub named_only: bool,
    pub variadic: bool,
    /// Its expression, read in the module and with spans from the item that declares it, and
    /// the types the declaration's type parameters stand for at this call.
    pub default: Option<DefaultAt<'d>>,
}

/// A default's expression, its module and item, and its type parameters' types at the call.
pub(super) type DefaultAt<'d> = (&'d next::Expr, ModId, ItemId, Vec<(Sym, Ty)>);

/// `all(x > 1 for x in xs)` as the call it means: what is walked, and the test of each `x`.
pub(super) struct TestsForm {
    pub iter: next::Expr,
    pub pat: next::Pattern,
    pub test: next::Expr,
}

/// `all(x > 1 for x in xs)` passed to a fn whose next parameter is a test is `all(xs, x => x > 1)`:
/// a generator of tests is the predicate (R01.10.1, decision 53).
pub(super) fn tests_form(fd: &decls::FnDecl, args: &[CallArg]) -> Option<TestsForm> {
    let [a] = args else { return None };
    let Some(next::Expr::Generator(g)) = a.written() else {
        return None;
    };
    let clauses: Vec<next::CompClause> = g.clauses().collect();
    let [c] = &clauses[..] else { return None };
    let test = matches!(fd.params.first().map(|p| &p.ty), Some(Ty::Fn(ps, r, _)) if ps.len() == 1 && **r == Ty::Bool);
    if !(fd.self_mode != SelfMode::None && test && c.filters().next().is_none()) {
        return None;
    }
    Some(TestsForm {
        iter: c.iter()?,
        pat: c.pattern()?,
        test: g.body()?,
    })
}

/// An integer literal, maybe negated or in parentheses: `3`, `-1`, `1e9`.
fn int_literal(x: &next::Expr) -> bool {
    match x {
        next::Expr::Literal(l) => l.token().is_some_and(|t| match t.kind() {
            TokenKind::Int => true,
            TokenKind::Float => (t.text().split_once(['e', 'E']))
                .is_some_and(|(m, e)| m.parse::<i64>().is_ok() && e.parse::<u32>().is_ok()),
            _ => false,
        }),
        next::Expr::Paren(p) => p.inner().is_some_and(|x| int_literal(&x)),
        next::Expr::Prefix(p) => {
            matches!(p.op_kind(), Some(cpl_next_syntax::ops::UnOp::Neg))
                && p.operand().is_some_and(|x| int_literal(&x))
        }
        _ => false,
    }
}

/// A lambda with a parameter whose type is not written.
fn open_lambda(x: &next::Expr) -> bool {
    let next::Expr::Lambda(l) = x else {
        return false;
    };
    (l.params()).is_some_and(|ps| ps.lambda_params().iter().any(|p| p.ty.is_none()))
}

impl Cx<'_> {
    /// `n(x for x in xs, ..)` where `n` is a free fn with no `self`.
    pub(super) fn fold_form(&self, n: &str, args: &[CallArg]) -> bool {
        args.first().is_some_and(|a| {
            a.label.is_none() && matches!(a.written(), Some(next::Expr::Generator(_)))
        }) && matches!(
            self.d.named(self.b.module, n),
            Some(decls::Named::Fn(f)) if self.items.f(f).self_ty.is_none()
        )
    }
    /// The parts as one tuple: past the widest tuple, the last part is a tuple of the rest,
    /// nested as far as needed.
    fn nest_parts(&mut self, mut parts: Vec<CallArg>, at: Span) -> TExpr {
        if parts.is_empty() {
            return TExpr {
                kind: TK::Unit,
                ty: Ty::Unit,
                span: at,
            };
        }
        let widest = || (2..parts.len()).rev().find(|&n| self.d.tuple(n).is_some());
        if let Some(wide) = self.d.tuple(parts.len()).map_or_else(widest, |_| None) {
            let rest = parts.split_off(wide - 1);
            let rest_at = Span::new(rest[0].at.start, rest[rest.len() - 1].at.end);
            let nested = self.nest_parts(rest, rest_at);
            parts.push(CallArg::held(None, nested, rest_at));
        }
        let Some(d) = self.d.tuple(parts.len()) else {
            let kind = "Tuple".to_string();
            self.err(at, Unbuilt::Expression { kind });
            return Self::error_node(at);
        };
        let targs = (0..parts.len()).map(|_| self.new_ty()).collect();
        let sets = (0..self.items.s(d).n_sets)
            .map(|_| self.new_set())
            .collect();
        self.construct(d, targs, sets, &parts, at, None)
    }
    /// The arguments past the others gathered into the one `..xs` parameter takes: a
    /// tuple when positional, a record when named, in written order (R06.1.5, decision 324).
    fn pack_variadic(
        &mut self,
        args: &[CallArg],
        formals: &[Formal<'_>],
        span: Span,
    ) -> Option<Vec<CallArg>> {
        let at = formals.iter().position(|f| f.variadic)?;
        let v = &formals[at];
        let fixed: Vec<&Formal> = formals.iter().filter(|f| !f.variadic).collect();
        let mut positional = formals[..at].iter().filter(|f| !f.named_only).count();
        let (mut out, mut tuple, mut record) = (Vec::new(), Vec::new(), Vec::new());
        for a in args {
            match &a.label {
                None if positional > 0 => {
                    positional -= 1;
                    out.push(a.clone());
                }
                None => tuple.push(a.clone()),
                Some(n) if fixed.iter().any(|f| f.label == n) => out.push(a.clone()),
                Some(_) => record.push(a.clone()),
            }
        }
        let at = (tuple.iter().chain(&record))
            .map(CallArg::value_at)
            .reduce(|a, b| Span::new(a.start, b.end))
            .unwrap_or(span);
        let packed = match (tuple.is_empty(), record.is_empty()) {
            (_, true) => self.nest_parts(tuple, at),
            (true, false) => {
                let names: Vec<&str> = record.iter().filter_map(|a| a.label.as_deref()).collect();
                let callee = crate::records::name(&names);
                self.call_named(&callee, &[], &record, at, None)
            }
            (false, false) => {
                self.err(
                    at,
                    Msg::verdict(
                        cpl_diag::codes::WRONG_ARG_COUNT,
                        format!(
                            "`..{}` takes positional or named arguments, not both",
                            v.label
                        ),
                    ),
                );
                self.nest_parts(tuple, at)
            }
        };
        out.push(CallArg::held(Some(v.label.to_string()), packed, at));
        Some(out)
    }
    /// The arguments of a call bound to its parameters: positional ones in order, named ones by
    /// label, the rest from their defaults (R06.1.5). In parameter order; what must run first to
    /// keep the written left-to-right order (R05.11) is in the first part.
    pub(super) fn args(
        &mut self,
        args: &[CallArg],
        formals: &[Formal<'_>],
        span: Span,
    ) -> (Vec<TStmt>, Vec<TArg>) {
        let packed;
        let args = match self.pack_variadic(args, formals, span) {
            Some(p) => {
                packed = p;
                &packed[..]
            }
            None => args,
        };
        let seen = self.errors.len();
        let mut slot: Vec<Option<usize>> = vec![None; formals.len()];
        let mut next = 0;
        let mut excess = false;
        for (k, a) in args.iter().enumerate() {
            let at = match &a.label {
                None => {
                    while next < formals.len() && formals[next].named_only {
                        next += 1;
                    }
                    next += 1;
                    excess |= next > formals.len();
                    (next <= formals.len()).then(|| next - 1)
                }
                Some(n) => {
                    let at = formals.iter().position(|f| f.label == n);
                    if at.is_none() {
                        self.err(
                            a.value_at(),
                            Msg::verdict(
                                cpl_diag::codes::UNKNOWN_ARG_NAME,
                                format!("no parameter named `{n}`"),
                            ),
                        );
                    }
                    at
                }
            };
            match at {
                Some(i) if slot[i].is_none() => slot[i] = Some(k),
                Some(i) => self.err(
                    a.value_at(),
                    Msg::verdict(
                        cpl_diag::codes::WRONG_ARG_COUNT,
                        format!("`{}` is given twice", formals[i].label),
                    ),
                ),
                None => {}
            }
        }
        let missing = (0..formals.len()).any(|i| slot[i].is_none() && formals[i].default.is_none());
        if excess || (missing && args.iter().all(|a| a.label.is_none())) {
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::WRONG_ARG_COUNT,
                    format!("expected {} arguments, found {}", formals.len(), args.len()),
                ),
            );
        } else if missing {
            for (i, f) in formals.iter().enumerate() {
                if slot[i].is_none() && f.default.is_none() {
                    self.err(
                        span,
                        Msg::verdict(
                            cpl_diag::codes::WRONG_ARG_COUNT,
                            format!("no argument for `{}`", f.label),
                        ),
                    );
                }
            }
        }
        // Arguments that do not fit the parameters make the call the error: its bounds are not asked.
        if self.errors.len() > seen {
            self.b.obligations.retain(|o| o.2 != span);
        }
        // A lambda with unwritten parameter types is checked after the other arguments, whose
        // types and bounds may fix the ones it takes (R06.1.2).
        // So is an integer literal for a parameter bound by `Domain`: the domain the others fix
        // makes it a position of that domain (R04.2.18).
        let late: Vec<bool> = (args.iter().enumerate())
            .map(|(k, a)| {
                let Some(x) = a.written() else {
                    return false;
                };
                let position = int_literal(x)
                    && (slot.iter().position(|s| *s == Some(k)))
                        .is_some_and(|i| self.domain_var(&formals[i].ty));
                position || open_lambda(x)
            })
            .collect();
        let mut typed: Vec<Option<TArg>> = (0..args.len()).map(|_| None).collect();
        for pass in [false, true] {
            if pass {
                self.solve(false);
            }
            for (k, a) in args.iter().enumerate() {
                if late[k] != pass {
                    continue;
                }
                typed[k] = match slot.iter().position(|s| *s == Some(k)) {
                    Some(i) => Some(self.arg(a, &formals[i].ty, formals[i].mutable)),
                    None => {
                        self.arg_expr(a, None);
                        None
                    }
                };
            }
        }
        // Bound in another order than written, or with a default before a written argument: the
        // written ones that compute something run first, in order.
        let written: Vec<usize> = slot.iter().flatten().copied().collect();
        let last = slot.iter().rposition(Option::is_some);
        let reordered = written.windows(2).any(|w| w[0] > w[1])
            || last.is_some_and(|l| slot[..l].iter().any(Option::is_none));
        let mut pre = Vec::new();
        if reordered {
            for t in typed.iter_mut() {
                // A built struct's parts are read one by one, so a name in it (a variadic
                // pack's) stays the name the struct reads where it is made, not a value moved
                // into a temp.
                let Some(TArg::Value(v)) = t else { continue };
                let mut parts = vec![v];
                while let Some(v) = parts.pop() {
                    if pure(v) {
                        continue;
                    }
                    let v = match v {
                        TExpr {
                            kind: TK::Struct(fs),
                            ..
                        } => {
                            parts.extend(fs.iter_mut().rev());
                            continue;
                        }
                        v => v,
                    };
                    let ty = v.ty.clone();
                    let tmp = self.temp("$arg", ty.clone());
                    let read = TExpr {
                        kind: TK::Var(tmp),
                        ty,
                        span: v.span,
                    };
                    pre.push(TStmt::Let(tmp, std::mem::replace(v, read)));
                }
            }
        }
        let mut out = Vec::with_capacity(formals.len());
        for (i, f) in formals.iter().enumerate() {
            out.push(match slot[i].and_then(|k| typed[k].take()) {
                Some(TArg::Value(v)) if f.variadic => TArg::Value(TExpr {
                    ty: v.ty.clone(),
                    span: v.span,
                    kind: TK::Pack(Box::new(v)),
                }),
                Some(a) => a,
                None => TArg::Value(match f.default {
                    Some((e, m, ref item, ref named)) => {
                        self.default_arg((e, m, item.clone()), named, &f.ty)
                    }
                    None => TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    },
                }),
            });
        }
        if formals.iter().any(|f| f.written.is_some()) {
            self.check_jagged(formals, &mut out, &mut pre);
        }
        (pre, out)
    }
    /// `f(a, b)` for `fn f(n: 0..=30, k: 0..=n)`: each argument is bound to its parameter's
    /// name, so `k`'s written range reads `n` as `a`, and `b` is checked into it here, at the
    /// call, where an ordinary range parameter's argument is (R06.4.1).
    fn check_jagged(&mut self, formals: &[Formal<'_>], out: &mut [TArg], pre: &mut Vec<TStmt>) {
        self.b.scopes.push(HashMap::new());
        for (f, a) in formals.iter().zip(out.iter_mut()) {
            if let (TArg::Value(v), false) = (a, f.variadic) {
                let ty = v.ty.clone();
                let tmp = self.bind(f.name, ty.clone());
                let read = TExpr {
                    kind: TK::Var(tmp),
                    ty,
                    span: v.span,
                };
                pre.push(TStmt::Let(tmp, std::mem::replace(v, read)));
            }
        }
        for (f, a) in formals.iter().zip(out.iter()) {
            let (Some(w), TArg::Value(v)) = (&f.written, a) else {
                continue;
            };
            let ty = self.body_ty_open(w, &[], pre);
            let checked = self.coerce(v.clone(), &ty);
            let tmp = self.temp("$jagged", ty);
            pre.push(TStmt::Let(tmp, checked));
        }
        self.b.scopes.pop();
    }
    fn arg(&mut self, a: &CallArg, pty: &Ty, pmut: bool) -> TArg {
        let x = match &a.value {
            ArgValue::Written(x) => x,
            ArgValue::Checked(v) => {
                if pmut {
                    let text = "an argument checked already is given to a `mut` parameter";
                    self.err(a.at, Msg::internal(text));
                }
                let v = self.lift_item(v.clone(), pty);
                return TArg::Value(self.coerce(v, pty));
            }
            ArgValue::Missing => return TArg::Value(Self::error_node(a.at)),
        };
        let at = self.at(x);
        let marked = matches!(x, next::Expr::Mut(_));
        let inner = match x {
            next::Expr::Mut(m) => m.inner(),
            x => Some(x.clone()),
        };
        let Some(inner) = inner else {
            return TArg::Value(Self::error_node(at));
        };
        if pmut {
            if !a.mutable && !marked {
                self.err(
                    at,
                    Msg::verdict(
                        cpl_diag::codes::MISSING_MUT_AT_CALL,
                        "a `mut` parameter is marked `mut` at the call",
                    ),
                );
            }
            let p = self.place(&inner);
            if let Some(name) = self.let_root(&p) {
                let msg = Msg::verdict(cpl_diag::codes::MUT_OF_IMMUTABLE, format!("`{name}` is bound by `let`, so it cannot be lent `mut`; declare it with `var`"));
                self.err(at, self.with_var_fix(&p, &name, msg));
            }
            if !self.unify(&p.ty, pty) {
                self.err(
                    at,
                    Msg::verdict(
                        cpl_diag::codes::TYPE_MISMATCH,
                        format!(
                            "expected `{}`, found `{}`",
                            self.show(pty),
                            self.show(&p.ty)
                        ),
                    ),
                );
            }
            TArg::Mut(p)
        } else {
            let v = self.expr(&inner, Some(pty));
            let v = self.lift_item(v, pty);
            TArg::Value(self.coerce(v, pty))
        }
    }
    /// A default is read where its fn or struct is declared: none of the caller's names are seen,
    /// and the declaration's type parameters name this call's types (`id: F = F.zero`).
    fn default_arg(
        &mut self,
        (e, m, item): (&next::Expr, ModId, ItemId),
        named: &[(Sym, Ty)],
        ty: &Ty,
    ) -> TExpr {
        // The caller's type parameters stay unnamed but still number a lambda's: `by: T -> K =
        // x => x` read in a generic fn is an instance per the caller's type arguments.
        let blank = self.b.syms.intern(&self.syms, "");
        let unnamed = vec![blank; self.b.generics.len()];
        let module = std::mem::replace(&mut self.b.module, m);
        let item = std::mem::replace(&mut self.b.item, item);
        let scopes = std::mem::replace(&mut self.b.scopes, vec![HashMap::new()]);
        let self_local = self.b.self_local.take();
        let generics = std::mem::replace(&mut self.b.generics, unnamed);
        let mut doms = self.b.doms.clone();
        doms.extend(named.iter().cloned());
        let doms = std::mem::replace(&mut self.b.doms, doms);
        let v = self.expr(e, Some(ty));
        let v = self.coerce(v, ty);
        self.b.module = module;
        self.b.item = item;
        self.b.scopes = scopes;
        self.b.self_local = self_local;
        self.b.generics = generics;
        self.b.doms = doms;
        v
    }
    /// The parameters of a fn as a call binds them.
    pub(super) fn formals<'d>(&self, sub: &Subst, fd: &'d decls::FnDecl) -> Vec<Formal<'d>> {
        let named = named_params(&fd.generics, sub);
        (fd.params.iter())
            .map(|p| Formal {
                label: &p.label,
                name: &p.name,
                written: p.written.clone(),
                ty: sub.ty(&p.ty),
                mutable: p.mutable,
                named_only: p.named_only,
                variadic: p.variadic,
                default: (p.default.as_ref())
                    .map(|e| (e, fd.module, fd.item.clone(), named.clone())),
            })
            .collect()
    }
    /// Whether `n` names the fn filling the `print` or `write` role here (R11.6.1).
    pub(super) fn names_io(&self, n: &str) -> bool {
        let r = &self.d.roles;
        matches!(self.d.named(self.b.module, n), Some(Named::Fn(f)) if [r.print, r.write].contains(&Some(f)))
    }
    /// `print`'s `sep:` and `precision:` from the innermost `with io` where it is written, unless
    /// the call names them (R09.1.2, R10.3.4).
    pub(super) fn io_args(&self, args: &[CallArg]) -> Vec<CallArg> {
        let mut out = args.to_vec();
        let Some(Named::Context(io)) = self.d.named(self.b.module, "io") else {
            return out;
        };
        for field in ["sep", "precision"] {
            let named = |a: &CallArg| a.label.as_deref() == Some(field);
            let set = (self.b.frames.iter().rev()).find(|x| x.ctx == io && x.field == field);
            if let (false, Some(x)) = (args.iter().any(named), set) {
                let at = self.at(&x.value);
                out.push(CallArg::labelled(
                    Some(field.into()),
                    Some(x.value.clone()),
                    at,
                ));
            }
        }
        out
    }
    /// `all(x > 1 for x in xs)`'s arguments: `xs`, then `x => x > 1`, checked once `xs` has
    /// fixed the type of `x`.
    fn tests_args(
        &mut self,
        f: &FnRef,
        fd: &decls::FnDecl,
        t: TestsForm,
        span: Span,
    ) -> Vec<CallArg> {
        let sub = Subst {
            tys: &f.targs,
            sets: &f.sets,
        };
        let iter_at = self.at(&t.iter);
        let mut seq = self.expr(&t.iter, None);
        if let Some(st) = &fd.self_ty {
            let st = sub.ty(st);
            let st = self.fresh_sets(&st);
            seq = self.coerce(seq, &st);
        }
        self.solve(false);
        let want = fd.params.first().map(|p| sub.ty(&p.ty));
        let param = super::lambda::LamParam {
            pat: Some(t.pat),
            ty: None,
            mutable: false,
        };
        let body = super::lambda::LamBody::Arm(next::ArmBody::Expr(t.test));
        let test = self.lambda_of(vec![param], None, body, want.as_ref(), span);
        vec![
            CallArg::held(None, seq, iter_at),
            CallArg::held(None, test, span),
        ]
    }
    /// The declarations' view of this body's names, for a type a body writes.
    pub(super) fn with_body_scope<R>(&self, f: impl FnOnce(&decls::BodyScope) -> R) -> R {
        let scoped = |n: Sym| self.owned_named(n);
        f(&decls::BodyScope {
            sigs: self.d,
            scope: self.d,
            m: self.b.module,
            generics: &self.b.generics,
            doms: &self.b.doms,
            scoped: &scoped,
        })
    }
}
