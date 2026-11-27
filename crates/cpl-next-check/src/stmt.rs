//! Statements, scopes, domains, ranges and reads.

use super::*;
use cpl_next_ir::unbuilt::Unbuilt;

impl<'a> Cx<'a> {
    /// The innermost scope.
    pub(super) fn scope(&mut self) -> &mut HashMap<String, usize> {
        if self.scopes.is_empty() {
            self.scopes.push(HashMap::new());
        }
        let last = self.scopes.len() - 1;
        &mut self.scopes[last]
    }
    pub(super) fn add_local(&mut self, name: &str, ty: Ty) -> LocalId {
        self.locals.push(Local {
            name: name.to_string(),
            ty,
            top: false,
            mutable: false,
            fixed: false,
        });
        let id = self.locals.len() - 1;
        self.scope().insert(name.to_string(), id);
        id
    }
    /// A new binding: a global at top level, a local in a fn.
    pub(super) fn bind(&mut self, name: &str, ty: Ty) -> VarRef {
        if self.decl.is_none() && self.scopes.len() == 1 {
            self.globals.push(Local {
                name: name.to_string(),
                ty,
                top: true,
                mutable: false,
                fixed: false,
            });
            self.global_names
                .insert(name.to_string(), self.globals.len() - 1);
            VarRef::Global(self.globals.len() - 1)
        } else if self.decl.is_none() {
            // A binding inside a top-level block lives in the globals too, under its scope.
            self.globals.push(Local {
                name: name.to_string(),
                ty,
                top: false,
                mutable: false,
                fixed: false,
            });
            let id = self.globals.len() - 1;
            self.scope().insert(name.to_string(), id);
            VarRef::Global(id)
        } else {
            VarRef::Local(self.add_local(name, ty))
        }
    }
    pub(super) fn lookup(&self, name: &str) -> Option<(VarRef, Ty)> {
        for (depth, s) in self.scopes.iter().enumerate().rev() {
            if let Some(id) = s.get(name) {
                return Some(if self.decl.is_none() {
                    let _ = depth;
                    (VarRef::Global(*id), self.globals[*id].ty.clone())
                } else {
                    (VarRef::Local(*id), self.locals[*id].ty.clone())
                });
            }
        }
        // A field of `self` is nearer than a top-level binding.
        if self.self_field(name).is_some() {
            return None;
        }
        self.global_names
            .get(name)
            .map(|id| (VarRef::Global(*id), self.globals[*id].ty.clone()))
    }
    pub(super) fn self_expr(&self, span: Span) -> Option<TExpr> {
        let id = self.self_local?;
        Some(TExpr {
            kind: TK::Var(VarRef::Local(id)),
            ty: self.locals[id].ty.clone(),
            span,
        })
    }

    /// `for name in v`: a value walks by its `recipe`, a bound's walk, or its `each` (else `iter`).
    fn walk_value(
        &mut self,
        mut call: TExpr,
        name: &str,
        body: &ast::Block,
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        if let Some(walk) = self.recipe_walk(&call) {
            return self.value_walk(walk, call, name, body, out);
        }
        // `for x in g.out(u)` through a bound on `g`'s type parameter.
        call = match call.kind {
            TK::CallTrait {
                tr,
                method,
                recv: Some(recv),
                args,
                ..
            } => return self.trait_walk((tr, method, recv, args), name, body, span, out),
            kind => TExpr { kind, ..call },
        };
        if !matches!(&call.kind, TK::Call { f, .. } if self.d.f(f.decl).region.is_some()) {
            // A type without its own `each` walks through its `Seq` impl (R07.1.1).
            let rt = self.shallow(&call.ty);
            if let Ty::Param(p) = rt {
                return self.bound_walk(p, call, name, body, span, out);
            }
            self.allow_region = true;
            let own = "each";
            let walk = match self.d.lookup_member(&rt, own) {
                Some(_) => own,
                None => "iter",
            };
            call = self.method_call(call, walk, vec![], span, false);
            self.allow_region = false;
        }
        let TK::Call { f, recv, args } = call.kind else {
            return;
        };
        let Some((false, elem)) = self.d.f(f.decl).region.clone() else {
            return self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    "this cannot be walked with `for`",
                ),
            );
        };
        let elem = self.result_of(&f, &elem);
        self.scopes.push(HashMap::new());
        let var = self.bind(name, elem);
        let body = self.block(body, None);
        self.scopes.pop();
        out.push(TStmt::Region {
            callee: Callee::Fn(f),
            recv,
            args,
            var,
            lend: false,
            body,
        });
    }
    /// `for x in s` with `s` of a type parameter: its bound's `iter` walks it, and each
    /// instance picks the impl (R07.1.1).
    /// `for x in g.out(u)` with `out` a generator of a bound on `g`'s type parameter: the
    /// impl each instance picks runs the body.
    fn trait_walk(
        &mut self,
        (tr, method, recv, args): (TraitRef, String, Box<TArg>, Vec<TArg>),
        name: &str,
        body: &ast::Block,
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        let m = self.d.traits[tr.tr.0].methods[&method];
        let Some((false, elem)) = self.d.f(m).region.clone() else {
            return self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    "this cannot be walked with `for`",
                ),
            );
        };
        let (TArg::Value(r) | TArg::Mut(r)) = recv.as_ref();
        let mut tys = vec![r.ty.clone()];
        tys.extend(tr.args.iter().cloned());
        let elem = Subst {
            tys: &tys,
            sets: &[],
        }
        .ty(&elem);
        self.scopes.push(HashMap::new());
        let var = self.bind(name, elem);
        let body = self.block(body, None);
        self.scopes.pop();
        out.push(TStmt::Region {
            callee: Callee::Trait(tr, method),
            recv: Some(recv),
            args,
            var,
            lend: false,
            body,
        });
    }
    fn bound_walk(
        &mut self,
        p: usize,
        recv: TExpr,
        name: &str,
        body: &ast::Block,
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        let found = (self.bound_traits(p).into_iter())
            .find_map(|tr| Some((self.d.traits[tr.tr.0].methods.get("iter").copied()?, tr)));
        let Some((m, tr)) = found else {
            let shown = self.show(&recv.ty);
            return self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::UNKNOWN_FIELD,
                    format!("`{shown}` has no `iter`"),
                ),
            );
        };
        let Some((false, elem)) = self.d.f(m).region.clone() else {
            return self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    "this cannot be walked with `for`",
                ),
            );
        };
        self.reach(m);
        let mut tys = vec![recv.ty.clone()];
        tys.extend(tr.args.iter().cloned());
        let elem = Subst {
            tys: &tys,
            sets: &[],
        }
        .ty(&elem);
        self.scopes.push(HashMap::new());
        let var = self.bind(name, elem);
        let body = self.block(body, None);
        self.scopes.pop();
        out.push(TStmt::Region {
            callee: Callee::Trait(tr.clone(), "iter".to_string()),
            recv: Some(Box::new(TArg::Value(recv))),
            args: Vec::new(),
            var,
            lend: false,
            body,
        });
    }
    /// E0311: a `@memo` fn's cache keeps results computed from what it captured, so nothing
    /// may write a captured `var` while the cache lives, to the end of its block (R06.4.1).
    fn memo_writes(&mut self, f: &ast::FnDecl, rest: &[ast::Stmt]) {
        use cpl_syntax::visit::{walk_block, walk_expr, walk_stmt, Visitor};
        struct Names(Vec<String>);
        impl Visitor for Names {
            fn expr(&mut self, e: &ast::Expr) {
                if let ExprKind::Name(n) = &e.kind {
                    self.0.push(n.clone());
                }
            }
        }
        fn root(e: &ast::Expr) -> Option<&str> {
            match &e.kind {
                ExprKind::Name(n) => Some(n),
                ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => root(base),
                ExprKind::Paren(x) => root(x),
                _ => None,
            }
        }
        struct Writes<'a> {
            names: &'a std::collections::HashSet<String>,
            at: Vec<(Span, String)>,
        }
        impl Writes<'_> {
            fn note(&mut self, e: &ast::Expr) {
                if let Some(n) = root(e).filter(|n| self.names.contains(*n)) {
                    self.at.push((e.span, n.to_string()));
                }
            }
        }
        impl Visitor for Writes<'_> {
            fn stmt(&mut self, s: &ast::Stmt) {
                if let StmtKind::Assign { target, .. } = &s.kind {
                    self.note(target);
                }
            }
            fn expr(&mut self, e: &ast::Expr) {
                if let ExprKind::Call { args, .. } = &e.kind {
                    for a in args.iter().filter(|a| a.mutable) {
                        self.note(&a.value);
                    }
                }
            }
        }
        let mut names = Names(Vec::new());
        match &f.body {
            FnBody::Block(b) => walk_block(b, &mut names),
            FnBody::Expr(e) => walk_expr(e, &mut names),
            FnBody::None => {}
        }
        let params: Vec<&str> = f.params.iter().map(|p| p.name.text.as_str()).collect();
        let captured: std::collections::HashSet<String> = (names.0.into_iter())
            .filter(|n| !params.contains(&n.as_str()))
            .filter(|n| match self.lookup(n) {
                Some((VarRef::Local(l), _)) => self.locals[l].mutable,
                Some((VarRef::Global(g), _)) => self.globals[g].mutable,
                None => false,
            })
            .collect();
        if captured.is_empty() {
            return;
        }
        let mut w = Writes {
            names: &captured,
            at: Vec::new(),
        };
        rest.iter().for_each(|s| walk_stmt(s, &mut w));
        for (span, n) in w.at {
            let text = format!(
                "`{n}` is written while the cache of `@memo` fn `{}`, which reads it, lives",
                f.name.text
            );
            self.err(span, Msg::verdict(cpl_diag::codes::BAD_MEMO, text));
        }
    }
    pub(super) fn block(&mut self, b: &ast::Block, want: Option<&Ty>) -> TExpr {
        self.scopes.push(HashMap::new());
        let mut out = Vec::new();
        let mut value = None;
        let outer = self.group.len();
        let wraps = self.wraps;
        let frames = self.frames.len();
        let memo = cpl_next_decls::memo_block(&b.stmts, &self.d.memo_tables);
        let (stmts, memos) = match &memo {
            Some((s, m)) => (&s[..], &m[..]),
            None => (&b.stmts[..], &[][..]),
        };
        self.declare_nested(stmts);
        for (i, s) in stmts.iter().enumerate() {
            if let Some((_, f)) = memos.iter().find(|(at, _)| *at == i) {
                self.memo_writes(f, &stmts[i + 4..]);
            }
            // A bare `with` line sets a context for the rest of its block (R09.1.1).
            if let StmtKind::With {
                ctx,
                args,
                body: None,
            } = &s.kind
            {
                if i > 0 {
                    self.err(
                        s.span,
                        Msg::verdict(
                            cpl_diag::codes::WITH_NOT_FIRST,
                            "a bare `with` line comes first in its block",
                        ),
                    );
                    continue;
                } else if ctx.text == "checks" {
                    if let Some(w) = self.checks(args, s.span) {
                        self.wraps = w;
                    }
                    continue;
                } else if let Some(fs) = self.ctx_frames(ctx, args) {
                    self.frames.extend(fs);
                    continue;
                }
            }
            // A last expression that never gives a value is a statement, as `return` is.
            if i + 1 == stmts.len() {
                if let StmtKind::Expr(e) = &s.kind {
                    if e.leaves() {
                        self.stmt(s, &mut out);
                        break;
                    }
                    // `a[i].push(x)` last in a block that gives no value writes the cell, as
                    // the same line anywhere else in the block does.
                    let unit = want.is_none_or(|w| self.shallow(w) == Ty::Unit);
                    if unit && self.cell_method(e, &mut out) {
                        break;
                    }
                    let v = self.expr(e, want);
                    value = Some(match want {
                        Some(w) if self.shallow(w) != Ty::Unit => self.coerce(v, w),
                        _ => v,
                    });
                    break;
                }
            }
            self.stmt(s, &mut out);
        }
        self.wraps = wraps;
        self.frames.truncate(frames);
        self.captures_set(outer, &out);
        self.group.truncate(outer);
        self.scopes.pop();
        let ty = value
            .as_ref()
            .map(|v: &TExpr| v.ty.clone())
            .unwrap_or(Ty::Unit);
        TExpr {
            kind: TK::Block(out, value.map(Box::new)),
            ty,
            span: b.span,
        }
    }
    /// A condition, in a scope of its own that holds the names its `let`s bind: the caller
    /// checks what the condition guards, then pops the scope.
    pub(super) fn cond(&mut self, c: &Cond) -> TExpr {
        self.scopes.push(HashMap::new());
        self.cond_in(c)
    }
    /// An expression a condition tests: a `Bool`, never an integer or optional read as one (R02.11.1).
    pub(super) fn bool_cond(&mut self, e: &ast::Expr) -> TExpr {
        let v = self.expr(e, Some(&Ty::Bool));
        let t = self.shallow(&v.ty);
        if matches!(t, Ty::Bool | Ty::Var(_) | Ty::Error) || self.coercion(&t, &Ty::Bool).is_some()
        {
            return self.coerce(v, &Ty::Bool);
        }
        let code = cpl_diag::codes::CONDITION_NOT_BOOL;
        let shown = self.show(&t);
        self.err(
            v.span,
            Msg::verdict(code, format!("this condition is a `{shown}`, not a `Bool`")),
        );
        TExpr {
            kind: TK::Error,
            ty: Ty::Bool,
            span: v.span,
        }
    }
    fn cond_in(&mut self, c: &Cond) -> TExpr {
        match c {
            // `let p = e`: holds when `e` matches `p`, whose names then name its parts; on an
            // optional `e`, a pattern of no optional matches the value it has (R01.7.1).
            Cond::Let { pat, value, and } => {
                let span = value.span;
                let v = self.expr(value, None);
                let mk = |kind| TExpr {
                    kind,
                    ty: Ty::Bool,
                    span,
                };
                let inner = (self.optional_of(&v.ty)).filter(|_| !self.optional_pattern(pat));
                if inner.is_none() && matches!(pat.kind, PatternKind::Name(_)) {
                    if self.shallow(&v.ty) != Ty::Error {
                        let t = self.show(&v.ty);
                        self.err(span, Unbuilt::IfLet { ty: t });
                    }
                    // Its names are the error, so their uses say nothing more.
                    self.bind_unknown(pat, false, &mut Vec::new());
                    return TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    };
                }
                let (hold, read) = self.held(v);
                let mut binds = Vec::new();
                let test = match inner {
                    Some(inner) => {
                        let flag = mk(TK::Field(Box::new(read.clone()), 0));
                        let val = TExpr {
                            kind: TK::Field(Box::new(read), 1),
                            ty: inner,
                            span,
                        };
                        let rest = self.refute(pat, &val, false, &mut binds);
                        Self::both(Some(flag), rest)
                    }
                    None => self.refute(pat, &read, false, &mut binds),
                };
                let rest = match and {
                    Some(more) => self.cond_in(more),
                    None => mk(TK::Bool(true)),
                };
                let then = mk(TK::Block(binds, Some(Box::new(rest))));
                let body = match test {
                    Some(t) => mk(TK::If(
                        Box::new(t),
                        Box::new(then),
                        Box::new(mk(TK::Bool(false))),
                    )),
                    None => then,
                };
                mk(TK::Block(hold, Some(Box::new(body))))
            }
            Cond::Guard { test, and } => {
                let t = self.bool_cond(test);
                let rest = self.cond_in(and);
                let span = t.span;
                let no = TExpr {
                    kind: TK::Bool(false),
                    ty: Ty::Bool,
                    span,
                };
                TExpr {
                    kind: TK::If(Box::new(t), Box::new(rest), Box::new(no)),
                    ty: Ty::Bool,
                    span,
                }
            }
            Cond::Expr(e) => self.bool_cond(e),
            _ => {
                self.errors.push(Unbuilt::Condition.into());
                TExpr {
                    kind: TK::Error,
                    ty: Ty::Bool,
                    span: Span::new(0, 0),
                }
            }
        }
    }
    /// `let (a, b) = e` or `let (u:, v:) = e`: binds the whole value, then each part by its field.
    fn let_parts(
        &mut self,
        s: &ast::Stmt,
        pat: &ast::Pattern,
        parts: Vec<(String, ast::Pattern)>,
        exact: Option<usize>,
        out: &mut Vec<TStmt>,
    ) {
        let tmp = format!("$tuple{}", pat.span.start);
        let mut whole = s.clone();
        if let StmtKind::Let { pat, mutable, .. } = &mut whole.kind {
            *pat = ast::Pattern {
                kind: PatternKind::Name(tmp.clone()),
                span: pat.span,
            };
            *mutable = false;
        }
        self.stmt(&whole, out);
        // A record pattern without `..` names every field (R02.5.5).
        if let Some(n) = exact {
            let name = ExprKind::Name(tmp.clone());
            let w = self.expr(
                &ast::Expr {
                    kind: name,
                    span: pat.span,
                },
                None,
            );
            if let Ty::Struct(d, ..) = self.shallow(&w.ty) {
                let have = self.d.s(d).fields.len();
                if self.d.s(d).record && have != n {
                    let code = cpl_diag::codes::TYPE_MISMATCH;
                    let msg = Msg::verdict(
                        code,
                        format!(
                            "this names {n} of the record's {have} fields; write `..` for the rest"
                        ),
                    );
                    self.err(pat.span, msg);
                }
            }
        }
        for (field, p) in parts {
            if matches!(p.kind, PatternKind::Wildcard) {
                continue;
            }
            let at = |kind| ast::Expr { kind, span: p.span };
            let part = at(ExprKind::Field {
                base: Box::new(at(ExprKind::Name(tmp.clone()))),
                name: ast::Name {
                    text: field,
                    span: p.span,
                },
            });
            let mut one = s.clone();
            if let StmtKind::Let { pat, ty, init, .. } = &mut one.kind {
                *pat = p;
                *ty = None;
                *init = Some(part);
            }
            self.stmt(&one, out);
        }
    }
    pub(super) fn stmt(&mut self, s: &ast::Stmt, out: &mut Vec<TStmt>) {
        match &s.kind {
            StmtKind::Let {
                mutable,
                pat,
                ty,
                init: Some(init),
                otherwise: None,
                ..
            } => {
                match &pat.kind {
                    // `var x` in a pattern binds a name that may change (R01.6.3).
                    PatternKind::Var(inner) => {
                        let mut one = s.clone();
                        if let StmtKind::Let { mutable, pat, .. } = &mut one.kind {
                            (*mutable, *pat) = (true, (**inner).clone());
                        }
                        return self.stmt(&one, out);
                    }
                    PatternKind::Variant { path, args } if path.len() == 1 && args.is_empty() => {
                        let code = cpl_diag::codes::VALUE_NAME_CASE;
                        let n = &path[0].text;
                        return self.err(
                            pat.span,
                            Msg::verdict(
                                code,
                                format!("a value's name starts lowercase, not `{n}`"),
                            ),
                        );
                    }
                    PatternKind::Literal(_) | PatternKind::Variant { .. } | PatternKind::Or(_) => {
                        let code = cpl_diag::codes::EXPECTED_PATTERN;
                        let msg = Msg::verdict(
                            code,
                            "this pattern may not match; give the `let` an `else` that leaves",
                        );
                        return self.err(s.span, msg);
                    }
                    _ => {}
                }
                if let PatternKind::Record { fields, rest } = &pat.kind {
                    let parts = fields.iter().map(|(n, p)| (n.text.clone(), p.clone()));
                    let exact = (!rest).then_some(fields.len());
                    return self.let_parts(s, pat, parts.collect(), exact, out);
                }
                if let PatternKind::Tuple(ps) = &pat.kind {
                    // `let (a, b): (T, U) = read()` reads each part in order. Tuples as values are not in the slice.
                    let (tys, call) = match (ty.as_ref().map(|t| &t.kind), Self::read_call(init)) {
                        (Some(TypeKind::Tuple(ts)), Some((None, call))) if ts.len() == ps.len() => {
                            (ts.clone(), call)
                        }
                        (
                            None,
                            Some((
                                Some(ast::Type {
                                    kind: TypeKind::Tuple(ts),
                                    ..
                                }),
                                call,
                            )),
                        ) if ts.len() == ps.len() => (ts, call),
                        _ => {
                            let parts =
                                (ps.iter().enumerate()).map(|(k, p)| (k.to_string(), p.clone()));
                            return self.let_parts(s, pat, parts.collect(), None, out);
                        }
                    };
                    for (p, t) in ps.iter().zip(tys) {
                        let mut one = s.clone();
                        if let StmtKind::Let { pat, ty, init, .. } = &mut one.kind {
                            *pat = p.clone();
                            *ty = Some(t);
                            *init = Some(call.clone());
                        }
                        self.stmt(&one, out);
                    }
                    return;
                }
                // `let _ = e` runs `e` for its effects and drops its value.
                if let PatternKind::Wildcard = pat.kind {
                    let declared = ty.as_ref().map(|t| self.body_ty(t, s.span, out));
                    let v = self.expr(init, declared.as_ref());
                    let v = match &declared {
                        Some(t) => self.coerce(v, t),
                        None => v,
                    };
                    out.push(TStmt::Expr(v));
                    return;
                }
                let PatternKind::Name(name) = &pat.kind else {
                    self.err(s.span, Unbuilt::LetPattern);
                    return self.bind_unknown(pat, false, out);
                };
                let declared = ty.as_ref().map(|t| self.body_ty(t, s.span, out));
                // A literal is never an array, so where one is declared it is the fill: its cell's
                // type, as a range of cuts filling `[V]Range<V>` is a piece (R04.4.4).
                let literal = matches!(
                    init.kind,
                    ExprKind::Int(_)
                        | ExprKind::Float(_)
                        | ExprKind::Str(_)
                        | ExprKind::Char(_)
                        | ExprKind::Bool(_)
                        | ExprKind::None
                        | ExprKind::Range { .. }
                );
                let cell = match (&declared, self.array_id()) {
                    (Some(Ty::Struct(d, ts, _)), Some(arr)) if literal && *d == arr => {
                        Some(ts[1].clone())
                    }
                    _ => None,
                };
                if !mutable && matches!(init.kind, ExprKind::Lambda { .. }) {
                    self.recur = Some(name.clone());
                }
                let v = self.expr(init, cell.as_ref().or(declared.as_ref()));
                self.recur = None;
                // `var a: [V]Int = 0` fills every slot: it is `Array.filled(0)`.
                let fill = match (&declared, self.array_id()) {
                    (Some(array @ Ty::Struct(d, ts, _)), Some(arr))
                        if *d == arr
                            && (!matches!(self.shallow(&v.ty), Ty::Struct(..))
                                || self.fits(&v.ty, &ts[1])) =>
                    {
                        Some((arr, ts.clone(), array.clone()))
                    }
                    _ => None,
                };
                let v = match fill {
                    Some((arr, ts, array)) => {
                        let elem = ts[1].clone();
                        let v = self.coerce(v, &elem);
                        let f = self.fnref(self.d.s(arr).methods["filled"], ts.clone(), Vec::new());
                        TExpr {
                            kind: TK::Call {
                                f,
                                recv: None,
                                args: vec![TArg::Value(v)],
                            },
                            ty: array,
                            span: s.span,
                        }
                    }
                    None => v,
                };
                let v = match &declared {
                    Some(t) => self.coerce(v, t),
                    // A `var` holding a function can be reassigned, so it gets a set; a `let` keeps the closure's own type.
                    None if *mutable => match self.shallow(&v.ty) {
                        Ty::Clo(c) => {
                            let (ps, r) = self.clo_sig(&c);
                            let slot = Ty::Fn(ps, Box::new(r), self.new_set());
                            self.coerce(v, &slot)
                        }
                        Ty::Fn(ps, r, set) if !matches!(self.shallow_set(&set), SetTy::Var(_)) => {
                            let slot = Ty::Fn(ps, r, self.new_set());
                            self.coerce(v, &slot)
                        }
                        _ => v,
                    },
                    None => v,
                };
                self.memory_peak(name, &v.ty, s.span);
                // What is declared is the binding's type even when the value is not one.
                let var = self.bind(name, declared.unwrap_or_else(|| v.ty.clone()));
                if let Some(keys) = super::index::literal_cut(init) {
                    self.view_keys.insert(var, keys);
                }
                match var {
                    VarRef::Local(i) => {
                        (self.locals[i].mutable, self.locals[i].fixed) = (*mutable, !*mutable)
                    }
                    VarRef::Global(i) => {
                        (self.globals[i].mutable, self.globals[i].fixed) = (*mutable, !*mutable)
                    }
                }
                out.push(TStmt::Let(var, v));
            }
            // `let v = x else: leave`: `v` names `x`'s payload after it, and the block runs when there is none (R01.6.2).
            StmtKind::Let {
                mutable,
                pat,
                ty,
                init: Some(init),
                otherwise: Some(otherwise),
                guard: None,
            } => {
                if !otherwise.leaves() {
                    let code = cpl_diag::codes::ELSE_FALLS_THROUGH;
                    self.err(s.span, Msg::verdict(code, "this `else` block can reach its end; end it with `return`, `break`, `continue` or a trap"));
                }
                let declared = ty.as_ref().map(|t| self.body_ty(t, s.span, out));
                // `let p = x else: leave`: the block runs when `x` does not match `p`, whose names
                // are bound after it.
                let PatternKind::Name(name) = &pat.kind else {
                    let v = self.expr(init, declared.as_ref());
                    let v = match &declared {
                        Some(t) => self.coerce(v, t),
                        None => v,
                    };
                    let (hold, read) = self.held(v);
                    out.extend(hold);
                    let none = self.block(otherwise, None);
                    let mut binds = Vec::new();
                    if let Some(test) = self.refute(pat, &read, *mutable, &mut binds) {
                        let span = s.span;
                        let unit = TExpr {
                            kind: TK::Unit,
                            ty: Ty::Unit,
                            span,
                        };
                        out.push(TStmt::Expr(TExpr {
                            kind: TK::If(Box::new(test), Box::new(unit), Box::new(none)),
                            ty: Ty::Unit,
                            span,
                        }));
                    }
                    out.extend(binds);
                    return;
                };
                let want = declared.clone().map(|t| self.optional_ty(t));
                let v = self.expr(init, want.as_ref());
                let v = match &want {
                    Some(w) => self.coerce(v, w),
                    None => v,
                };
                let Some(inner) = self.optional_of(&v.ty) else {
                    if self.shallow(&v.ty) != Ty::Error {
                        let t = self.show(&v.ty);
                        self.err(init.span, Unbuilt::LetElse { ty: t });
                    }
                    // Its name is the error, so its uses say nothing more.
                    self.block(otherwise, None);
                    self.bind(name, Ty::Error);
                    return;
                };
                let (hold, flag, val) = self.opened(v, inner.clone());
                out.extend(hold);
                let none = self.block(otherwise, None);
                let span = s.span;
                let unit = TExpr {
                    kind: TK::Unit,
                    ty: Ty::Unit,
                    span,
                };
                out.push(TStmt::Expr(TExpr {
                    kind: TK::If(Box::new(flag), Box::new(unit), Box::new(none)),
                    ty: Ty::Unit,
                    span,
                }));
                let var = self.bind(name, inner);
                match var {
                    VarRef::Local(i) => {
                        (self.locals[i].mutable, self.locals[i].fixed) = (*mutable, !*mutable)
                    }
                    VarRef::Global(i) => {
                        (self.globals[i].mutable, self.globals[i].fixed) = (*mutable, !*mutable)
                    }
                }
                out.push(TStmt::Let(var, val));
            }
            // `(a, b) = (b, a)`: the right side is built first, then each place takes its part (R01.6.4).
            StmtKind::Assign {
                target:
                    ast::Expr {
                        kind: ExprKind::Tuple(places),
                        ..
                    },
                op: ast::AssignOp::Set,
                value,
            } => {
                let tmp = format!("$parts{}", s.span.start);
                let at = |kind, span| ast::Expr { kind, span };
                let whole = ast::Stmt {
                    kind: StmtKind::Let {
                        mutable: false,
                        pat: ast::Pattern {
                            kind: PatternKind::Name(tmp.clone()),
                            span: value.span,
                        },
                        ty: None,
                        init: Some(value.clone()),
                        otherwise: None,
                        guard: None,
                    },
                    span: s.span,
                    ..s.clone()
                };
                self.stmt(&whole, out);
                for (k, place) in places.iter().enumerate() {
                    let part = at(
                        ExprKind::Field {
                            base: Box::new(at(ExprKind::Name(tmp.clone()), place.span)),
                            name: ast::Name {
                                text: k.to_string(),
                                span: place.span,
                            },
                        },
                        place.span,
                    );
                    let one = ast::Stmt {
                        kind: StmtKind::Assign {
                            target: place.clone(),
                            op: ast::AssignOp::Set,
                            value: part,
                        },
                        span: place.span,
                        ..s.clone()
                    };
                    self.stmt(&one, out);
                }
            }
            StmtKind::Assign { target, op, value } if super::index::packed(target).is_some() => {
                let packed = super::index::packed(target);
                let one = ast::Stmt {
                    kind: StmtKind::Assign {
                        target: packed.unwrap_or_else(|| target.clone()),
                        op: *op,
                        value: value.clone(),
                    },
                    ..s.clone()
                };
                self.stmt(&one, out);
            }
            StmtKind::Assign { target, op, value } => {
                // `a[i] = v` and `a[i] op= v`: the indices, then the value, are read once; then
                // `a = v` is `a`'s `IndexSet` write when it has one (spec 12, W8); otherwise `a`'s
                // `IndexMut` lends the cell and the assignment runs while it is lent (R12.4.6).
                if let ExprKind::Index { indices, .. } = &target.kind {
                    if indices.len() == 1 {
                        let target = self.hoist_indices(target, out);
                        let ExprKind::Index { base, indices } = &target.kind else {
                            return;
                        };
                        // The cell's type, so `out[v] = j..i` of cuts is a piece (R04.4.4).
                        let cell = match &super::index::unparen(base).kind {
                            ExprKind::Name(n) => self.lookup(n).and_then(|(_, t)| self.cell_of(&t)),
                            _ => None,
                        };
                        let value = self.hoist(value, cell.as_ref(), "$rhs", out);
                        let (op, span) = (*op, s.span);
                        if op == ast::AssignOp::Set {
                            if let Some(set) = self.index_set(base, &indices[0], &value, span) {
                                return self.stmt(&set, out);
                            }
                        }
                        self.through_cell(
                            (base, &indices[0]),
                            target.span,
                            out,
                            Box::new(move |this, cell, out| {
                                let assign = ast::Stmt {
                                    kind: StmtKind::Assign {
                                        target: cell,
                                        op,
                                        value,
                                    },
                                    span,
                                    decorators: Vec::new(),
                                    public: false,
                                };
                                this.stmt(&assign, out)
                            }),
                        );
                        return;
                    }
                }
                if let ExprKind::Call { .. } = &target.kind {
                    // `t.at(i) = v`: `at` lends a place; the assignment runs while it is lent.
                    self.allow_region = true;
                    let call = self.expr(target, None);
                    self.allow_region = false;
                    let TK::Call { f, recv, args } = call.kind else {
                        return;
                    };
                    let Some((true, elem)) = self.d.f(f.decl).region.clone() else {
                        return self.err(
                            s.span,
                            Msg::verdict(
                                cpl_diag::codes::TYPE_MISMATCH,
                                "this call does not lend a place, so it cannot be assigned",
                            ),
                        );
                    };
                    let elem = Subst {
                        tys: &f.targs,
                        sets: &f.sets,
                    }
                    .ty(&elem);
                    self.scopes.push(HashMap::new());
                    let var = self.bind("$lent", elem.clone());
                    let lent = TExpr {
                        kind: TK::Var(var),
                        ty: elem.clone(),
                        span: target.span,
                    };
                    let v = self.expr(value, Some(&elem));
                    let v = match op {
                        AssignOp::Set => v,
                        AssignOp::Binary(b) => self.binary(*b, lent.clone(), v, target.span),
                        _ => return self.err(s.span, Unbuilt::Assignment),
                    };
                    let v = self.coerce(v, &elem);
                    self.scopes.pop();
                    let body = TExpr {
                        kind: TK::Block(vec![TStmt::Assign(lent, v)], None),
                        ty: Ty::Unit,
                        span: s.span,
                    };
                    out.push(TStmt::Region {
                        callee: Callee::Fn(f),
                        recv,
                        args,
                        var,
                        lend: true,
                        body,
                    });
                    return;
                }
                let place = self.place(target);
                let mut root = &place;
                let mut cache = false;
                while let TK::Field(b, i) = &root.kind {
                    if let Ty::Struct(sid, ..) = self.shallow(&b.ty) {
                        {
                            let sd = self.d.s(sid);
                            cache |= sd.cached.get(*i).copied().unwrap_or(false);
                            if !cache && !sd.var_fields.get(*i).copied().unwrap_or(true) {
                                let (st, f) = (sd.name.clone(), sd.fields[*i].0.clone());
                                self.err(target.span, Msg::verdict(cpl_diag::codes::ASSIGN_FIXED_FIELD, format!("field `{st}.{f}` is fixed (not `var`) and cannot be assigned")));
                            }
                        }
                    }
                    root = b;
                }
                if let Some(name) = self.let_root(&place).filter(|_| !cache) {
                    self.err(target.span, Msg::verdict(cpl_diag::codes::ASSIGN_TO_LET, format!("`{name}` is bound by `let` and cannot be assigned; declare it with `var`")));
                }
                let v = match op {
                    AssignOp::Set => self.expr(value, Some(&place.ty)),
                    AssignOp::Binary(b) => {
                        let v = self.expr(value, Some(&place.ty));
                        self.binary(*b, place.clone(), v, target.span)
                    }
                    _ => return self.err(s.span, Unbuilt::Assignment),
                };
                let v = self.coerce(v, &place.ty);
                out.push(TStmt::Assign(place, v));
            }
            // A `const` of the program is a `let` its initializer fixes (R01.6.1).
            StmtKind::Const {
                name,
                generics,
                ty,
                init: Some(init),
                ..
            } if generics.is_empty() => {
                let kind = StmtKind::Let {
                    mutable: false,
                    pat: ast::Pattern {
                        kind: PatternKind::Name(name.text.clone()),
                        span: name.span,
                    },
                    ty: ty.clone(),
                    init: Some(init.clone()),
                    otherwise: None,
                    guard: None,
                };
                self.stmt(&ast::Stmt { kind, ..s.clone() }, out)
            }
            StmtKind::Expr(e) => {
                if self.cell_method(e, out) {
                    return;
                }
                let v = self.expr(e, None);
                out.push(TStmt::Expr(v));
            }
            StmtKind::While {
                cond,
                body,
                else_body: None,
            } => {
                let c = self.cond(cond);
                self.loops += 1;
                let b = self.block(body, None);
                self.loops -= 1;
                self.scopes.pop();
                out.push(TStmt::While(c, b));
            }
            StmtKind::For { .. } if !self.in_for => {
                self.loops += 1;
                self.in_for = true;
                self.stmt(s, out);
                self.in_for = false;
                self.loops -= 1;
            }
            StmtKind::For {
                pat,
                iter,
                body,
                else_body: None,
                ..
            } => {
                self.in_for = false;
                // `for _ in xs` (and `repeat n:`): each item is bound to a name nothing reads.
                if let PatternKind::Wildcard = pat.kind {
                    let mut named = s.clone();
                    if let StmtKind::For { pat, .. } = &mut named.kind {
                        pat.kind = PatternKind::Name("$each".into());
                    }
                    self.in_for = true;
                    return self.stmt(&named, out);
                }
                if let (PatternKind::Name(name), ExprKind::Name(dn)) = (&pat.kind, &iter.kind) {
                    // A declared domain's positions are its ordinals; any other walks its `Seq`.
                    let ordinals = self.domain_named(dn).filter(|d| matches!(d, Ty::Dom(_)));
                    if let Some(dty) = ordinals {
                        let lo = TExpr {
                            kind: TK::Int(0),
                            ty: Ty::Int,
                            span: s.span,
                        };
                        let hi = TExpr {
                            kind: TK::DomLen(dty.clone()),
                            ty: Ty::Int,
                            span: s.span,
                        };
                        self.scopes.push(HashMap::new());
                        let var = self.bind(name, dty);
                        let b = self.block(body, None);
                        self.scopes.pop();
                        out.push(TStmt::For(var, lo, hi, b));
                        return;
                    }
                }
                if let PatternKind::Tuple(_) | PatternKind::Record { .. } = &pat.kind {
                    // `for (a, b) in xs` binds the item, then `let (a, b) = item` opens the body.
                    let tmp = format!("$item{}", s.span.start);
                    let mut one = s.clone();
                    if let StmtKind::For {
                        pat: p, body: b, ..
                    } = &mut one.kind
                    {
                        let open = ast::Stmt {
                            kind: StmtKind::Let {
                                mutable: false,
                                pat: pat.clone(),
                                ty: None,
                                init: Some(ast::Expr {
                                    kind: ExprKind::Name(tmp.clone()),
                                    span: pat.span,
                                }),
                                otherwise: None,
                                guard: None,
                            },
                            span: pat.span,
                            decorators: Vec::new(),
                            public: false,
                        };
                        *p = ast::Pattern {
                            kind: PatternKind::Name(tmp),
                            span: pat.span,
                        };
                        b.stmts.insert(0, open);
                    }
                    return self.stmt(&one, out);
                }
                let closed = matches!(
                    iter.kind,
                    ExprKind::Range {
                        lo: Some(_),
                        hi: Some(_),
                        step: None,
                        ..
                    }
                );
                if let (PatternKind::Name(name), false) = (&pat.kind, closed) {
                    // `for x in gen(args)` or `xs.getter`, or `for x in xs` through the type's own `each`.
                    self.allow_region =
                        matches!(iter.kind, ExprKind::Call { .. } | ExprKind::Field { .. });
                    let call = self.expr(iter, None);
                    self.allow_region = false;
                    return self.walk_value(call, name, body, s.span, out);
                }
                let (
                    PatternKind::Name(name),
                    ExprKind::Range {
                        lo: Some(lo),
                        hi: Some(hi),
                        inclusive,
                        step: None,
                    },
                ) = (&pat.kind, &iter.kind)
                else {
                    return self.err(s.span, Unbuilt::ForForm);
                };
                let first = self.expr(lo, Some(&Ty::Int));
                if self.ord_ends(&first.ty, None).is_some() {
                    let r = self.range_from(first, hi, *inclusive, None, None, iter.span);
                    return self.walk_value(r, name, body, s.span, out);
                }
                // `lo..=hi` is `lo..hi+1` here, so it fails at the largest Int where the language does not.
                let hi1;
                let hi = if *inclusive {
                    hi1 = Self::plus_one(hi);
                    &hi1
                } else {
                    hi
                };
                let lo = self.coerce(first, &Ty::Int);
                let hi = self.expr(hi, Some(&Ty::Int));
                let hi = self.coerce(hi, &Ty::Int);
                self.scopes.push(HashMap::new());
                let var = self.bind(name, Ty::Int);
                let b = self.block(body, None);
                self.scopes.pop();
                out.push(TStmt::For(var, lo, hi, b));
            }
            StmtKind::Return(e) => {
                let ret = self.ret.clone();
                let v = e.as_ref().map(|e| {
                    let v = self.expr(e, Some(&ret));
                    self.coerce(v, &ret)
                });
                out.push(TStmt::Return(v));
            }
            StmtKind::Assert { cond, message } => {
                let c = self.bool_cond(cond);
                let msg = match message.as_ref().map(|m| &m.kind) {
                    Some(ExprKind::Str(ps)) => ps
                        .iter()
                        .map(|p| {
                            if let StrPiece::Text(t) = p {
                                t.clone()
                            } else {
                                String::new()
                            }
                        })
                        .collect(),
                    _ => "assertion failed".to_string(),
                };
                out.push(TStmt::Assert(c, msg, s.span, CheckKind::Assert));
            }
            StmtKind::Domain { name, def }
                if matches!(def.kind, ExprKind::Tuple(_) | ExprKind::Record(_)) =>
            {
                let def = self.declare_factors(&name.text, def, s.span, out);
                let Some(p) = self.type_path(&def) else {
                    return self.err(
                        def.span,
                        Msg::verdict(
                            cpl_diag::codes::TYPE_MISMATCH,
                            "a product domain's parts are domains",
                        ),
                    );
                };
                self.doms.insert(name.text.clone(), p);
            }
            StmtKind::Domain { name, def } => {
                let ExprKind::Range {
                    lo: Some(lo),
                    hi: Some(hi),
                    inclusive,
                    step: None,
                } = &def.kind
                else {
                    self.err(
                        s.span,
                        Unbuilt::DomainKind {
                            kind: crate::kind_name(&def.kind),
                        },
                    );
                    self.doms.insert(name.text.clone(), Ty::Error);
                    self.refused_names.insert(name.text.clone());
                    return;
                };
                let (lo, end, within) = self.domain_ends(lo, hi, *inclusive);
                self.declare_domain(&name.text, lo, end, within, s.span, out);
            }
            StmtKind::Yield { value, from: false } => match self.region.clone() {
                Some((false, t)) => {
                    let v = self.expr(value, Some(&t));
                    let v = self.coerce(v, &t);
                    out.push(TStmt::Yield(v));
                }
                _ => self.err(
                    s.span,
                    Msg::verdict(
                        cpl_diag::codes::TYPE_MISMATCH,
                        "`yield` outside a fn that `yields`",
                    ),
                ),
            },
            StmtKind::Lend { place, .. } => match self.region.clone() {
                // Every path through a lender reaches one `lend`; inside a loop it could run twice.
                Some((true, _)) if self.loops > 0 => self.err(
                    s.span,
                    Msg::verdict(
                        cpl_diag::codes::BAD_PROJECTION,
                        "`lend` inside a loop: a lender lends one place once",
                    ),
                ),
                Some((true, t)) => {
                    // `lend mut s[k]` of a cell that is no slot lends on the cell `s`'s
                    // `IndexMut` lends (R12.4.6).
                    if let ExprKind::Index { base, indices } = &super::index::unparen(place).kind {
                        let seen = self.errors.len();
                        let probe = self.expr_as_typed(place, None);
                        self.errors.truncate(seen);
                        if let ([k], false) = (&indices[..], matches!(probe.kind, TK::Slot(..))) {
                            let span = s.span;
                            return self.through_cell(
                                (base, k),
                                place.span,
                                out,
                                Box::new(move |this, cell, out| {
                                    this.lend_out(&cell, &t, span, out)
                                }),
                            );
                        }
                    }
                    self.lend_out(place, &t, s.span, out);
                }
                // In a read lend, `lend place` returns the place's value (R05.6.3).
                None if (self.decl).is_some_and(|f| self.d.f(f).lends.is_some()) => {
                    let ret = self.ret.clone();
                    let v = self.expr(place, Some(&ret));
                    let v = self.coerce(v, &ret);
                    out.push(TStmt::Return(Some(v)));
                }
                _ => self.err(
                    s.span,
                    Msg::verdict(
                        cpl_diag::codes::BAD_PROJECTION,
                        "`lend` outside a fn that `lends`",
                    ),
                ),
            },
            StmtKind::Break(None) => out.push(TStmt::Break),
            StmtKind::Continue(None) => out.push(TStmt::Continue),
            StmtKind::Fn(fd) => self.nested_fn(fd, s, out),
            // `var a: T` with no value: the parser has said so, and `a` is still declared, so
            // what names it below is checked as if the value were there.
            StmtKind::Let {
                pat:
                    ast::Pattern {
                        kind: PatternKind::Name(name),
                        ..
                    },
                ty,
                init: None,
                ..
            } => {
                let t = match ty {
                    Some(t) => self.body_ty(t, s.span, out),
                    None => Ty::Error,
                };
                self.bind(name, t);
            }
            StmtKind::With {
                ctx,
                args,
                body: Some(body),
            } if ctx.text == "checks" => {
                if let Some(w) = self.checks(args, s.span) {
                    let outer = std::mem::replace(&mut self.wraps, w);
                    let b = self.block(body, None);
                    self.wraps = outer;
                    out.push(TStmt::Expr(b));
                }
            }
            // A block takes its own bare `with` lines; one here is the program's own, for the
            // rest of the file (R09.1.1).
            StmtKind::With {
                ctx,
                args,
                body: None,
            } if ctx.text == "checks" => {
                if let Some(w) = self.checks(args, s.span) {
                    self.wraps = w;
                }
            }
            StmtKind::With {
                ctx,
                args,
                body: None,
            } if matches!(
                self.d.named(self.module, &ctx.text),
                Some(decls::Named::Context(_))
            ) =>
            {
                if let Some(fs) = self.ctx_frames(ctx, args) {
                    self.frames.extend(fs);
                }
            }
            StmtKind::With {
                ctx,
                args,
                body: Some(body),
            } if matches!(
                self.d.named(self.module, &ctx.text),
                Some(decls::Named::Context(_))
            ) =>
            {
                if let Some(fs) = self.ctx_frames(ctx, args) {
                    let n = self.frames.len();
                    self.frames.extend(fs);
                    let b = self.block(body, None);
                    self.frames.truncate(n);
                    out.push(TStmt::Expr(b));
                }
            }
            // `unchecked:` turns every check off in its block (R09.4.3); overflow is the one whose
            // result it defines: `Int` arithmetic wraps.
            StmtKind::Unchecked(body) => {
                let outer = std::mem::replace(&mut self.wraps, true);
                let b = self.block(body, None);
                self.wraps = outer;
                out.push(TStmt::Expr(b));
            }
            other => {
                let module = match other {
                    StmtKind::Mod { name, .. } => Some(name.text.as_str()),
                    _ => None,
                };
                if let Some(n) = decls::item_name(s).or(module) {
                    self.refused_names.insert(n.to_string());
                }
                self.err(
                    s.span,
                    Unbuilt::Statement {
                        kind: crate::kind_name(other),
                    },
                )
            }
        }
    }
    /// Whether `Int` arithmetic wraps under `with checks(...)` (R09.4). A checked build already
    /// runs every check, so `: on` changes nothing but `overflow: on` inside `overflow: off`.
    fn checks(&mut self, args: &[ast::Arg], span: Span) -> Option<bool> {
        let mut wraps = self.wraps;
        for a in args {
            let name = a.name.as_ref().map(|n| n.text.as_str());
            let on = match &a.value.kind {
                ExprKind::Name(v) if v == "on" => true,
                ExprKind::Name(v) if v == "off" => false,
                _ => {
                    self.err(span, Unbuilt::ChecksValue);
                    return None;
                }
            };
            match name {
                Some("overflow") => wraps = !on,
                Some("ranges" | "bounds") if on => {}
                _ => {
                    self.err(span, Unbuilt::ChecksField);
                    return None;
                }
            }
        }
        Some(wraps)
    }
    /// `read()` or `read<T>()`: the type argument if written, and the bare `read()` call.
    pub(super) fn read_call(e: &ast::Expr) -> Option<(Option<ast::Type>, ast::Expr)> {
        let ExprKind::Call { callee, args } = &e.kind else {
            return None;
        };
        if !args.is_empty() {
            return None;
        }
        match &callee.kind {
            ExprKind::Name(n) if n == "read" => Some((None, e.clone())),
            ExprKind::Generic { base, args: targs }
                if targs.len() == 1 && matches!(&base.kind, ExprKind::Name(n) if n == "read") =>
            {
                Some((
                    Some(targs[0].clone()),
                    ast::Expr {
                        kind: ExprKind::Call {
                            callee: base.clone(),
                            args: Vec::new(),
                        },
                        span: e.span,
                    },
                ))
            }
            _ => None,
        }
    }
    /// The start of a domain `lo..hi` or `lo..=hi`, the integer after its last position, and
    /// what the types of the two ends say every position lies within.
    pub(super) fn domain_ends(
        &mut self,
        lo: &ast::Expr,
        hi: &ast::Expr,
        inclusive: bool,
    ) -> (TExpr, TExpr, (Option<i64>, Option<i64>)) {
        // An end keeps the range type it was written with.
        let lo = self.expr_as_typed(lo, None);
        let hi = self.expr_as_typed(hi, None);
        let ends = |e: &TExpr, ty: Ty| match (&e.kind, ty) {
            (TK::Int(k), _) => (Some(*k), Some(*k)),
            (_, Ty::Range(a, b)) => (Some(a), Some(b)),
            _ => (None, None),
        };
        let least = ends(&lo, self.shallow(&lo.ty)).0;
        let most = ends(&hi, self.shallow(&hi.ty)).1;
        let most = if inclusive {
            most
        } else {
            most.map(|m| m.saturating_sub(1))
        };
        let lo = self.coerce(lo, &Ty::Int);
        let hi = self.coerce(hi, &Ty::Int);
        let end = if inclusive {
            let span = hi.span;
            let one = TExpr {
                kind: TK::Int(1),
                ty: Ty::Int,
                span,
            };
            TExpr {
                kind: TK::Bin(BinOp::Add, Box::new(hi), Box::new(one), Overflow::Traps),
                ty: Ty::Int,
                span,
            }
        } else {
            hi
        };
        (lo, end, (least, most))
    }
    /// A domain's size is a hidden variable next to its declaration.
    pub(super) fn array_id(&self) -> Option<StructId> {
        self.d.sugar.array
    }
    /// E0904: an array the input alone can fill past the judge's memory limit (R03.1.9). Only
    /// one-domain arrays of fixed-width cells are measured; nothing else is claimed.
    fn memory_peak(&mut self, name: &str, t: &Ty, span: Span) {
        let (Some(judge), Some(arr)) = (&self.judge, self.array_id()) else {
            return;
        };
        let Ty::Struct(a, ts, _) = self.shallow(t) else {
            return;
        };
        let (Some(Ty::Dom(i)), Some(elem)) = (ts.first().map(|d| self.shallow(d)), ts.get(1))
        else {
            return;
        };
        let width: u128 = match self.shallow(elem) {
            Ty::Int | Ty::U64 | Ty::Dom(_) => 8,
            Ty::Bool | Ty::Char => 1,
            _ => return,
        };
        let (Some(lo), Some(hi)) = self.domains[i].within else {
            return;
        };
        let cells = u128::try_from(hi - lo + 1).unwrap_or(0);
        let (bytes, limit) = (cells * width, u128::from(judge.memory_mb) << 20);
        if a == arr && bytes > limit {
            let (mb, judge) = (judge.memory_mb, judge.name.clone());
            self.err(
                span,
                Msg::verdict(cpl_diag::codes::MEMORY_OVER_PROFILE, format!("`{name}` takes {} MB at the input bounds, over the {mb} MB limit on {judge}", bytes >> 20)),
            );
        }
    }
    pub(super) fn declare_domain(
        &mut self,
        name: &str,
        lo: TExpr,
        end: TExpr,
        within: (Option<i64>, Option<i64>),
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> usize {
        let count = match (&lo.kind, &end.kind) {
            (TK::Int(a), TK::Int(b)) => Some((b - a).max(0)),
            _ => None,
        };
        let first = self.bind(&format!("{name}$lo"), Ty::Int);
        out.push(TStmt::Let(first, lo));
        let len = self.bind(&format!("{name}$len"), Ty::Int);
        // Spec 4: a range whose start is past its end is empty, never a trap.
        let tmp = self.bind(&format!("{name}$n"), Ty::Int);
        let var = |r| TExpr {
            kind: TK::Var(r),
            ty: Ty::Int,
            span,
        };
        let n = TExpr {
            kind: TK::Bin(
                BinOp::Sub,
                Box::new(end),
                Box::new(var(first)),
                Overflow::Traps,
            ),
            ty: Ty::Int,
            span,
        };
        out.push(TStmt::Let(tmp, n));
        let v = |k| TExpr {
            kind: k,
            ty: Ty::Int,
            span,
        };
        let neg = TExpr {
            kind: TK::Cmp(
                CmpOp::Lt,
                Box::new(v(TK::Var(tmp))),
                Box::new(v(TK::Int(0))),
            ),
            ty: Ty::Bool,
            span,
        };
        out.push(TStmt::Let(
            len,
            v(TK::If(
                Box::new(neg),
                Box::new(v(TK::Int(0))),
                Box::new(v(TK::Var(tmp))),
            )),
        ));
        let info = DomInfo {
            name: name.to_string(),
            len,
            lo: first,
            within,
            owner: self.decl,
            count,
        };
        if let (None, Some(&Ty::Dom(i))) = (self.decl, self.d.top_doms.get(name)) {
            if self.domains[i].name.is_empty() {
                self.domains[i] = info;
                self.doms.insert(name.to_string(), Ty::Dom(i));
                return i;
            }
        }
        self.domains.push(info);
        self.doms
            .insert(name.to_string(), Ty::Dom(self.domains.len() - 1));
        self.domains.len() - 1
    }
    /// A product's range factor is a domain of its own, declared here; the product then names it.
    fn declare_factors(
        &mut self,
        product: &str,
        def: &ast::Expr,
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> ast::Expr {
        let xs: Vec<&ast::Expr> = match &def.kind {
            ExprKind::Tuple(xs) => xs.iter().collect(),
            ExprKind::Record(fs) => fs.iter().map(|(_, x)| x).collect(),
            _ => return def.clone(),
        };
        let mut parts = Vec::new();
        for (i, &x) in xs.iter().enumerate() {
            let ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step: None,
            } = &x.kind
            else {
                parts.push(x.clone());
                continue;
            };
            let name = decls::factor_name(product, i);
            let (lo, end, within) = self.domain_ends(lo, hi, *inclusive);
            self.declare_domain(&name, lo, end, within, span, out);
            parts.push(ast::Expr {
                kind: ExprKind::Name(name),
                span: x.span,
            });
        }
        let kind = match &def.kind {
            ExprKind::Record(fs) => {
                ExprKind::Record(fs.iter().map(|(n, _)| n.clone()).zip(parts).collect())
            }
            _ => ExprKind::Tuple(parts),
        };
        ast::Expr {
            kind,
            span: def.span,
        }
    }
    pub(super) fn plus_one(e: &ast::Expr) -> ast::Expr {
        Self::and_one(e, BinOp::Add)
    }
    pub(super) fn minus_one(e: &ast::Expr) -> ast::Expr {
        Self::and_one(e, BinOp::Sub)
    }
    fn and_one(e: &ast::Expr, op: BinOp) -> ast::Expr {
        let one = ast::Expr {
            kind: ExprKind::Int(1),
            span: e.span,
        };
        ast::Expr {
            kind: ExprKind::Binary {
                op,
                op_span: e.span,
                lhs: Box::new(e.clone()),
                rhs: Box::new(one),
            },
            span: e.span,
        }
    }
    /// `[lo..hi]T` and `[n]T` index by a range with no name: each gets a domain of its own, declared in `out`.
    pub(super) fn name_ranges(
        &mut self,
        t: &ast::Type,
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> ast::Type {
        let mut t = t.clone();
        match &mut t.kind {
            TypeKind::Array { index, elem } => {
                **elem = self.name_ranges(elem, span, out);
                if index.len() == 1 {
                    let zero = || ast::Expr {
                        kind: ExprKind::Int(0),
                        span,
                    };
                    let ends = match &index[0].kind {
                        TypeKind::Range {
                            lo: Some(lo),
                            hi: Some(hi),
                            inclusive,
                        } => {
                            // Its cells count from a number; one from a value is a domain (R04.3.4).
                            if lo.int_literal().is_none() {
                                self.err(
                                    span,
                                    Msg::verdict(cpl_diag::codes::TYPE_MISMATCH, "an array over a range that starts at a value: name it as a domain, `domain D = lo..hi`, then `[D]T`"),
                                );
                            }
                            Some(((**lo).clone(), (**hi).clone(), *inclusive))
                        }
                        TypeKind::Named { name, args }
                            if args.is_empty()
                                && self.lookup(&name.text).is_some()
                                && self.domain_named(&name.text).is_none() =>
                        {
                            Some((
                                zero(),
                                ast::Expr {
                                    kind: ExprKind::Name(name.text.clone()),
                                    span: name.span,
                                },
                                false,
                            ))
                        }
                        _ => None,
                    };
                    if let Some((lo, hi, inclusive)) = ends {
                        let (lo, end, within) = self.domain_ends(&lo, &hi, inclusive);
                        let name = format!("$range{}", self.domains.len());
                        self.declare_domain(&name, lo, end, within, span, out);
                        index[0] = ast::Type {
                            kind: TypeKind::Named {
                                name: ast::Name {
                                    text: name,
                                    span: index[0].span,
                                },
                                args: Vec::new(),
                            },
                            span: index[0].span,
                        };
                    }
                }
            }
            TypeKind::Named { args, .. } => {
                for a in args.iter_mut() {
                    *a = self.name_ranges(a, span, out);
                }
            }
            // The ends are values: they are worked out once, here, and the range gets a name.
            TypeKind::Range { lo, hi, inclusive }
                if decls::static_range(lo, hi, *inclusive).is_none() =>
            {
                let name = format!("$ends{}", self.ranges.len());
                let mut end =
                    |cx: &mut Self, e: &Option<Box<ast::Expr>>, which: &str, open: bool| {
                        e.as_ref().map(|b| {
                            let b = if open {
                                Self::minus_one(b)
                            } else {
                                (**b).clone()
                            };
                            let b = cx.expr(&b, Some(&Ty::Int));
                            let b = cx.coerce(b, &Ty::Int);
                            let v = cx.bind(&format!("{name}${which}"), Ty::Int);
                            out.push(TStmt::Let(v, b));
                            v
                        })
                    };
                let lo = end(self, lo, "lo", false);
                let hi = end(self, hi, "hi", !*inclusive);
                self.ranges.push(RangeInfo { lo, hi });
                self.doms
                    .insert(name.clone(), Ty::RangeOf(self.ranges.len() - 1));
                t.kind = TypeKind::Named {
                    name: ast::Name {
                        text: name,
                        span: t.span,
                    },
                    args: Vec::new(),
                };
            }
            _ => {}
        }
        t
    }
    /// `with ctx(f: v, ..)`: the fields it sets, each to a constant (R09.1.2); `None` when `ctx`
    /// names no context.
    fn ctx_frames(&mut self, ctx: &ast::Name, args: &[ast::Arg]) -> Option<Vec<super::CtxFrame>> {
        let d = self.d;
        let Some(decls::Named::Context(id)) = d.named(self.module, &ctx.text) else {
            return None;
        };
        let c = &d.contexts[id.0];
        let mut fs = Vec::new();
        for a in args {
            let field = a.name.as_ref().map(|n| n.text.clone()).unwrap_or_default();
            if !c.fields.iter().any(|x| x.name.text == field) {
                let code = cpl_diag::codes::UNKNOWN_ARG_NAME;
                let msg =
                    Msg::verdict(code, format!("context `{}` has no field `{field}`", c.name));
                self.err(a.value.span, msg);
            } else if self.const_value(&a.value).is_none() {
                let code = cpl_diag::codes::CONTEXT_NOT_CONSTANT;
                let msg = Msg::verdict(code, format!("`{}.{field}` is set to a constant", c.name));
                self.err(a.value.span, msg);
            } else {
                fs.push(super::CtxFrame {
                    ctx: id,
                    field,
                    value: a.value.clone(),
                });
            }
        }
        Some(fs)
    }

    pub(super) fn body_ty(&mut self, t: &ast::Type, span: Span, out: &mut Vec<TStmt>) -> Ty {
        self.body_ty_open(t, &[], span, out)
    }
    /// `t` read with `holes` as type parameters past the fn's own, each then a fresh variable
    /// for the checker to infer.
    pub(super) fn body_ty_open(
        &mut self,
        t: &ast::Type,
        holes: &[String],
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> Ty {
        let t = self.name_ranges(t, span, out);
        let mut errs = Vec::new();
        let mut generics = self.generics.clone();
        generics.extend(holes.iter().cloned());
        let t = decls::body_type(self.d, self.module, &generics, &self.doms, &t, &mut errs);
        for e in errs {
            self.err(span, e);
        }
        let t = self.fresh_sets(&t);
        if holes.is_empty() {
            return t;
        }
        let tys: Vec<Ty> = (0..self.generics.len())
            .map(Ty::Param)
            .chain(holes.iter().map(|_| self.new_ty()))
            .collect();
        Subst {
            tys: &tys,
            sets: &[],
        }
        .ty(&t)
    }
    /// `v`, after a check that it is between the two ends, each inclusive.
    pub(super) fn checked_between(
        &mut self,
        v: TExpr,
        lo: Option<TExpr>,
        hi: Option<TExpr>,
        span: Span,
    ) -> TExpr {
        let kind = if matches!(v.kind, TK::ReadInt) {
            CheckKind::ReadRange
        } else {
            CheckKind::RangeStore
        };
        self.scopes.push(HashMap::new());
        let tmp = self.bind("$ranged", Ty::Int);
        let var = TExpr {
            kind: TK::Var(tmp),
            ty: Ty::Int,
            span,
        };
        let mut stmts = vec![TStmt::Let(tmp, v)];
        for (bound, op) in [(lo, CmpOp::Ge), (hi, CmpOp::Le)] {
            if let Some(b) = bound {
                let c = TExpr {
                    kind: TK::Cmp(op, Box::new(var.clone()), Box::new(b)),
                    ty: Ty::Bool,
                    span,
                };
                stmts.push(TStmt::Assert(
                    c,
                    "value outside its range".to_string(),
                    span,
                    kind,
                ));
            }
        }
        self.scopes.pop();
        TExpr {
            kind: TK::Block(stmts, Some(Box::new(var))),
            ty: Ty::Int,
            span,
        }
    }
    /// `read()` at a known type.
    pub(super) fn read_as(&mut self, t: &Ty, span: Span) -> TExpr {
        match self.shallow(t) {
            Ty::Struct(d, ts, _)
                if Some(d) == self.array_id()
                    && self.shallow(&ts[1]) == Ty::Int
                    && self.d.core_fn("read_array").is_some() =>
            {
                let Some(read_array) = self.d.core_fn("read_array") else {
                    return TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    };
                };
                let f = self.fnref(read_array, vec![ts[0].clone()], Vec::new());
                let ty = self.ret_ty(&f);
                TExpr {
                    kind: TK::Call {
                        f,
                        recv: None,
                        args: vec![],
                    },
                    ty,
                    span,
                }
            }
            want @ (Ty::Range(..) | Ty::RangeOf(_)) => {
                let v = self.read_as(&Ty::Int, span);
                self.coerce(v, &want)
            }
            // A type already reported reads nothing, so nothing downstream reports it again.
            Ty::Error => TExpr {
                kind: TK::Error,
                ty: Ty::Error,
                span,
            },
            Ty::Int | Ty::Var(_) => {
                self.unify(t, &Ty::Int);
                TExpr {
                    kind: TK::ReadInt,
                    ty: Ty::Int,
                    span,
                }
            }
            Ty::Struct(d, ts, sets) if self.d.tuples.get(&ts.len()) == Some(&d) => {
                let parts = ts.iter().map(|p| self.read_as(p, span)).collect();
                TExpr {
                    kind: TK::Struct(parts),
                    ty: Ty::Struct(d, ts, sets),
                    span,
                }
            }
            other => {
                self.err(
                    span,
                    Unbuilt::Read {
                        ty: self.show(&other),
                    },
                );
                TExpr {
                    kind: TK::Error,
                    ty: Ty::Error,
                    span,
                }
            }
        }
    }
    /// `i` as a position of the domain `dty`; a literal that is never one is E0306 (R04.4.25).
    pub(super) fn position(&mut self, dty: Ty, i: TExpr) -> TExpr {
        let span = i.span;
        if let (Ty::Dom(d), TK::Int(k)) = (&dty, &i.kind) {
            let (least, most) = self.domains[*d].within;
            if least.is_some_and(|l| *k < l) || most.is_some_and(|m| *k > m) {
                let name = self.domains[*d].name.clone();
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::RANGE_STORE_CONST,
                        format!("`{k}` is never a position of `{name}`"),
                    ),
                );
            }
        }
        TExpr {
            kind: TK::DomFrom(dty.clone(), Box::new(i)),
            ty: dty,
            span,
        }
    }
    /// An `Int` used to index an array is checked against the array's domain. The proof that removes the check is the facts layer's.
    /// A declared domain, or a type that implements core's `Domain`.
    pub(super) fn is_domain(&self, t: &Ty) -> bool {
        match t {
            Ty::Dom(_) => true,
            Ty::Struct(..) => (self.d.domain_trait).is_some_and(|tr| self.d.has_impl(t, tr)),
            _ => false,
        }
    }
    pub(super) fn index_arg(&mut self, base: &Ty, i: TExpr) -> TExpr {
        if let Ty::Struct(d, ts, _) = self.shallow(base) {
            if Some(d) == self.array_id() && self.shallow(&i.ty) == Ty::Int {
                let dty = self.shallow(&ts[0]);
                if matches!(dty, Ty::Dom(_) | Ty::Param(_)) {
                    let span = i.span;
                    // A named domain's positions are never raw `Int`s; a literal converts (R02.8.1, R04.8.8).
                    if let Ty::Dom(d) = &dty {
                        let named = !self.domains[*d].name.starts_with('$');
                        if named && !matches!(i.kind, TK::Int(_) | TK::Neg(_)) {
                            let name = self.domains[*d].name.clone();
                            self.err(
                                span,
                                Msg::verdict(
                                    cpl_diag::codes::WRONG_DOMAIN,
                                    format!(
                                        "an `Int` indexes a `[{name}]` array: write `{name}(i)`"
                                    ),
                                ),
                            );
                        }
                    }
                    return self.position(dty, i);
                }
            }
        }
        // A position of one domain never indexes an array over another (R04.3.1).
        if let Some(k) = self.key_of(base) {
            let (k, t) = (self.zonk(&k), self.zonk(&i.ty));
            if k != t && self.is_domain(&k) && self.is_domain(&t) {
                let (ks, ts) = (self.show(&k), self.show(&t));
                self.err(
                    i.span,
                    Msg::verdict(
                        cpl_diag::codes::WRONG_DOMAIN,
                        format!("a `{ts}` indexes a `[{ks}]` array: index it with a `{ks}`"),
                    ),
                );
                return TExpr {
                    kind: TK::Error,
                    ty: Ty::Error,
                    span: i.span,
                };
            }
        }
        // Keys never wrap (R07.2.17).
        if let TK::Neg(k) = &i.kind {
            if let TK::Int(k) = k.kind {
                self.err(
                    i.span,
                    Msg::verdict(
                        cpl_diag::codes::NEGATIVE_LITERAL_INDEX,
                        format!("`-{k}` is never a key: keys do not wrap; count from `.len`"),
                    ),
                );
            }
        }
        i
    }
    /// An expression that names storage: a variable, or a field of one.
    /// The name of the `let` a place lies in, if it lies in one.
    pub(super) fn let_root(&self, place: &TExpr) -> Option<String> {
        let mut root = place;
        while let TK::Field(b, _) = &root.kind {
            root = b;
        }
        let TK::Var(v) = &root.kind else { return None };
        let (fixed, name) = match v {
            VarRef::Local(i) => (self.locals[*i].fixed, &self.locals[*i].name),
            VarRef::Global(i) => (self.globals[*i].fixed, &self.globals[*i].name),
        };
        (fixed && !name.starts_with('$')).then(|| name.clone())
    }

    /// `lend mut place` in a lender of `t`s: `place` is what its caller's body runs on.
    fn lend_out(&mut self, place: &ast::Expr, t: &Ty, span: Span, out: &mut Vec<TStmt>) {
        let p = self.place(place);
        if !self.unify(&p.ty, t) {
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    format!("expected `{}`, found `{}`", self.show(t), self.show(&p.ty)),
                ),
            );
        }
        out.push(TStmt::LendOut(p));
    }

    pub(super) fn place(&mut self, e: &ast::Expr) -> TExpr {
        let v = self.expr_as_typed(e, None);
        if !matches!(
            v.kind,
            TK::Var(_) | TK::Field(..) | TK::Slot(..) | TK::Error
        ) {
            self.err(
                e.span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    "this is not a place that can be assigned or lent",
                ),
            );
        }
        v
    }
}
