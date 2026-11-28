//! Statements, scopes, ranges and reads.

use super::place::{place_binding, place_held, InPlace};
use super::*;
use cpl_next_ir::unbuilt::Unbuilt;

impl<'a> Cx<'a> {
    /// The innermost scope.
    pub(super) fn scope(&mut self) -> &mut HashMap<String, usize> {
        if self.b.scopes.is_empty() {
            self.b.scopes.push(HashMap::new());
        }
        let last = self.b.scopes.len() - 1;
        &mut self.b.scopes[last]
    }
    pub(super) fn add_local(&mut self, name: &str, ty: Ty) -> LocalId {
        self.b.locals.push(Local {
            name: name.to_string(),
            ty,
            top: false,
            mutable: false,
            fixed: false,
            let_at: None,
        });
        let id = self.b.locals.len() - 1;
        self.scope().insert(name.to_string(), id);
        id
    }
    /// A new binding: a global at top level, a local in a fn.
    pub(super) fn bind(&mut self, name: &str, ty: Ty) -> VarRef {
        if self.b.decl.is_none() && self.b.scopes.len() == 1 {
            self.globals.push(Local {
                name: name.to_string(),
                ty,
                top: true,
                mutable: false,
                fixed: false,
                let_at: None,
            });
            self.global_names
                .insert(name.to_string(), self.globals.len() - 1);
            VarRef::Global(self.globals.len() - 1)
        } else if self.b.decl.is_none() {
            // A binding inside a top-level block lives in the globals too, under its scope.
            self.globals.push(Local {
                name: name.to_string(),
                ty,
                top: false,
                mutable: false,
                fixed: false,
                let_at: None,
            });
            let id = self.globals.len() - 1;
            self.scope().insert(name.to_string(), id);
            VarRef::Global(id)
        } else {
            VarRef::Local(self.add_local(name, ty))
        }
    }
    /// A slot the checker made and holds by its id: no name in scope reaches it. `role` is
    /// only what a trace shows; storage with no name is a temporary to ownership.
    pub(super) fn slot(&mut self, ty: Ty, role: &str) -> VarRef {
        let local = Local {
            name: role.to_string(),
            ty,
            top: self.b.decl.is_none() && self.b.scopes.len() == 1,
            mutable: false,
            fixed: false,
            let_at: None,
        };
        match self.b.decl {
            None => {
                self.globals.push(local);
                VarRef::Global(self.globals.len() - 1)
            }
            Some(_) => {
                self.b.locals.push(local);
                VarRef::Local(self.b.locals.len() - 1)
            }
        }
    }
    pub(super) fn lookup(&self, name: &str) -> Option<(VarRef, Ty)> {
        for (depth, s) in self.b.scopes.iter().enumerate().rev() {
            if let Some(id) = s.get(name) {
                return Some(if self.b.decl.is_none() {
                    let _ = depth;
                    (VarRef::Global(*id), self.globals[*id].ty.clone())
                } else {
                    (VarRef::Local(*id), self.b.locals[*id].ty.clone())
                });
            }
        }
        // A field of `self` is nearer than a top-level binding, and only the program sees its own.
        if self.self_field(name).is_some() || self.b.module != self.d.program {
            return None;
        }
        self.global_names
            .get(name)
            .map(|id| (VarRef::Global(*id), self.globals[*id].ty.clone()))
    }
    pub(super) fn self_expr(&self, span: Span) -> Option<TExpr> {
        let id = self.b.self_local?;
        Some(TExpr {
            kind: TK::Var(VarRef::Local(id)),
            ty: self.b.locals[id].ty.clone(),
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
        // `for x in g.out(u)` through a bound on `g`'s type parameter; a bound's value
        // (`s.keys`) walks as the value it returns.
        let yields = |tr: &TraitRef, m: &str| {
            self.d
                .member(&self.d.t(tr.tr).methods, m)
                .is_some_and(|m| self.d.f(m).region.is_some())
        };
        call = match call.kind {
            TK::CallTrait {
                tr,
                method,
                recv: Some(recv),
                args,
                ..
            } if yields(&tr, &method) => {
                return self.trait_walk((tr, method, recv, args), name, body, span, out);
            }
            kind => TExpr { kind, ..call },
        };
        if !matches!(&call.kind, TK::Call { f, .. } if self.d.f(f.decl).region.is_some()) {
            // A type without its own `each` walks through its `Seq` impl (R07.1.1).
            let rt = self.shallow(&call.ty);
            if let Ty::Param(p) = rt {
                return self.bound_walk(p, call, name, body, span, out);
            }
            self.b.allow_region = true;
            let own = "each";
            let walk = match self.d.lookup_member(&rt, own) {
                Some(_) => own.to_string(),
                None => (self.d.iteration_walk()).map_or_else(String::new, |(n, _)| n.to_string()),
            };
            call = self.method_call(call, &walk, vec![], span, false);
            self.b.allow_region = false;
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
        self.b.scopes.push(HashMap::new());
        let var = self.bind(name, elem);
        let body = self.block(body, None);
        self.b.scopes.pop();
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
        let Some(m) = self.d.member(&self.d.t(tr.tr).methods, &method) else {
            return self.err(
                span,
                Msg::internal("a bound's walk names no fn of its trait"),
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
        let (TArg::Value(r) | TArg::Mut(r)) = recv.as_ref();
        let mut tys = vec![r.ty.clone()];
        tys.extend(tr.args.iter().cloned());
        let elem = Subst {
            tys: &tys,
            sets: &[],
        }
        .ty(&elem);
        self.b.scopes.push(HashMap::new());
        let var = self.bind(name, elem);
        let body = self.block(body, None);
        self.b.scopes.pop();
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
        let walk = self.d.iteration_walk().map(|(n, m)| (n.to_string(), m));
        let found = (self.bound_traits(p).into_iter())
            .find(|tr| Some(tr.tr) == self.d.roles.iteration)
            .zip(walk);
        let Some((tr, (walk, m))) = found else {
            let shown = self.show(&recv.ty);
            return self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::UNKNOWN_FIELD,
                    format!("`{shown}` is not bound by the iteration role's trait (R11.6.1)"),
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
        self.b.scopes.push(HashMap::new());
        let var = self.bind(name, elem);
        let body = self.block(body, None);
        self.b.scopes.pop();
        out.push(TStmt::Region {
            callee: Callee::Trait(tr.clone(), walk),
            recv: Some(Box::new(TArg::Value(recv))),
            args: Vec::new(),
            var,
            lend: false,
            body,
        });
    }
    pub(super) fn block(&mut self, b: &ast::Block, want: Option<&Ty>) -> TExpr {
        self.b.scopes.push(HashMap::new());
        let mut out = Vec::new();
        let mut value = None;
        let outer = self.b.group.len();
        let (doms_at, locals_at) = (self.b.block_doms.len(), self.b.locals.len());
        let globals_at = self.globals.len();
        let checks_kept = self.checks_kept;
        let frames = self.b.frames.len();
        let memo = cpl_next_decls::memo_block(&b.stmts, &|w| self.d.memo_word(w));
        let (stmts, memos) = match &memo {
            Some((s, m)) => (&s[..], &m[..]),
            None => (&b.stmts[..], &[][..]),
        };
        self.declare_nested(stmts);
        let mut placed = 0;
        for (i, s) in stmts.iter().enumerate() {
            if i < placed {
                continue;
            }
            // A memo's stmts are its table, its body, `f`, then `f.clear`, each bound by now.
            if let Some(&(at, _)) = memos.iter().find(|(at, _)| at + 4 == i) {
                let bound = |s: &ast::Stmt| match &s.kind {
                    StmtKind::Fn(f) => self.lookup(&f.name.text).map(|(v, _)| v),
                    _ => None,
                };
                if let (Some(f), Some(clear)) = (bound(&stmts[at + 2]), bound(&stmts[at + 3])) {
                    self.b.memo_clears.insert(f, clear);
                }
            }
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
                        self.checks_kept = w;
                        out.push(TStmt::Checks(w));
                    }
                    continue;
                } else if let Some(fs) = self.ctx_frames(ctx, args) {
                    self.b.frames.extend(fs);
                    continue;
                }
            }
            // `let p = mut a[i]` names the cell: the statements through `p`'s last use run while
            // `a`'s `IndexedMut` lends it, `p` that cell (R05.3.4). A block value the lend would
            // hold is refused below with the bare `mut`.
            let unit = want.is_none_or(|w| self.shallow(w) == Ty::Unit);
            let named = place_binding(s)
                .map(|n| (place_held(&n.0, &stmts[i + 1..]), n))
                .filter(|(held, _)| unit || i + 1 + held < stmts.len());
            if let Some((held, named)) = named {
                let rest = ast::Block {
                    stmts: stmts[i + 1..i + 1 + held].to_vec(),
                    span: b.span,
                };
                let inside: InPlace<'_> = Box::new(move |this, body| {
                    let v = this.block(&rest, None);
                    body.push(TStmt::Expr(v));
                });
                self.place_rest(named, s.span, inside, &mut out);
                placed = i + 1 + held;
                continue;
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
        self.checks_kept = checks_kept;
        self.b.frames.truncate(frames);
        self.captures_set(outer, &out);
        self.b.group.truncate(outer);
        self.b.scopes.pop();
        let ty = value
            .as_ref()
            .map(|v: &TExpr| v.ty.clone())
            .unwrap_or(Ty::Unit);
        let mine = self.b.block_doms.split_off(doms_at);
        if !mine.is_empty() {
            // R02.9.2: a value whose type mentions a domain declared here leaves neither as
            // the block's value nor in a binding made before the block.
            let local = |t: &Ty| {
                let mut ds = Vec::new();
                super::domain::doms_in(t, &mut ds);
                ds.into_iter().find(|d| mine.contains(d))
            };
            let mut found = local(&self.zonk(&ty)).map(|d| (d, "as the block's value".to_string()));
            let before = self.b.locals[..locals_at]
                .iter()
                .chain(&self.globals[..globals_at]);
            for l in before {
                if found.is_some() {
                    break;
                }
                found = local(&self.zonk(&l.ty))
                    .map(|d| (d, format!("in `{}`, made before it", l.name)));
            }
            if let Some((d, how)) = found {
                let msg = cpl_diag::msgs::check::DOMAIN_ESCAPES_A_VALUE_OF_FRESH;
                let text = msg.say(&[&self.domains[d].name, &how]);
                self.err(b.span, Msg::verdict(cpl_diag::codes::DOMAIN_ESCAPES, text));
            }
        }
        TExpr {
            kind: TK::Block(out, value.map(Box::new)),
            ty,
            span: b.span,
        }
    }
    /// `inside`, the statements after `let p = mut a[i]` through `p`'s last use, run while
    /// `a`'s `IndexedMut` lends cell `i`, with `p` naming that cell (R05.3.4).
    pub(super) fn place_rest(
        &mut self,
        (n, place): (String, &ast::Expr),
        at: Span,
        inside: InPlace<'_>,
        out: &mut Vec<TStmt>,
    ) {
        let target = self.hoist_indices(place, out);
        let ExprKind::Index { base, indices } = &target.kind else {
            return;
        };
        // `p` lends all of `a`, so a literal key is held as any other is: no literal-key
        // split lets `a[0]` be read beside it.
        let key = match &indices[0].kind {
            ExprKind::Int(_) => {
                let v = self.expr(&indices[0], Some(&Ty::Int));
                self.hold(v, "$idx", indices[0].span, out)
            }
            _ => indices[0].clone(),
        };
        self.through_cell(
            (base, &key),
            at,
            out,
            Box::new(move |this, cell, body| {
                let ExprKind::Name(c) = &cell.kind else {
                    return;
                };
                let id = this.b.scopes.iter().rev().find_map(|s| s.get(c).copied());
                if let (Some(id), Some(scope)) = (id, this.b.scopes.last_mut()) {
                    scope.insert(n, id);
                }
                inside(this, body);
            }),
        );
    }
    /// Top-level statements, each from its own item, with what a `let p = mut a[i]` among them
    /// holds run inside its lend.
    pub(super) fn items_placed(&mut self, items: &[(ItemId, ast::Stmt)], out: &mut Vec<TStmt>) {
        let mut k = 0;
        while let Some((off, s)) = items.get(k) {
            self.b.item = off.clone();
            out.push(TStmt::Item(off.clone()));
            k += 1;
            let Some(named) = place_binding(s) else {
                self.stmt(s, out);
                continue;
            };
            let stmts: Vec<ast::Stmt> = items[k..].iter().map(|(_, s)| s.clone()).collect();
            let held = place_held(&named.0, &stmts);
            let rest = items[k..k + held].to_vec();
            let inside: InPlace<'_> = Box::new(move |this, body| {
                this.b.scopes.push(HashMap::new());
                this.items_placed(&rest, body);
                this.b.scopes.pop();
            });
            self.place_rest(named, s.span, inside, out);
            k += held;
        }
    }
    /// A condition, in a scope of its own that holds the names its `let`s bind: the caller
    /// checks what the condition guards, then pops the scope.
    pub(super) fn cond(&mut self, c: &Cond) -> TExpr {
        self.b.scopes.push(HashMap::new());
        self.cond_in(c)
    }
    /// An expression a condition tests: a `Bool`, never an integer or optional read as one (R02.11.1).
    pub(super) fn bool_cond(&mut self, e: &ast::Expr) -> TExpr {
        if self.question_in_cond(e) {
            return Self::error_node(e.span);
        }
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
                    if !self.mentions_error(&v.ty) {
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
        // A tuple pattern reads a record's fields in order (R02.5.5): `let (tree, used) =
        // kruskal(..)` of `-> (tree: Rooted<V, _, _>, used: [E]Bool)`.
        let fields = match self.lookup(&tmp).map(|(_, t)| self.shallow(&t)) {
            Some(Ty::Struct(d, ..)) if self.d.s(d).record => self.d.field_names(d),
            _ => Vec::new(),
        };
        for (field, p) in parts {
            let field = (field.parse::<usize>().ok())
                .and_then(|k| fields.get(k).cloned())
                .unwrap_or(field);
            if matches!(p.kind, PatternKind::Wildcard) {
                continue;
            }
            let named = match &p.kind {
                PatternKind::Name(n) => Some((n.clone(), field.clone())),
                _ => None,
            };
            let at = |kind| ast::Expr { kind, span: p.span };
            let part = at(ExprKind::Field {
                base: Box::new(at(ExprKind::Name(tmp.clone()))),
                name: ast::Name {
                    text: field,
                    span: p.span,
                },
            });
            // `domain I` names the result's `domain C` afresh: the rest of the value is over `I`.
            if let PatternKind::Domain(name) = &p.kind {
                let v = self.expr(&part, None);
                match self.bind_fresh(&name.text, &v.ty, p.span, out) {
                    Some((c, i)) => {
                        if let Some((VarRef::Local(k) | VarRef::Global(k), t)) = self.lookup(&tmp) {
                            let t = crate::pattern::retag(&self.zonk(&t), c, i);
                            match self.b.decl.is_none() {
                                true => self.globals[k].ty = t,
                                false => self.b.locals[k].ty = t,
                            }
                        }
                    }
                    None => self.err(p.span, Unbuilt::DomainPattern),
                }
                continue;
            }
            let mut one = s.clone();
            if let StmtKind::Let { pat, ty, init, .. } = &mut one.kind {
                *pat = p;
                *ty = None;
                *init = Some(part);
            }
            self.stmt(&one, out);
            // `tree.Pre` is the domain the whole value owns as its `tree.Pre`.
            if let Some((n, field)) = named {
                self.hand_down(&tmp, &field, &n);
            }
        }
    }
    pub(super) fn stmt(&mut self, s: &ast::Stmt, out: &mut Vec<TStmt>) {
        let (mark, at) = (self.b.list_doms.len(), out.len());
        let outer = std::mem::replace(&mut self.b.rhs_extent, s.rhs_extent());
        self.stmt_kind(s, out);
        self.b.rhs_extent = outer;
        let mut pre = Vec::new();
        for (d, n, span) in self.b.list_doms.split_off(mark) {
            if !matches!(self.shallow(&d), Ty::Var(_)) {
                continue;
            }
            let int = |k: usize| ast::Expr {
                kind: ExprKind::Int(k as _),
                span,
            };
            let (lo, end, within, _) = self.domain_ends(&int(0), &int(n), false);
            let name = format!("$range{}", self.domains.len());
            let i = self.declare_domain(&name, lo, end, within, span, &mut pre);
            self.unify(&d, &Ty::Dom(i));
        }
        out.splice(at..at, pre);
    }
    fn stmt_kind(&mut self, s: &ast::Stmt, out: &mut Vec<TStmt>) {
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
                    let (tys, call) = match (ty.as_ref().map(|t| &t.kind), self.per_part_call(init))
                    {
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
                let mut cell = None;
                while let (Some(Ty::Struct(d, ts, _)), Some(arr)) =
                    (cell.as_ref().or(declared.as_ref()), self.array_id())
                {
                    if !literal || *d != arr {
                        break;
                    }
                    cell = Some(ts[1].clone());
                }
                if !mutable && matches!(init.kind, ExprKind::Lambda { .. }) {
                    self.b.recur = Some(name.clone());
                }
                let v = self.expr(init, cell.as_ref().or(declared.as_ref()));
                self.b.recur = None;
                // `var a: [V]Int = 0` fills every slot: it is `Array.filled(0)`.
                let fill = match (&declared, self.array_id()) {
                    (Some(array @ Ty::Struct(d, ts, _)), Some(arr))
                        if *d == arr
                            && (!matches!(self.shallow(&v.ty), Ty::Struct(..))
                                || self.fits(&v.ty, &ts[1])) =>
                    {
                        Some(array.clone())
                    }
                    _ => None,
                };
                let v = match fill {
                    Some(array) => self.fill(v, array, s.span),
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
                if !*mutable && self.is_none_value(&v) {
                    self.none_lets.insert(var);
                }
                if let (false, Some(r)) =
                    (*mutable, super::unify::known_range(&self.b.let_ranges, &v))
                {
                    self.b.let_ranges.insert(var, r);
                }
                if let Some(keys) = super::index::literal_cut(init) {
                    self.view_keys.insert(var, keys);
                }
                let let_at = (!*mutable).then(|| self.let_keyword(s.span)).flatten();
                match var {
                    VarRef::Local(i) => {
                        (self.b.locals[i].mutable, self.b.locals[i].fixed) = (*mutable, !*mutable);
                        self.b.locals[i].let_at = let_at;
                    }
                    VarRef::Global(i) => {
                        (self.globals[i].mutable, self.globals[i].fixed) = (*mutable, !*mutable);
                        self.globals[i].let_at = let_at;
                    }
                }
                let fresh = self.fresh_args(&v.ty);
                out.push(TStmt::Let(var, v));
                self.own_fresh(name, var, fresh, s.span, out);
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
                    if !self.mentions_error(&v.ty) {
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
                        (self.b.locals[i].mutable, self.b.locals[i].fixed) = (*mutable, !*mutable)
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
                // A written tuple: each part is read into a copy of its own before any place is
                // written, so no part borrows a place the writes change.
                if let ExprKind::Tuple(parts) = &value.kind {
                    if parts.len() == places.len() {
                        let mut temps = Vec::new();
                        for (k, part) in parts.iter().enumerate() {
                            let name = format!("{tmp}_{k}");
                            let v = self.expr(part, None);
                            let (ty, span) = (v.ty.clone(), v.span);
                            let var = self.bind(&name, ty.clone());
                            let kind = TK::Owned(Box::new(v));
                            out.push(TStmt::Let(var, TExpr { kind, ty, span }));
                            temps.push(name);
                        }
                        for (place, name) in places.iter().zip(temps) {
                            let one = ast::Stmt {
                                kind: StmtKind::Assign {
                                    target: place.clone(),
                                    op: ast::AssignOp::Set,
                                    value: at(ExprKind::Name(name), place.span),
                                },
                                span: place.span,
                                ..s.clone()
                            };
                            self.stmt(&one, out);
                        }
                        return;
                    }
                }
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
                let packed = match self.key_ctor(target, out) {
                    Ok(Some(one)) => Some(one),
                    Ok(None) => super::index::packed(target),
                    Err(()) => return,
                };
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
                let kept = self.b.kept.take();
                // `a[i].f = v`: `f` is written in the cell `a`'s `IndexMut` lends (R12.4.6).
                if let Some((at, names)) = super::index::cell_path(target) {
                    let seen = self.errors.len();
                    let probe = self.expr_as_typed(target, None);
                    self.errors.truncate(seen);
                    let mut cell = &probe;
                    while let TK::Field(b, _) = &cell.kind {
                        cell = b;
                    }
                    if !matches!(cell.kind, TK::Slot(..)) {
                        let want = probe.ty.clone();
                        let at = self.hoist_indices(at, out);
                        let value = self.hoist(value, (Some(&want), true), "$rhs", out);
                        let ExprKind::Index { base, indices } = &at.kind else {
                            return;
                        };
                        let (op, span) = (*op, s.span);
                        return self.through_cell(
                            (base, &indices[0]),
                            target.span,
                            out,
                            Box::new(move |this, cell, out| {
                                this.b.kept = kept;
                                let assign = ast::Stmt {
                                    kind: StmtKind::Assign {
                                        target: super::index::on_fields(cell, &names),
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
                    }
                }
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
                        // The value is read whole before the cell is written (R01.6.4).
                        let value = self.hoist(value, (cell.as_ref(), true), "$rhs", out);
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
                                this.b.kept = kept;
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
                    self.b.allow_region = true;
                    let call = self.expr(target, None);
                    self.b.allow_region = false;
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
                    self.b.scopes.push(HashMap::new());
                    let var = self.bind("$lent", elem.clone());
                    let lent = TExpr {
                        kind: TK::Var(var),
                        ty: elem.clone(),
                        span: target.span,
                    };
                    let step = match op {
                        AssignOp::Min | AssignOp::Max => {
                            Self::kept(kept, self.keep(lent, *op == AssignOp::Max, value, s.span))
                        }
                        _ => {
                            let v = self.expr(value, Some(&elem));
                            let v = match op {
                                AssignOp::Binary(b) => {
                                    self.binary(*b, lent.clone(), v, target.span)
                                }
                                _ => v,
                            };
                            TStmt::Assign(lent, self.coerce(v, &elem))
                        }
                    };
                    self.b.scopes.pop();
                    let body = TExpr {
                        kind: TK::Block(vec![step], None),
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
                let place = self.assign_place(target);
                let v = match op {
                    AssignOp::Set => self.expr(value, Some(&place.ty)),
                    AssignOp::Binary(b) => {
                        let v = self.expr(value, Some(&place.ty));
                        self.binary(*b, place.clone(), v, target.span)
                    }
                    AssignOp::Min | AssignOp::Max => {
                        let k = self.keep(place, *op == AssignOp::Max, value, s.span);
                        return out.push(Self::kept(kept, k));
                    }
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
                if self.dropped_question(e) || self.cell_method(e, out) {
                    return;
                }
                let v = self.expr(e, None);
                out.push(TStmt::Expr(v));
            }
            StmtKind::For { .. } | StmtKind::While { .. }
                if self.b.framed != Some(s.span.start) =>
            {
                self.framed_loop(s, out)
            }
            StmtKind::While {
                cond,
                body,
                else_body: None,
            } => {
                let c = self.cond(cond);
                self.b.loops += 1;
                let b = self.block(body, None);
                self.b.loops -= 1;
                self.b.scopes.pop();
                out.push(TStmt::While(c, b));
            }
            StmtKind::For { .. } if !self.b.in_for => {
                self.b.loops += 1;
                self.b.in_for = true;
                self.stmt(s, out);
                self.b.in_for = false;
                self.b.loops -= 1;
            }
            StmtKind::For {
                pat,
                iter,
                body,
                else_body: None,
                ..
            } => {
                self.b.in_for = false;
                // A bare `..` names no domain to walk (R04.4.11).
                if let ExprKind::Range {
                    lo: None,
                    hi: None,
                    step: None,
                    ..
                } = iter.kind
                {
                    let text = cpl_diag::msgs::check::BARE_MEANS_ALL_DIMENSION.say(&[]);
                    let help = "name what to walk: `for x in D` or `for i in 0..n`".to_string();
                    let msg = Msg::verdict(cpl_diag::codes::EXPECTED_EXPR, text);
                    return self.err_help(iter.span, msg, Some(help));
                }
                // `for _ in xs` (and `repeat n:`): each item is bound to a name nothing reads.
                if let PatternKind::Wildcard = pat.kind {
                    let mut named = s.clone();
                    if let StmtKind::For { pat, .. } = &mut named.kind {
                        pat.kind = PatternKind::Name("$each".into());
                    }
                    self.b.in_for = true;
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
                        self.b.scopes.push(HashMap::new());
                        let var = self.bind(name, dty);
                        let b = self.block(body, None);
                        self.b.scopes.pop();
                        out.push(TStmt::For(var, lo, hi, Box::new(b)));
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
                    self.b.allow_region =
                        matches!(iter.kind, ExprKind::Call { .. } | ExprKind::Field { .. });
                    let call = self.expr(iter, None);
                    self.b.allow_region = false;
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
                self.b.scopes.push(HashMap::new());
                let var = self.bind(name, Ty::Int);
                let b = self.block(body, None);
                self.b.scopes.pop();
                out.push(TStmt::For(var, lo, hi, Box::new(b)));
            }
            StmtKind::Return(e) => {
                let ret = self.b.ret.clone();
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
            StmtKind::Domain { name, def, .. }
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
                self.b.doms.insert(name.text.clone(), p);
            }
            StmtKind::Domain { name, def, counted } => {
                let ExprKind::Range {
                    lo: Some(lo),
                    hi: Some(hi),
                    inclusive,
                    step: None,
                } = &def.kind
                else {
                    if self.counted_domain(&name.text, def, s.span, out) {
                        return;
                    }
                    self.err(
                        s.span,
                        Unbuilt::DomainKind {
                            kind: crate::kind_name(&def.kind),
                        },
                    );
                    self.b.doms.insert(name.text.clone(), Ty::Error);
                    self.b.refused_names.insert(name.text.clone());
                    return;
                };
                let (lo, end, within, chars) = self.domain_ends(lo, hi, *inclusive);
                let i = self.declare_domain(&name.text, lo, end, within, s.span, out);
                self.domains[i].chars = chars;
                self.domains[i].counted = *counted;
            }
            StmtKind::Yield { value, from: false } => match self.b.region.clone() {
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
            StmtKind::Lend { place, .. } => match self.b.region.clone() {
                // Every path through a lender reaches one `lend`; inside a loop it could run twice.
                Some((true, _)) if self.b.loops > 0 => self.err(
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
                None if (self.b.decl).is_some_and(|f| self.d.f(f).lends.is_some()) => {
                    let ret = self.b.ret.clone();
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
            StmtKind::Break(label) => self.break_to(label.as_ref(), s.span, out),
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
                    let b = self.block_keeping(w, body, s.span);
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
                    self.checks_kept = w;
                    out.push(TStmt::Checks(w));
                }
            }
            StmtKind::With {
                ctx,
                args,
                body: None,
            } if matches!(
                self.d.named(self.b.module, &ctx.text),
                Some(decls::Named::Context(_))
            ) =>
            {
                if let Some(fs) = self.ctx_frames(ctx, args) {
                    self.b.frames.extend(fs);
                }
            }
            StmtKind::With {
                ctx,
                args,
                body: Some(body),
            } if matches!(
                self.d.named(self.b.module, &ctx.text),
                Some(decls::Named::Context(_))
            ) =>
            {
                if let Some(fs) = self.ctx_frames(ctx, args) {
                    let n = self.b.frames.len();
                    self.b.frames.extend(fs);
                    let b = self.block(body, None);
                    self.b.frames.truncate(n);
                    out.push(TStmt::Expr(b));
                }
            }
            // `unchecked:` turns every check off in its block, in every build (R09.4.3).
            StmtKind::Unchecked(body) => {
                let b = self.block_keeping(Checks::NONE, body, s.span);
                out.push(TStmt::Expr(b));
            }
            // The loader made an inline `mod`'s items a module of their own (R11.1.3).
            StmtKind::Mod { body, .. } if body.stmts.is_empty() => {}
            other => {
                let module = match other {
                    StmtKind::Mod { name, .. } => Some(name.text.as_str()),
                    _ => None,
                };
                if let Some(n) = decls::item_name(s).or(module) {
                    self.b.refused_names.insert(n.to_string());
                    // It shadows what the name meant outside, as the item would: its uses are
                    // the error, not another declaration's.
                    if self.b.decl.is_some() {
                        self.bind(n, Ty::Error);
                    }
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
    /// The checks kept under `with checks(...)` here (R09.4): each field keeps its check on or
    /// turns it off, from the checks kept around it.
    fn checks(&mut self, args: &[ast::Arg], span: Span) -> Option<Checks> {
        let mut kept = self.checks_kept;
        for a in args {
            let on = match &a.value.kind {
                ExprKind::Name(v) if v == "on" => true,
                ExprKind::Name(v) if v == "off" => false,
                _ => {
                    self.err(span, Unbuilt::ChecksValue);
                    return None;
                }
            };
            match a.name.as_ref().and_then(|n| CheckName::named(&n.text)) {
                Some(c) => kept = kept.with(c, on),
                None => {
                    self.err(span, Unbuilt::ChecksField);
                    return None;
                }
            }
        }
        Some(kept)
    }
    /// `body` checked and lowered with `kept` its checks: the block opens by saying so.
    fn block_keeping(&mut self, kept: Checks, body: &ast::Block, span: Span) -> TExpr {
        let outer = std::mem::replace(&mut self.checks_kept, kept);
        let b = self.block(body, None);
        self.checks_kept = outer;
        TExpr {
            kind: TK::Block(vec![TStmt::Checks(kept), TStmt::Expr(b)], None),
            ty: Ty::Unit,
            span,
        }
    }
    /// `f(..)` or `f<T>(..)` for a fn whose parameters all have defaults and that gives its one
    /// type parameter, as `read(base: 1)`: the type argument if written, and the call without it.
    /// Bound to a tuple pattern with a tuple type it is called once per part, with the same
    /// arguments, so a later part's type may name an earlier part (R10.2.3).
    pub(super) fn per_part_call(&self, e: &ast::Expr) -> Option<(Option<ast::Type>, ast::Expr)> {
        let ExprKind::Call { callee, args } = &e.kind else {
            return None;
        };
        let (base, targ) = match &callee.kind {
            ExprKind::Generic { base, args: targs } if targs.len() == 1 => {
                (&**base, Some(targs[0].clone()))
            }
            _ => (&**callee, None),
        };
        let ExprKind::Name(n) = &base.kind else {
            return None;
        };
        let f = self
            .d
            .fn_named(self.b.module, n)
            .filter(|_| self.declared(n))?;
        let fd = self.d.f(f);
        let shape = fd.params.iter().all(|p| p.default.is_some()) && fd.generics.len() == 1;
        (shape && fd.ret == Ty::Param(0)).then(|| {
            let kind = ExprKind::Call {
                callee: Box::new(base.clone()),
                args: args.clone(),
            };
            (targ, ast::Expr { kind, span: e.span })
        })
    }
    /// A domain's size is a hidden variable next to its declaration.
    pub(super) fn array_id(&self) -> Option<StructId> {
        self.d.sugar.array
    }
    /// `Array.filled(v)` through the trait marked `@literal("[;]")`: `var a: [V]Int = 0` (R01.6.1).
    fn fill(&mut self, v: TExpr, array: Ty, span: Span) -> TExpr {
        let Ty::Struct(_, ts, _) = &array else {
            return v;
        };
        let elem = ts[1].clone();
        // `[V][W]Int = 0` fills each row with a filled `[W]Int`.
        let v = match self.shallow(&elem) {
            Ty::Struct(d, ..) if Some(d) == self.array_id() && !self.fits(&v.ty, &elem) => {
                self.fill(v, elem.clone(), span)
            }
            _ => v,
        };
        let v = self.coerce(v, &elem);
        let tr = self.d.literal_trait("[;]");
        let fns = tr.map(|tr| self.d.trait_fns(tr)).unwrap_or_default();
        match (tr, fns.first()) {
            (Some(tr), Some(&(n, m))) => {
                let n = self.d.text(n);
                self.reach(m);
                let tr = decls::TraitRef {
                    tr,
                    args: vec![elem],
                };
                let o = (array.clone(), tr.clone(), span, Origin::Bound);
                self.b.obligations.push(o);
                let kind = TK::CallTrait {
                    ctx: self.trait_ctx(tr.tr, n),
                    tr,
                    method: n.to_string(),
                    recv: None,
                    self_ty: Box::new(array.clone()),
                    targs: Box::new([]),
                    args: vec![TArg::Value(v)],
                };
                TExpr {
                    kind,
                    ty: array,
                    span,
                }
            }
            _ => {
                self.err(
                    span,
                    Msg::internal("no trait is marked `@literal(\"[;]\")`"),
                );
                v
            }
        }
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
        let (Some(key), Some(elem)) = (ts.first().map(|d| self.shallow(d)), ts.get(1)) else {
            return;
        };
        let (lo, hi) = match key {
            Ty::Dom(i) => self.domains[i].within,
            Ty::Range(lo, hi) => (Some(lo), Some(hi)),
            _ => return,
        };
        let width: u128 = match self.shallow(elem) {
            Ty::I128 => 16,
            Ty::Int | Ty::U64 | Ty::Dom(_) => 8,
            Ty::Bool | Ty::Char => 1,
            _ => return,
        };
        let (Some(lo), Some(hi)) = (lo, hi) else {
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
    /// `with ctx(f: v, ..)`: the fields it sets, each to a constant (R09.1.2); `None` when `ctx`
    /// names no context.
    fn ctx_frames(&mut self, ctx: &ast::Name, args: &[ast::Arg]) -> Option<Vec<super::CtxFrame>> {
        let d = self.d;
        let Some(decls::Named::Context(id)) = d.named(self.b.module, &ctx.text) else {
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
        let mut generics = self.b.generics.clone();
        generics.extend(holes.iter().cloned());
        let (t, errs) = decls::body_type(
            self.d,
            self.d,
            self.b.module,
            &generics,
            &self.b.doms,
            &|n: &str| self.owned_named(n),
            &t,
        );
        for e in errs {
            self.err(span, e);
        }
        let t = self.fresh_sets(&t);
        if holes.is_empty() {
            return t;
        }
        let tys: Vec<Ty> = (0..self.b.generics.len())
            .map(Ty::Param)
            .chain(holes.iter().map(|_| self.new_ty()))
            .collect();
        Subst {
            tys: &tys,
            sets: &[],
        }
        .ty(&t)
    }
    /// Whether `v` is input: a value an `@input` fn gives, or a part of one (R10.2.2).
    pub(super) fn is_input(&self, v: &TExpr) -> bool {
        match &v.kind {
            TK::Call { f, .. } => self.d.input_fns.contains(&f.decl),
            TK::Field(b, _) => matches!(&b.kind, TK::Var(t) if self.b.input_parts.contains(t)),
            _ => false,
        }
    }
    /// `v`, after a check that it is between the two ends, each inclusive.
    pub(super) fn checked_between(
        &mut self,
        v: TExpr,
        lo: Option<TExpr>,
        hi: Option<TExpr>,
        span: Span,
    ) -> TExpr {
        let kind = if self.is_input(&v) {
            CheckKind::ReadRange
        } else {
            CheckKind::RangeStore
        };
        self.checked_as(v, (lo, hi), span, kind)
    }
    /// `v`, trapping as `kind` unless it lies within `lo..=hi`.
    pub(super) fn checked_as(
        &mut self,
        v: TExpr,
        (lo, hi): (Option<TExpr>, Option<TExpr>),
        span: Span,
        kind: CheckKind,
    ) -> TExpr {
        self.b.scopes.push(HashMap::new());
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
        self.b.scopes.pop();
        TExpr {
            kind: TK::Block(stmts, Some(Box::new(var))),
            ty: Ty::Int,
            span,
        }
    }
    /// `i` as a position of the domain `dty`; a literal that is never one is E0306 (R04.4.25).
    pub(super) fn position(&mut self, dty: Ty, i: TExpr, kind: CheckKind) -> TExpr {
        let span = i.span;
        let within = match &dty {
            Ty::Dom(d) => Some((self.domains[*d].within, self.domains[*d].name.clone())),
            Ty::Range(lo, hi) => Some(((Some(*lo), Some(*hi)), self.show(&dty))),
            _ => None,
        };
        if let (Some(((least, most), name)), TK::Int(k)) = (within, &i.kind) {
            if least.is_some_and(|l| *k < l) || most.is_some_and(|m| *k > m) {
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
            kind: TK::DomFrom(dty.clone(), Box::new(i), kind),
            ty: dty,
            span,
        }
    }
    /// The Int `i` as a cut of the domain `dty`, the count of positions before it; a literal
    /// that is never one is E0306 (R04.4.25).
    pub(super) fn cut_ordinal(&mut self, dty: Ty, i: TExpr) -> TExpr {
        let span = i.span;
        let within = match &dty {
            Ty::Dom(d) => Some((self.domains[*d].within, self.domains[*d].name.clone())),
            Ty::Range(lo, hi) => Some(((Some(*lo), Some(*hi)), self.show(&dty))),
            _ => None,
        };
        if let (Some(((least, most), name)), TK::Int(k)) = (within, &i.kind) {
            if least.is_some_and(|l| *k < l) || most.is_some_and(|m| *k > m.saturating_add(1)) {
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::RANGE_STORE_CONST,
                        format!("`{k}` is never a cut of `{name}`"),
                    ),
                );
            }
        }
        TExpr {
            kind: TK::CutOrd(dty, Box::new(i)),
            ty: Ty::Int,
            span,
        }
    }
}
