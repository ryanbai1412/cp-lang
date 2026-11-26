//! Lambdas: pattern parameters, unpacking a tuple or record argument, generators of tests (R06.1).
use super::expr::block;
use super::*;

impl Cx<'_> {
    pub(super) fn lambda(
        &mut self,
        params: &[ast::Pattern],
        param_tys: &[Option<ast::Type>],
        body: &ast::Expr,
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        // A lambda passed where `?(A -> R)` is wanted is that fn, made `Some` by the coercion (R02.4.2).
        let expected = want.map(|w| {
            let w = self.shallow(w);
            match self.optional_of(&w).map(|t| self.shallow(&t)) {
                Some(f @ Ty::Fn(..)) => f,
                _ => w,
            }
        });
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
                (Some(t), _) => {
                    let mut errs = Vec::new();
                    let t = decls::body_type(
                        self.d,
                        self.module,
                        &self.generics,
                        &self.doms,
                        t,
                        &mut errs,
                    );
                    self.fresh_sets(&t)
                }
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
        let ret = er.unwrap_or_else(|| self.new_ty());
        self.ret = ret.clone();
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
        let b = self.expr(body, Some(&ret));
        let b = block(pre, self.coerce(b, &ret));
        self.ret = saved_ret;
        self.region = saved_region;
        self.loops = saved_loops;
        self.scopes.pop();
        let id = LambdaId(self.lambdas.len());
        self.lambdas.push(LambdaInfo {
            params: ptys,
            ret,
            owner: self.decl,
        });
        let targs = (0..self.generics.len()).map(Ty::Param).collect();
        let sets = (0..self.n_sets).map(SetTy::Param).collect();
        mk(
            TK::Lambda {
                id,
                params: ids,
                body: Box::new(b),
            },
            Ty::Clo(Box::new(CloRef::Lambda(id, targs, sets))),
        )
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
