//! Lambdas: pattern parameters, unpacking a tuple or record argument, generators of tests (R06.1).
use super::call::CallArg;
use super::expr::block;
use super::*;
use cpl_next_ir::unbuilt::Unbuilt;
use std::collections::BTreeSet;

impl<'a> Cx<'a> {
    /// The id of the next entry of `t`, which holds `len`: the one the whole program's check
    /// gives it, whatever else is checked.
    pub(super) fn id(&mut self, t: super::Table, len: usize) -> usize {
        let key = (self.b.decl, self.copy, t, self.made[t as usize]);
        self.made[t as usize] += 1;
        self.id_at(key, len)
    }
    /// The id `key` names in its table, which holds `len`.
    pub(super) fn id_at(
        &mut self,
        key: (Option<FnId>, u32, super::Table, u32),
        len: usize,
    ) -> usize {
        let k = key.2 as usize;
        let next = self.ids.next[k].max(len);
        let id = *self.ids.keyed.entry(key).or_insert(next);
        self.ids.next[k] = self.ids.next[k].max(id + 1);
        id
    }
    /// A call passing sets: each set it passes also holds what the callee adds to it, which
    /// the callee's summary (`param_adds`) says and link reads per instance.
    pub(super) fn set_call(&mut self, f: &FnRef) {
        for (slot, s) in f.sets.iter().enumerate() {
            let at = self.id(super::Table::Added, self.added.len());
            put(&mut self.added, at, (f.clone(), slot));
            let m = Member::Sub(SetTy::AddedAt(at));
            match self.shallow_set(s) {
                SetTy::Var(v) => {
                    if let super::SetVar::Root(ms) = &mut self.sets[v] {
                        ms.push(m);
                    }
                }
                SetTy::Param(j) => {
                    if let Some(g) = self.b.decl {
                        self.param_adds.entry((g, j)).or_default().push(m);
                    }
                }
                SetTy::Global(g, k) => self.global_adds.entry((g, k)).or_default().push(m),
                _ => {}
            }
        }
    }
    /// Each global's fn-value sets, made that global's own (`SetTy::Global`) once the top level
    /// is checked: a body checked later adds to one in `global_adds`, so it changes no other
    /// body's types and each body is typed once it is checked.
    pub(super) fn own_global_sets(&mut self) {
        for g in 0..self.globals.len() {
            let mut sets = Vec::new();
            self.sets_in(&self.globals[g].ty.clone(), &mut sets);
            for (k, s) in sets.into_iter().enumerate() {
                if let SetTy::Var(r) = self.shallow_set(&s) {
                    let was = std::mem::replace(
                        &mut self.sets[r],
                        super::SetVar::Bound(SetTy::Global(g, k)),
                    );
                    if let super::SetVar::Root(ms) = was {
                        self.global_adds.entry((g, k)).or_default().extend(ms);
                    }
                }
            }
        }
    }
    /// The fn-value sets of `t`, in order.
    fn sets_in(&self, t: &Ty, out: &mut Vec<SetTy>) {
        match self.shallow(t) {
            Ty::Raw(e) | Ty::Ref(e) | Ty::Lent(e) | Ty::Mut(e) | Ty::RangeTo(e, ..) => {
                self.sets_in(&e, out)
            }
            Ty::Struct(_, ts, ss) => {
                ts.iter().for_each(|t| self.sets_in(t, out));
                out.extend(ss);
            }
            Ty::Fn(ps, r, s) => {
                ps.iter().for_each(|t| self.sets_in(t, out));
                self.sets_in(&r, out);
                out.push(s);
            }
            _ => {}
        }
    }
    pub(super) fn new_lambda(&mut self, info: LambdaInfo) -> LambdaId {
        let id = self.id(super::Table::Lambda, self.lambdas.len());
        super::put(&mut self.lambdas, id, info);
        LambdaId(id)
    }
    /// A lambda made from its parameters, result and body (R06.1): a written `(a, b) => e`, a
    /// nested `fn`, or a fn value the checker makes (`xs.map(bump)`).
    pub(super) fn lambda_of(
        &mut self,
        params: Vec<LamParam>,
        ret_ty: Option<Ty>,
        body: LamBody<'_, 'a>,
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let me = self.b.recur.take();
        let pre = self.b.nested.take();
        let nested = pre.and(me.clone());
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
        // `(a, b) => ..` passed as a fn of one tuple or record means `((a, b)) => ..` (R06.1.4):
        // one parameter, whose parts the patterns take.
        let mut params = params;
        let mut parts_of_one = None;
        if let Some([one]) = eps.as_deref() {
            if params.len() > 1 && params.iter().all(|p| p.ty.is_none() && p.pat.is_some()) {
                match self.parts(one) {
                    Some(fields) if fields.len() == params.len() => {
                        let pats: Vec<next::Pattern> =
                            params.drain(..).filter_map(|p| p.pat).collect();
                        parts_of_one = Some(pats);
                        params = vec![LamParam {
                            pat: None,
                            ty: None,
                            mutable: false,
                        }];
                    }
                    Some(fields) => {
                        let code = cpl_diag::codes::WRONG_ARG_COUNT;
                        let (k, n) = (params.len(), fields.len());
                        self.err(span, Msg::verdict(code, format!("this lambda takes {k} parameters; the value it is passed has {n} parts")));
                        return mk(TK::Error, Ty::Error);
                    }
                    None => {}
                }
            }
        }
        self.b.scopes.push(HashMap::new());
        let (mut ids, mut ptys, mut vals) = (Vec::new(), Vec::new(), Vec::new());
        // A pattern parameter is a slot of its own; the body opens by taking it apart (R06.1.3).
        let mut opens = Vec::new();
        for (i, p) in params.iter().enumerate() {
            let at = p.pat.as_ref().map_or(span, |q| self.at(q));
            let (name, opened) = match &p.pat {
                Some(next::Pattern::Path(pp)) => match super::pattern::bind_name(pp) {
                    Some(n) => (Some(n.text().to_string()), None),
                    None => {
                        self.err(span, Unbuilt::LambdaParam);
                        continue;
                    }
                },
                Some(next::Pattern::Wildcard(_)) | None => (None, None),
                Some(
                    q @ (next::Pattern::Tuple(_)
                    | next::Pattern::Record(_)
                    | next::Pattern::Paren(_)),
                ) => (None, Some(q.clone())),
                Some(_) => {
                    self.err(span, Unbuilt::LambdaParam);
                    continue;
                }
            };
            let ty = match (p.ty.clone(), eps.as_ref().and_then(|ps| ps.get(i))) {
                (Some(t), _) => t,
                (None, Some(t)) => t.clone(),
                (None, None) => {
                    let shown = name.as_deref().unwrap_or("_");
                    self.err(
                        span,
                        Msg::verdict(cpl_diag::codes::INFERENCE_NEEDS_ANNOTATION, format!("the type of `{shown}` is not known here; write `({shown}: T) => ...`")),
                    );
                    Ty::Error
                }
            };
            // A `mut` parameter's place is the caller's: its signature says so, its body sees `T`.
            let ty = match self.shallow(&ty) {
                Ty::Mut(t) => *t,
                _ => ty,
            };
            ptys.push(match p.mutable {
                true => Ty::Mut(Box::new(ty.clone())),
                false => ty.clone(),
            });
            let var = match &name {
                Some(n) => self.bind(n, ty.clone()),
                None => self.temp("$param", ty.clone()),
            };
            match var {
                VarRef::Local(id) | VarRef::Global(id) => ids.push(id),
            }
            let read = TExpr {
                kind: TK::Var(var),
                ty,
                span: at,
            };
            if let Some(q) = opened {
                opens.push((q, read.clone()));
            }
            vals.push(read);
        }
        let saved_ret = self.b.ret.clone();
        // A lambda is not part of the generator or lender around it: no `yield` or `lend` inside.
        let saved_region = self.b.region.take();
        let saved_loops = std::mem::take(&mut self.b.loops);
        let ret = match ret_ty {
            Some(t) => t,
            None => er.unwrap_or_else(|| self.new_ty()),
        };
        self.b.ret = ret.clone();
        // The body sees its own signature, so a lambda bound by `let` can call itself (R06.2.3).
        // A nested `fn` was made before its block was checked; it also sees its siblings.
        let (id, group) = match pre {
            Some(id) => (id, self.lambdas[id.0].group.clone()),
            None => {
                let id = self.new_lambda(LambdaInfo {
                    params: ptys.clone(),
                    ret: ret.clone(),
                    owner: self.b.decl,
                    group: Vec::new(),
                    yields: None,
                    written: Vec::new(),
                });
                (id, Vec::new())
            }
        };
        let outer_selves = self.b.selves.len();
        let depth = self.b.scopes.len();
        if let Some(name) = me {
            self.b.selves.push((name, id, depth));
        }
        for (_, name, sib) in &self.b.group {
            if let Some(name) = name.as_ref().filter(|_| *sib != id && group.contains(sib)) {
                self.b.selves.push((name.clone(), *sib, depth));
            }
        }
        let mut pre = Vec::new();
        for (q, read) in opens {
            self.let_pattern(&q, false, read, &mut pre);
        }
        if let (Some(pats), Some(whole)) = (parts_of_one, vals.first().cloned()) {
            let tys = self
                .field_tys(&whole.ty)
                .map(|(_, t)| t)
                .unwrap_or_default();
            for (k, q) in pats.iter().enumerate() {
                let ty = tys.get(k).cloned().unwrap_or(Ty::Error);
                let kind = TK::Field(Box::new(whole.clone()), k);
                let at = self.at(q);
                self.let_pattern(q, false, TExpr { kind, ty, span: at }, &mut pre);
            }
        }
        // A body that ends in `return` has no value of its own to give the result.
        // Where a `()` result is wanted the body is a statement, as a `fn`'s is:
        // `(mut d, e) => d.unite(e.0, e.1)` drops `unite`'s `Bool` (R01.8.1, R06.4.8).
        let unit = self.shallow(&ret) == Ty::Unit && body.bare_try().is_none() && !body.is_block();
        // A nested `fn` whose body can end without its value is that error (E0409).
        let falls = match (nested.as_ref(), body.falls_through(&|n| self.traps(n))) {
            (Some(name), Some(end)) => self.falls_through(name, end, &ret),
            _ => false,
        };
        let never = body.never_ends(&|n| self.traps(n));
        let b = match unit || falls {
            true => self.lambda_effect(body, vals, &ret, span),
            false => self.lambda_value(body, vals, &ret),
        };
        let b = match &b.kind {
            _ if unit || falls => b,
            TK::Block(ss, None) if matches!(ss.last(), Some(TStmt::Return(_))) || never => b,
            _ => self.coerce(b, &ret),
        };
        let b = block(pre, b);
        self.b.ret = saved_ret;
        self.b.region = saved_region;
        self.b.loops = saved_loops;
        self.b.scopes.pop();
        self.b.selves.truncate(outer_selves);
        self.lambdas[id.0] = LambdaInfo {
            params: ptys,
            ret,
            owner: self.b.decl,
            group,
            yields: None,
            written: std::mem::take(&mut self.lambdas[id.0].written),
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
    /// A lambda's body run for its effects, its value dropped.
    fn lambda_effect(
        &mut self,
        body: LamBody<'_, 'a>,
        vals: Vec<TExpr>,
        ret: &Ty,
        span: Span,
    ) -> TExpr {
        self.scoped_block(span, |cx, out| {
            match body {
                LamBody::Arm(next::ArmBody::Expr(e)) | LamBody::Fn(next::FnBody::Expr(e)) => {
                    cx.expr_stmt(&e, out)
                }
                LamBody::Arm(next::ArmBody::Stmt(s)) => cx.stmt(&s, out),
                LamBody::Arm(next::ArmBody::Block(b)) | LamBody::Fn(next::FnBody::Block(b)) => {
                    let e = cx.block(&b, None);
                    out.push(TStmt::Expr(e));
                }
                LamBody::Built(f) => {
                    let e = f(cx, vals, ret);
                    out.push(TStmt::Expr(e));
                }
                LamBody::Ready(e) => out.push(TStmt::Expr(e)),
            }
            None
        })
    }
    /// A lambda's body checked as its result.
    fn lambda_value(&mut self, body: LamBody<'_, 'a>, vals: Vec<TExpr>, ret: &Ty) -> TExpr {
        match body {
            LamBody::Arm(b) => self.arm_body(&b, Some(ret)),
            LamBody::Fn(next::FnBody::Block(b)) => self.block(&b, Some(ret)),
            LamBody::Fn(next::FnBody::Expr(e)) => self.expr(&e, Some(ret)),
            LamBody::Built(f) => f(self, vals, ret),
            LamBody::Ready(e) => e,
        }
    }
    /// A nested `fn`'s parameter types, each jagged one its reach (R06.4.1), and its result,
    /// reporting nothing: its types are reported once, where its lambda is made.
    pub(super) fn nested_sig(&mut self, fd: &next::FnDecl) -> (Vec<Ty>, Ty) {
        let seen = self.errors.len();
        let sig = self.fn_sig(fd, &mut Vec::new());
        self.errors.truncate(seen);
        sig
    }
    fn fn_sig(&mut self, fd: &next::FnDecl, out: &mut Vec<TStmt>) -> (Vec<Ty>, Ty) {
        let ps = nested_params(fd);
        let jag = cpl_next_decls::jagged(&written_params(&ps));
        let mut tys = Vec::new();
        for (p, j) in ps.iter().zip(&jag) {
            tys.push(match (j, p.ty()) {
                (Some(((lo, hi), _)), _) => Ty::Range(*lo, *hi),
                (None, Some(t)) => self.body_ty(&t, out),
                (None, None) => Ty::Error,
            });
        }
        let ret = match fd.ret().and_then(|r| r.ty()) {
            Some(t) => self.body_ty(&t, out),
            None => Ty::Unit,
        };
        (tys, ret)
    }
    /// A block's runs of `fn`s: each fn is a lambda, made before the block is checked so the
    /// fns of one run can call each other (R05.10.5). A `@memo fn`'s body is one of its own,
    /// under the name only its wrapper reaches.
    pub(super) fn declare_nested(&mut self, entries: &[next::BlockEntry]) {
        let mut run: Vec<LambdaId> = Vec::new();
        let end_run = |cx: &mut Self, run: &mut Vec<LambdaId>| {
            for id in run.iter() {
                cx.lambdas[id.0].group = run.clone();
            }
            run.clear();
        };
        for e in entries {
            let fd = match e {
                next::BlockEntry::Decl(next::Decl::Fn(fd)) => fd,
                _ => {
                    end_run(self, &mut run);
                    continue;
                }
            };
            let memo = super::stmt::is_memo(fd);
            if !memo && nested_gap(fd).is_some() {
                end_run(self, &mut run);
                continue;
            }
            let (params, ret) = self.nested_sig(fd);
            let ps = written_params(&nested_params(fd));
            let jag = cpl_next_decls::jagged(&ps);
            let written = match jag.iter().all(Option::is_none) {
                true => Vec::new(),
                false => (ps.into_iter().zip(jag))
                    .map(|((n, _), j)| (n, j.map(|(_, t)| t)))
                    .collect(),
            };
            let id = self.new_lambda(LambdaInfo {
                params,
                ret,
                owner: self.b.decl,
                group: Vec::new(),
                yields: None,
                written,
            });
            let own = fd.name().map_or_else(String::new, |t| t.text().to_string());
            let at = self.at(fd).start;
            if memo {
                end_run(self, &mut run);
                self.b.group.push((at, None, id));
                run.push(id);
                end_run(self, &mut run);
                continue;
            }
            self.b.group.push((at, Some(own), id));
            run.push(id);
        }
        end_run(self, &mut run);
    }
    /// `fn f(x: T) -> R: body` in a block is a lambda bound to `name` (R05.10): its own name,
    /// or `None` for a `@memo fn`'s body, which only its wrapper calls.
    pub(super) fn nested_fn(
        &mut self,
        fd: &next::FnDecl,
        name: Option<&str>,
        out: &mut Vec<TStmt>,
    ) -> Option<VarRef> {
        let span = self.at(fd);
        if let Some(what) = nested_gap(fd) {
            let what = what.to_string();
            self.err(span, Unbuilt::NestedFn { what });
            return Some(match name {
                Some(n) => self.bind(n, Ty::Error),
                None => self.temp("$body", Ty::Error),
            });
        }
        let &(_, _, id) = self.b.group.iter().rev().find(|g| g.0 == span.start)?;
        let body = LamBody::Fn(fd.body()?);
        let (tys, ret) = self.fn_sig(fd, out);
        let params = (nested_params(fd).into_iter().zip(tys))
            .map(|(p, ty)| LamParam {
                pat: p.pattern(),
                ty: Some(ty),
                mutable: false,
            })
            .collect();
        self.b.nested = Some(id);
        self.b.recur = name.map(str::to_string);
        let v = self.lambda_of(params, Some(ret), body, None, span);
        self.b.nested = None;
        self.b.recur = None;
        // A body no name reaches (a `@memo` fn's) is the checker's own binding.
        let Some(name) = name else {
            let var = self.temp("$body", v.ty.clone());
            out.push(TStmt::Let(var, v));
            return Some(var);
        };
        let var = self.let_bind(name, false, v.ty.clone(), v, span, out);
        if fd.name().is_none_or(|t| t.text() != name) {
            match var {
                VarRef::Local(l) => self.b.locals[l].synthetic = true,
                VarRef::Global(g) => self.globals[g].synthetic = true,
            }
        }
        Some(var)
    }
    /// The variables bound where the code being checked is.
    pub(super) fn names_set(&self) -> BTreeSet<VarRef> {
        let top = self.b.decl.is_none();
        (self.b.scopes.iter().enumerate())
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
        let run: Vec<LambdaId> = self.b.group[outer..].iter().map(|g| g.2).collect();
        let (mine, rest) = std::mem::take(&mut self.b.ahead)
            .into_iter()
            .partition(|a| run.contains(&a.0));
        self.b.ahead = rest;
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
            free_vars(body, self.b.decl.is_none(), &mut used, &mut bound);
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
                    VarRef::Local(l) => self.b.locals[*l].name.clone(),
                    VarRef::Global(g) => self.globals[*g].name.clone(),
                };
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::USED_BEFORE_SET,
                        format!(
                            "`{name}` uses `{what}`, which is not set yet where `{name}` is called"
                        ),
                    ),
                );
            }
        }
    }

    /// A declared fn passed where a fn of fewer parameters is wanted: its defaults fill the rest,
    /// so `xs.map(bump)` is `xs.map(x => bump(x))` (R06.1.8).
    pub(super) fn filled(&mut self, n: &str, want: Option<&Ty>, span: Span) -> Option<TExpr> {
        let Some(Ty::Fn(ps, ..)) = want.map(|w| self.shallow(w)) else {
            return None;
        };
        let f = (self.fn_named(self.b.module, n)).filter(|_| self.lookup(n).is_none())?;
        let fd = self.items.f(f);
        if fd.params.len() <= ps.len() || fd.params[ps.len()..].iter().any(|p| p.default.is_none())
        {
            return None;
        }
        Some(self.forwarding(n, ps.len(), want, span))
    }
    /// `(a, ..) => n(a, ..)`: a lambda of `k` parameters that calls the fn `n` with them.
    fn forwarding(&mut self, n: &str, k: usize, want: Option<&Ty>, span: Span) -> TExpr {
        let params = (0..k)
            .map(|_| LamParam {
                pat: None,
                ty: None,
                mutable: false,
            })
            .collect();
        let name = n.to_string();
        let body = move |cx: &mut Cx<'a>, xs: Vec<TExpr>, ret: &Ty| {
            let args: Vec<CallArg> = (xs.into_iter())
                .map(|x| CallArg::held(None, x, span))
                .collect();
            cx.call_named(&name, &[], &args, span, Some(ret))
        };
        self.lambda_of(params, None, LamBody::Built(Box::new(body)), want, span)
    }

    pub(super) fn lambda_ty(&self, id: LambdaId) -> Ty {
        let targs = (0..self.b.generics.len()).map(Ty::Param).collect();
        let sets = (0..self.b.n_sets).map(SetTy::Param).collect();
        Ty::Clo(Box::new(CloRef::Lambda(id, targs, sets)))
    }

    /// `n` inside the body of the lambda `let n = ...` binds, unless something inside shadows it.
    pub(super) fn itself(&self, n: &str, span: Span) -> Option<TExpr> {
        let &(_, id, depth) = self.b.selves.iter().rev().find(|s| s.0 == n)?;
        if self.b.scopes[depth - 1..].iter().any(|s| s.contains_key(n)) {
            return None;
        }
        Some(TExpr {
            kind: TK::Itself(id),
            ty: self.lambda_ty(id),
            span,
        })
    }

    /// A fn item named where a fn of fewer parameters is wanted, each one it leaves out defaulted
    /// or variadic: `(a, ..) => f(a, ..)`, so a call through the value fills the rest from their
    /// defaults (R06.1.5) and gives a variadic none, as `max` passed as `(T, T) -> T`.
    pub(super) fn defaulted_fn_value(
        &mut self,
        n: &str,
        span: Span,
        want: Option<&Ty>,
    ) -> Option<TExpr> {
        let Ty::Fn(ps, ..) = self.shallow(want?) else {
            return None;
        };
        let d = self
            .fn_named(self.b.module, n)
            .filter(|_| self.declared(n))?;
        let fd = self.items.f(d);
        let rest = fd.params.get(ps.len()..).filter(|r| !r.is_empty())?;
        if !rest.iter().all(|p| p.default.is_some() || p.variadic)
            || fd.params[..ps.len()].iter().any(|p| p.variadic)
        {
            return None;
        }
        Some(self.forwarding(n, ps.len(), want, span))
    }

    /// The fields a lambda's parameters take apart when it is passed as a fn of one value of
    /// this type: a tuple's positions or a record's fields, in order (R06.1.4).
    fn parts(&mut self, t: &Ty) -> Option<Vec<String>> {
        let Ty::Struct(d, ..) = self.shallow(t) else {
            return None;
        };
        let s = self.items.s(d);
        let names = self.field_names(d);
        let positional = (names.iter().enumerate()).all(|(k, n)| *n == format!("_{k}"));
        (positional || s.record).then_some(names)
    }
}

/// One parameter of a lambda being made: what binds it (`None` for a slot only a made body
/// reads), its type when it is known, and whether it is `mut`.
pub(super) struct LamParam {
    pub pat: Option<next::Pattern>,
    pub ty: Option<Ty>,
    pub mutable: bool,
}

/// A lambda's body: a written arm (`x => e`), a nested fn's, or one the checker makes from
/// the parameters' values and the result wanted.
pub(super) enum LamBody<'b, 'a> {
    Arm(next::ArmBody),
    Fn(next::FnBody),
    Built(Box<dyn FnOnce(&mut Cx<'a>, Vec<TExpr>, &Ty) -> TExpr + 'b>),
    /// A body built before its lambda, which reads none of its parameters.
    Ready(TExpr),
}

impl LamBody<'_, '_> {
    fn bare_try(&self) -> Option<Span> {
        match self {
            LamBody::Arm(next::ArmBody::Expr(e)) | LamBody::Fn(next::FnBody::Expr(e)) => {
                e.bare_try()
            }
            _ => None,
        }
    }
    fn is_block(&self) -> bool {
        matches!(
            self,
            LamBody::Arm(next::ArmBody::Block(_) | next::ArmBody::Expr(next::Expr::Block(_)))
                | LamBody::Fn(next::FnBody::Block(_) | next::FnBody::Expr(next::Expr::Block(_)))
        )
    }
    fn falls_through(
        &self,
        todo: cpl_next_syntax::fall_through::Traps,
    ) -> Option<cpl_next_syntax::fall_through::End> {
        match self {
            LamBody::Arm(b) => Some(b.falls_through(todo)),
            LamBody::Fn(b) => Some(b.falls_through(todo)),
            LamBody::Built(_) | LamBody::Ready(_) => None,
        }
    }
    fn never_ends(&self, todo: cpl_next_syntax::fall_through::Traps) -> bool {
        match self {
            LamBody::Arm(next::ArmBody::Expr(e)) => e.never_ends(todo),
            LamBody::Arm(next::ArmBody::Block(b)) => b.never_ends(todo),
            LamBody::Fn(b) => b.never_ends(todo),
            LamBody::Arm(next::ArmBody::Stmt(_)) | LamBody::Built(_) | LamBody::Ready(_) => false,
        }
    }
}

fn nested_params(fd: &next::FnDecl) -> Vec<next::Param> {
    fd.params()
        .map(|l| l.params().collect())
        .unwrap_or_default()
}

/// Each parameter's name and written type, as `jagged` reads them.
fn written_params(ps: &[next::Param]) -> Vec<(String, Option<next::Type>)> {
    (ps.iter())
        .map(|p| {
            (
                p.name().map_or_else(String::new, |t| t.text().to_string()),
                p.ty(),
            )
        })
        .collect()
}

/// What a nested `fn` has that a lambda cannot carry yet. `@memo` is its wrapper's.
fn nested_gap(fd: &next::FnDecl) -> Option<&'static str> {
    use cpl_next_syntax::grammar::Decorator;
    let ps = nested_params(fd);
    let odd = |p: &next::Param| p.is_variadic() || p.is_named_only() || p.label().is_some();
    let memo = |x: &next::Decorator| {
        x.name().and_then(|n| Decorator::of(n.text())) == Some(Decorator::Memo)
    };
    let ret = fd.ret();
    Some(if fd.decorators().any(|x| !memo(&x)) {
        "a decorator"
    } else if fd.generics().is_some() || fd.where_clause().is_some() {
        "type parameters"
    } else if ret.as_ref().is_some_and(|r| r.is_yields() || r.lends()) {
        "a `yields` or `lends` result"
    } else if fd.uses().is_some() {
        "`uses`"
    } else if ps.iter().any(|p| p.is_mut()) {
        "a `mut` parameter"
    } else if ps
        .iter()
        .any(|p| odd(p) || p.ty().is_none() || p.default().is_some())
    {
        "a parameter a lambda cannot take"
    } else if fd.body().is_none() {
        "no body"
    } else {
        return None;
    })
}
