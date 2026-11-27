//! Lambdas: pattern parameters, unpacking a tuple or record argument, generators of tests (R06.1).
use super::expr::block;
use super::*;
use std::collections::BTreeSet;

impl Cx<'_> {
    pub(super) fn lambda(
        &mut self,
        params: &[ast::Pattern],
        param_tys: &[Option<ast::Type>],
        ret_ty: Option<&ast::Type>,
        body: &ast::Expr,
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let me = self.recur.take();
        let pre = self.nested.take();
        // A lambda passed where `?(A -> R)` is wanted is that fn, made `Some` by the coercion (R02.4.2).
        let expected = want.map(|w| {
            let w = self.shallow(w);
            match self.optional_of(&w).map(|t| self.shallow(&t)) {
                Some(f @ Ty::Fn(..)) => f,
                _ => w,
            }
        });
        // Given to a type parameter a trait bounds, the lambda is the fn type an impl of it targets.
        let expected = match expected {
            Some(Ty::Var(v)) => self.fn_impl_for(v, params.len()).or(Some(Ty::Var(v))),
            e => e,
        };
        // Given to a type parameter nothing else fixes, unwritten parameters are `Int`s, as a
        // literal is (R08.2.11).
        let open = matches!(&expected, Some(Ty::Var(v)) if !self.obliged(*v));
        let (eps, er) = match &expected {
            Some(Ty::Fn(ps, r, _)) => (Some(ps.clone()), Some((**r).clone())),
            _ if open => (Some(vec![Ty::Int; params.len()]), None),
            _ => (None, None),
        };
        // `(a, b) => ..` passed as a fn of one tuple or record means `((a, b)) => ..` (R06.1.4).
        let whole;
        let (params, param_tys) = match eps.as_deref() {
            Some([one]) if params.len() > 1 && param_tys.iter().all(Option::is_none) => {
                match self.parts(one) {
                    Some(fields) if fields.len() == params.len() => {
                        let fields = fields.into_iter().zip(params.iter().cloned());
                        let kind = PatternKind::Record {
                            fields: (fields.map(|(text, p)| (ast::Name { text, span: p.span }, p)))
                                .collect(),
                            rest: false,
                        };
                        whole = [ast::Pattern { kind, span }];
                        (&whole[..], &[][..])
                    }
                    Some(fields) => {
                        let code = cpl_diag::codes::WRONG_ARG_COUNT.0;
                        let (k, n) = (params.len(), fields.len());
                        self.err(span, format!("{code}: this lambda takes {k} parameters; the value it is passed has {n} parts"));
                        return mk(TK::Error, Ty::Error);
                    }
                    None => (params, param_tys),
                }
            }
            _ => (params, param_tys),
        };
        self.scopes.push(HashMap::new());
        let mut ids = Vec::new();
        let mut ptys = Vec::new();
        // A pattern parameter binds a fresh name; the body opens with `let pattern = name` (R06.1.3).
        let mut opens = Vec::new();
        for (i, p) in params.iter().enumerate() {
            let tmp;
            let (name, shown) = match &p.kind {
                PatternKind::Name(name) => (name, name.as_str()),
                PatternKind::Wildcard | PatternKind::Tuple(_) | PatternKind::Record { .. } => {
                    tmp = format!("$param{}", p.span.start);
                    if !matches!(p.kind, PatternKind::Wildcard) {
                        opens.push((tmp.clone(), p));
                    }
                    (&tmp, "_")
                }
                _ => {
                    self.err(span, "slice: unsupported lambda parameter");
                    continue;
                }
            };
            let ty = match (
                param_tys.get(i).and_then(|t| t.as_ref()),
                eps.as_ref().and_then(|ps| ps.get(i)),
            ) {
                (Some(t), _) => self.written_ty(t, span),
                (None, Some(t)) => t.clone(),
                (None, None) => {
                    self.err(
                        span,
                        format!(
                            "{}: the type of `{shown}` is not known here; write `({shown}: T) => ...`",
                            cpl_diag::codes::INFERENCE_NEEDS_ANNOTATION.0
                        ),
                    );
                    Ty::Error
                }
            };
            ptys.push(ty.clone());
            match self.bind(name, ty) {
                VarRef::Local(id) | VarRef::Global(id) => ids.push(id),
            }
        }
        let saved_ret = self.ret.clone();
        // A lambda is not part of the generator or lender around it: no `yield` or `lend` inside.
        let saved_region = self.region.take();
        let saved_loops = std::mem::take(&mut self.loops);
        let ret = match ret_ty {
            Some(t) => self.written_ty(t, span),
            None => er.unwrap_or_else(|| self.new_ty()),
        };
        self.ret = ret.clone();
        // The body sees its own signature, so a lambda bound by `let` can call itself (R06.2.3).
        // A nested `fn` was made before its block was checked; it also sees its siblings.
        let (id, group) = match pre {
            Some(id) => (id, self.lambdas[id.0].group.clone()),
            None => {
                self.lambdas.push(LambdaInfo {
                    params: ptys.clone(),
                    ret: ret.clone(),
                    owner: self.decl,
                    group: Vec::new(),
                    yields: None,
                });
                (LambdaId(self.lambdas.len() - 1), Vec::new())
            }
        };
        let outer_selves = self.selves.len();
        let depth = self.scopes.len();
        if let Some(name) = me {
            self.selves.push((name, id, depth));
        }
        for (_, name, sib) in &self.group {
            if *sib != id && group.contains(sib) {
                self.selves.push((name.clone(), *sib, depth));
            }
        }
        let mut pre = Vec::new();
        for (tmp, pat) in opens {
            let open = ast::Stmt {
                kind: StmtKind::Let {
                    mutable: false,
                    pat: pat.clone(),
                    ty: None,
                    init: Some(ast::Expr {
                        kind: ExprKind::Name(tmp),
                        span: pat.span,
                    }),
                    otherwise: None,
                    guard: None,
                },
                span: pat.span,
                decorators: Vec::new(),
                public: false,
            };
            self.stmt(&open, &mut pre);
        }
        // A body that ends in `return` has no value of its own to give the result.
        let b = self.expr(body, Some(&ret));
        let b = match &b.kind {
            TK::Block(ss, None) if matches!(ss.last(), Some(TStmt::Return(_))) => b,
            _ => self.coerce(b, &ret),
        };
        let b = block(pre, b);
        self.ret = saved_ret;
        self.region = saved_region;
        self.loops = saved_loops;
        self.scopes.pop();
        self.selves.truncate(outer_selves);
        self.lambdas[id.0] = LambdaInfo {
            params: ptys,
            ret,
            owner: self.decl,
            group,
            yields: None,
        };
        mk(
            TK::Lambda {
                id,
                params: ids,
                body: Box::new(b),
            },
            self.lambda_ty(id),
        )
    }

    /// A type written on a lambda's parameter or result.
    fn written_ty(&mut self, t: &ast::Type, span: Span) -> Ty {
        let mut errs = Vec::new();
        let t = decls::body_type(
            self.d,
            self.module,
            &self.generics,
            &self.doms,
            t,
            &mut errs,
        );
        for e in errs {
            self.err(span, e.text);
        }
        self.fresh_sets(&t)
    }

    /// A block's runs of `fn` statements: each fn is a lambda its `let` binds, made before the
    /// block is checked so the fns of one run can call each other (R05.10.5).
    pub(super) fn declare_nested(&mut self, stmts: &[ast::Stmt]) {
        let mut run: Vec<LambdaId> = Vec::new();
        for s in stmts.iter().map(Some).chain([None]) {
            let fd = match s.map(|s| (&s.kind, s)) {
                Some((StmtKind::Fn(fd), s)) if nested_gap(fd, s).is_none() => fd,
                _ => {
                    for id in &run {
                        self.lambdas[id.0].group = run.clone();
                    }
                    run.clear();
                    continue;
                }
            };
            // Its types are reported once, where its lambda is made.
            let quiet = |c: &mut Self, t: &ast::Type| {
                let t = decls::body_type(c.d, c.module, &c.generics, &c.doms, t, &mut Vec::new());
                c.fresh_sets(&t)
            };
            let params = (fd.params.iter())
                .map(|p| p.ty.as_ref().map_or(Ty::Error, |t| quiet(self, t)))
                .collect();
            let ret = fd.ret.as_ref().map_or(Ty::Unit, |t| quiet(self, t));
            let id = LambdaId(self.lambdas.len());
            self.lambdas.push(LambdaInfo {
                params,
                ret,
                owner: self.decl,
                group: Vec::new(),
                yields: None,
            });
            let at = s.map_or(0, |s| s.span.start);
            self.group.push((at, fd.name.text.clone(), id));
            run.push(id);
        }
    }

    /// `fn f(x: T) -> R: body` in a block is `let f = (x: T) -> R => body` (R05.10).
    pub(super) fn nested_fn(&mut self, fd: &ast::FnDecl, s: &ast::Stmt, out: &mut Vec<TStmt>) {
        if let Some(what) = nested_gap(fd, s) {
            self.err(s.span, format!("slice: a nested fn with {what}"));
            self.bind(&fd.name.text, Ty::Error);
            return;
        }
        let Some(&(_, _, id)) = self.group.iter().rev().find(|g| g.0 == s.span.start) else {
            return;
        };
        let name = |text: &str, span| ast::Pattern {
            kind: PatternKind::Name(text.to_string()),
            span,
        };
        let body = match &fd.body {
            FnBody::Block(b) => ast::Expr {
                kind: ExprKind::Block(b.clone()),
                span: b.span,
            },
            FnBody::Expr(e) => e.clone(),
            FnBody::None => return,
        };
        let unit = ast::Type {
            kind: TypeKind::Tuple(Vec::new()),
            span: fd.sig_span,
        };
        let lambda = ast::Expr {
            kind: ExprKind::Lambda {
                params: (fd.params.iter())
                    .map(|p| name(&p.name.text, p.span))
                    .collect(),
                param_tys: fd.params.iter().map(|p| p.ty.clone()).collect(),
                param_mut: Vec::new(),
                ret_ty: Some(fd.ret.clone().unwrap_or(unit)),
                body: Box::new(body),
            },
            span: s.span,
        };
        let bind = ast::Stmt {
            kind: StmtKind::Let {
                mutable: false,
                pat: name(&fd.name.text, fd.name.span),
                ty: None,
                init: Some(lambda),
                otherwise: None,
                guard: None,
            },
            span: s.span,
            decorators: Vec::new(),
            public: false,
        };
        self.nested = Some(id);
        self.stmt(&bind, out);
        self.nested = None;
    }

    /// The variables bound where the code being checked is.
    pub(super) fn names_set(&self) -> BTreeSet<VarRef> {
        let top = self.decl.is_none();
        (self.scopes.iter().enumerate())
            .flat_map(|(i, s)| {
                s.values().map(move |&l| match top && i == 0 {
                    true => VarRef::Global(l),
                    false => VarRef::Local(l),
                })
            })
            .collect()
    }

    /// A nested `fn` used above its statement, in the block `out` checked: each variable it or a
    /// sibling it may call captures must be set where it is used (E0205, R06.1.7).
    pub(super) fn captures_set(&mut self, outer: usize, out: &[TStmt]) {
        let run: Vec<LambdaId> = self.group[outer..].iter().map(|g| g.2).collect();
        let (mine, rest) = std::mem::take(&mut self.ahead)
            .into_iter()
            .partition(|a| run.contains(&a.0));
        self.ahead = rest;
        let mine: Vec<(LambdaId, String, Span, BTreeSet<VarRef>)> = mine;
        if mine.is_empty() {
            return;
        }
        let mut caps: HashMap<LambdaId, BTreeSet<VarRef>> = HashMap::new();
        for s in out {
            let TStmt::Let(_, e) = s else { continue };
            let mut e = e;
            while let TK::Inject(x) | TK::Widen(x) = &e.kind {
                e = x;
            }
            let TK::Lambda { id, params, body } = &e.kind else {
                continue;
            };
            let (mut used, mut bound) = (BTreeSet::new(), BTreeSet::new());
            free_vars(body, &mut used, &mut bound);
            let own = |v: &VarRef| matches!(v, VarRef::Local(l) | VarRef::Global(l) if params.contains(l));
            let free = used.difference(&bound).filter(|v| !own(v)).copied();
            caps.insert(*id, free.collect());
        }
        for (id, name, span, set) in mine {
            let group = match &self.lambdas[id.0].group {
                g if g.is_empty() => vec![id],
                g => g.clone(),
            };
            let unset = (group.iter())
                .flat_map(|g| caps.get(g).into_iter().flatten())
                .find(|v| !set.contains(v));
            if let Some(v) = unset {
                let what = match v {
                    VarRef::Local(l) => self.locals[*l].name.clone(),
                    VarRef::Global(g) => self.globals[*g].name.clone(),
                };
                self.err(
                    span,
                    format!(
                        "{}: `{name}` uses `{what}`, which is not set yet where `{name}` is called",
                        cpl_diag::codes::USED_BEFORE_SET.0
                    ),
                );
            }
        }
    }

    pub(super) fn lambda_ty(&self, id: LambdaId) -> Ty {
        let targs = (0..self.generics.len()).map(Ty::Param).collect();
        let sets = (0..self.n_sets).map(SetTy::Param).collect();
        Ty::Clo(Box::new(CloRef::Lambda(id, targs, sets)))
    }

    /// `n` inside the body of the lambda `let n = ...` binds, unless something inside shadows it.
    pub(super) fn itself(&self, n: &str, span: Span) -> Option<TExpr> {
        let &(_, id, depth) = self.selves.iter().rev().find(|s| s.0 == n)?;
        if self.scopes[depth - 1..].iter().any(|s| s.contains_key(n)) {
            return None;
        }
        Some(TExpr {
            kind: TK::Itself(id),
            ty: self.lambda_ty(id),
            span,
        })
    }

    /// The fields a lambda's parameters take apart when it is passed as a fn of one value of
    /// this type: a tuple's positions or a record's fields, in order (R06.1.4).
    fn parts(&mut self, t: &Ty) -> Option<Vec<String>> {
        let Ty::Struct(d, ..) = self.shallow(t) else {
            return None;
        };
        let s = self.d.s(d);
        let names: Vec<String> = s.fields.iter().map(|f| f.0.clone()).collect();
        let positional = (names.iter().enumerate()).all(|(k, n)| *n == format!("_{k}"));
        (positional || crate::records::is_record(&s.name)).then_some(names)
    }
}

/// `all(x > 1 for x in xs)` passed to a fn whose next parameter is a test is `all(xs, x => x > 1)`:
/// a generator of tests is the predicate (R01.10.1, decision 53).
pub(super) fn tests_form(fd: &decls::FnDecl, args: &[ast::Arg]) -> Option<Vec<ast::Arg>> {
    let [a] = args else { return None };
    let ExprKind::Comprehension { body, clauses } = &a.value.kind else {
        return None;
    };
    let [c] = &clauses[..] else { return None };
    let test = matches!(fd.params.first().map(|p| &p.ty), Some(Ty::Fn(ps, r, _)) if ps.len() == 1 && **r == Ty::Bool);
    if !(a.value.is_generator() && fd.self_mode != SelfMode::None && test && c.conds.is_empty()) {
        return None;
    }
    let lambda = ExprKind::Lambda {
        params: vec![c.pat.clone()],
        param_tys: vec![None],
        param_mut: Vec::new(),
        ret_ty: None,
        body: body.clone(),
    };
    let arg = |value| ast::Arg {
        name: None,
        mutable: false,
        value,
    };
    let span = a.value.span;
    Some(vec![
        arg(c.iter.clone()),
        arg(ast::Expr { kind: lambda, span }),
    ])
}

/// What a nested `fn` has that a lambda cannot carry yet.
fn nested_gap(fd: &ast::FnDecl, s: &ast::Stmt) -> Option<&'static str> {
    let odd = |p: &ast::Param| p.variadic || p.lazy || p.named_only || p.label.is_some();
    Some(if !s.decorators.is_empty() {
        "a decorator"
    } else if !fd.generics.is_empty() || !fd.bounds.is_empty() {
        "type parameters"
    } else if fd.lends.is_some()
        || matches!(fd.ret.as_ref().map(|t| &t.kind), Some(TypeKind::Yields(_)))
    {
        "a `yields` or `lends` result"
    } else if !fd.uses.is_empty() {
        "`uses`"
    } else if fd.params.iter().any(|p| p.mutable) {
        "a `mut` parameter"
    } else if fd
        .params
        .iter()
        .any(|p| odd(p) || p.ty.is_none() || p.default.is_some())
    {
        "a parameter a lambda cannot take"
    } else if matches!(fd.body, FnBody::None) {
        "no body"
    } else {
        return None;
    })
}
