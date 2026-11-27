//! A call of a name: a fn, a type's constructor, a conversion, or an alias of a type (R02.6.3).
use super::expr::block;
use super::*;
use cpl_next_ir::unbuilt::Unbuilt;

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
        let named = self.d.views[self.module.0][n];
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
        let mut pre = Vec::new();
        let mut given: Vec<Ty> = written
            .iter()
            .map(|t| self.body_ty(t, span, &mut pre))
            .collect();
        // `type Z = Mod<7>`: `Z(x)` is a call of the type the alias names, as `Mod<7>(x)` is.
        if let Named::Alias(id) = named {
            // A parameter not written is inferred, as a struct's is: `AliasBox(42)`.
            let missing = (self.d.aliases[id.0].generics.len()).saturating_sub(written.len());
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
        let ctor = match named {
            Named::Struct(d) => self.d.s(d).ctor.filter(|_| !match self
                .decl
                .and_then(|f| self.d.f(f).owner)
            {
                Some(decls::Owner::Struct(o)) => o == d,
                Some(decls::Owner::Impl(i)) => {
                    let im = &self.d.impls[i];
                    im.derived && matches!(im.for_, Ty::Struct(o, ..) if o == d)
                }
                _ => false,
            }),
            _ => None,
        };
        let v = match (ctor, named) {
            // `List(s)`: a `new` that takes nothing builds nothing from `s`, so `T(x)` is `x`'s
            // conversion to `T`, by its impl of core's `@convert` trait (R07.8.1, R04.2.28).
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
        if pre.is_empty() {
            v
        } else {
            let ty = v.ty.clone();
            mk(TK::Block(pre, Some(Box::new(v))), ty)
        }
    }
}
