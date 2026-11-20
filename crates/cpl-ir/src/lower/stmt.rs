//! Statements, blocks and patterns.

use super::*;

/// What writing through `x!` says when `x` is `none`; reading it says so too.
const NO_VALUE: &str = "`!` on `none`: there is no value here";

impl<'a> Lowerer<'a> {
    pub(super) fn block(&mut self, block: &ast::Block, tail: Tail) -> Block {
        self.cx.scopes.push(HashMap::new());
        self.cx.local_syms.push(HashMap::new());
        let mut out = Vec::new();
        match block.stmts.split_last() {
            None if tail == Tail::Give => self.needs_value(block.span),
            None => {}
            Some((last, init)) => {
                let mut terminated = false;
                for (idx, stmt) in init.iter().enumerate() {
                    self.stmt_with_rest(stmt, &block.stmts[idx + 1..], &mut out);
                    if matches!(out.last(), Some(Stmt::Return(_))) {
                        terminated = true;
                        break;
                    }
                }
                if !terminated {
                    match &last.kind {
                        ast::StmtKind::Expr(e) if tail != Tail::Discard => self.tail(e, tail, &mut out),
                        _ => {
                            self.stmt(last, &mut out);
                            if tail == Tail::Give {
                                self.needs_value(last.span);
                            }
                        }
                    }
                }
            }
        }
        self.cx.local_syms.pop();
        self.cx.scopes.pop();
        Block { stmts: out }
    }

    fn needs_value(&mut self, span: Span) {
        self.error(
            codes::TYPE_MISMATCH,
            span,
            "this block is used as a value, so it has to end with an expression",
        );
    }

    /// Lowers `e` where its value goes to `tail`. `if`, `match` and inline
    /// blocks pass the destination on to their branches, so a fn that ends
    /// in an `if` returns from each branch.
    pub(super) fn tail(&mut self, e: &ast::Expr, tail: Tail, out: &mut Vec<Stmt>) {
        match &e.kind {
            ast::ExprKind::Paren(inner)
                if matches!(
                    inner.kind,
                    ast::ExprKind::If { .. }
                        | ast::ExprKind::Match { .. }
                        | ast::ExprKind::Block(_)
                ) =>
            {
                self.tail(inner, tail, out)
            }
            ast::ExprKind::If {
                branches,
                otherwise,
            } => match (otherwise, tail) {
                (None, Tail::Give) => {
                    self.error(
                        codes::TYPE_MISMATCH,
                        e.span,
                        "an `if` used as a value needs an `else`",
                    );
                }
                // Without an `else` there is no value on one path: a
                // statement, and the fn's value comes later.
                (None, _) => self.if_chain(branches, None, Tail::Discard, out),
                (Some(otherwise), _) => self.if_chain(branches, Some(otherwise), tail, out),
            },
            ast::ExprKind::Match { scrutinee, arms } => {
                self.match_stmt(e, scrutinee, arms, tail, out)
            }
            ast::ExprKind::Block(block) => out.extend(self.block(block, tail).stmts),
            _ => match tail {
                Tail::Discard => {
                    let value = self.expr(e, None);
                    out.push(Stmt::Expr(value));
                }
                Tail::Return => {
                    let hint = self.cx.ret.clone();
                    let value = self.expr(e, hint.as_ref());
                    match hint {
                        Some(Ty::Unit) => out.push(Stmt::Expr(value)),
                        Some(ty) => {
                            let value = self.coerce(value, &ty, e.span);
                            out.push(Stmt::Return(Some(value)));
                        }
                        None => {
                            self.cx.ret = Some(value.ty.clone());
                            if value.ty == Ty::Unit {
                                out.push(Stmt::Expr(value));
                            } else {
                                out.push(Stmt::Return(Some(value)));
                            }
                        }
                    }
                }
                Tail::Give => {
                    let hint = self.cx.dos.last().and_then(|frame| frame.ty.clone());
                    let mut value = self.expr(e, hint.as_ref());
                    match hint {
                        Some(ty) => value = self.coerce(value, &ty, e.span),
                        None => {
                            if let Some(frame) = self.cx.dos.last_mut() {
                                frame.ty = Some(value.ty.clone());
                            }
                        }
                    }
                    out.push(Stmt::Give(value));
                }
            },
        }
    }

    fn at_top_level(&self) -> bool {
        self.cx.is_main && self.cx.scopes.len() == 1
    }

    pub(super) fn stmt(&mut self, stmt: &ast::Stmt, out: &mut Vec<Stmt>) {
        self.stmt_with_rest(stmt, &[], out);
    }

    pub(super) fn stmt_with_rest(
        &mut self,
        stmt: &ast::Stmt,
        rest: &[ast::Stmt],
        out: &mut Vec<Stmt>,
    ) {
        // `let u = o ?? return -1`: the test and the way out go ahead of
        // the statement, which then reads the value that is there.
        let root = match &stmt.kind {
            ast::StmtKind::Let { init: Some(e), otherwise: None, .. }
            | ast::StmtKind::Assign { value: e, .. }
            | ast::StmtKind::Expr(e)
            | ast::StmtKind::Return(Some(e)) => Some(e),
            _ => None,
        };
        let root = root.map(|mut e| {
            while let ast::ExprKind::Paren(inner) = &e.kind {
                e = inner;
            }
            e
        });
        let way_out = root.filter(|e| diverging_default(e).is_some()).map(|e| e.span);
        let outer = (
            std::mem::replace(&mut self.cx.way_out, way_out),
            std::mem::take(&mut self.cx.pending),
        );
        let at = out.len();
        self.stmt_inner(stmt, rest, out);
        let pending = std::mem::replace(&mut self.cx.pending, outer.1);
        self.cx.way_out = outer.0;
        out.splice(at..at, pending);
    }

    fn stmt_inner(&mut self, stmt: &ast::Stmt, rest: &[ast::Stmt], out: &mut Vec<Stmt>) {
        match &stmt.kind {
            ast::StmtKind::Let {
                mutable,
                pat,
                ty: None,
                init: Some(init),
                otherwise: Some(els_block),
                ..
            } => {
                let Some((present, inner)) = self.optional(init, "let ... else", out) else {
                    return;
                };
                let absent = Expr {
                    kind: ExprKind::Unary(UnOp::Not, Box::new(present)),
                    ty: Ty::Bool,
                    span: self.span(stmt.span),
                };
                let els = self.block(els_block, Tail::Discard);
                out.push(Stmt::If(absent, els, Block::default()));
                self.bind_mut(pat, inner, *mutable, out);
            }
            ast::StmtKind::Let {
                mutable,
                pat,
                ty,
                init,
                ..
            } => {
                // Handle untyped `var m = Map()` / `HashMap()` by peeking subsequent uses in the block.
                let inferred_map_ty = if ty.is_none() {
                    if let (
                        ast::PatternKind::Name(vname),
                        Some(ast::Expr {
                            kind: ast::ExprKind::Call { callee, args },
                            ..
                        }),
                    ) = (&pat.kind, init)
                    {
                        if args.is_empty()
                            && matches!(&callee.kind, ast::ExprKind::Name(n) if n == "Map" || n == "HashMap")
                        {
                            let hashed = matches!(&callee.kind, ast::ExprKind::Name(n) if n == "HashMap");
                            let (kt, vt) = self.peek_map_kv_types(vname, rest);
                            Some(if hashed {
                                Ty::HashMap(Box::new(kt), Box::new(vt))
                            } else {
                                Ty::Map(Box::new(kt), Box::new(vt))
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                let val = if let Some(mty) = inferred_map_ty {
                    Some(self.mk(ExprKind::Tuple(Vec::new()), mty, stmt.span))
                } else {
                    self.let_value_with_rest(ty.as_ref(), init.as_ref(), pat, rest, stmt.span)
                };
                if let Some(value) = val {
                    let single_name = match &pat.kind {
                        ast::PatternKind::Name(n) => Some(n.clone()),
                        ast::PatternKind::Variant { path, args }
                            if path.len() == 1 && args.is_empty() =>
                        {
                            Some(path[0].text.clone())
                        }
                        _ => None,
                    };
                    let runtime_const_name = if !*mutable && !self.at_top_level() && value.ty == Ty::Int {
                        single_name.filter(|n| n.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
                    } else {
                        None
                    };
                    self.bind_mut(pat, value, *mutable, out);
                    if let Some(cname) = runtime_const_name {
                        if let Some(slot) = self.lookup(&cname) {
                            let gid = self.globals.len();
                            self.globals.push(Global {
                                name: format!("{cname}_rt_{gid}"),
                                ty: Ty::Int,
                                span: Span::default(),
                            });
                            let val_ref = self.local(slot, stmt.span);
                            out.push(Stmt::Assign(
                                Place {
                                    kind: PlaceKind::Global(gid),
                                    ty: Ty::Int,
                                    span: self.span(stmt.span),
                                },
                                AssignOp::Set,
                                val_ref,
                            ));
                            self.cx
                                .local_syms
                                .last_mut()
                                .unwrap()
                                .insert(cname, Sym::Global(gid));
                        }
                    }
                }
            }
            ast::StmtKind::Const {
                name,
                generics,
                ty,
                init,
                ..
            } => {
                if !generics.is_empty() {
                    return;
                }
                if let Some(value) = self.let_value(ty.as_ref(), init.as_ref(), stmt.span) {
                    if let ExprKind::Int(v) = value.kind {
                        self.const_ints.insert(name.text.clone(), v);
                    }
                    self.declare(&name.text, value, false, out);
                }
            }
            ast::StmtKind::TypeAlias { name, generics, ty } if !self.at_top_level() => {
                if generics.is_empty() {
                    let resolved = self.resolve_ty(ty);
                    self.cx
                        .local_type_aliases
                        .insert(name.text.clone(), resolved.clone());
                    let sym = match resolved {
                        Ty::Struct(id) => Some(Sym::Struct(id)),
                        Ty::Enum(id) => Some(Sym::Enum(id)),
                        Ty::Pos(id) => Some(Sym::Domain(id)),
                        _ => None,
                    };
                    if let Some(sym) = sym {
                        self.cx
                            .local_syms
                            .last_mut()
                            .unwrap()
                            .insert(name.text.clone(), sym);
                    }
                }
            }
            ast::StmtKind::Impl {
                trait_name: Some(tn),
                target,
                body,
                ..
            } if matches!(&tn.kind, ast::TypeKind::Named { name, .. } if name.text == "Index") => {
                // Freeze `let K = <expr>` for `impl Index for S`.
                if let ast::TypeKind::Named { name: tname, .. } = &target.kind {
                    if let Some(Sym::Struct(sid)) = self.get_sym(&tname.text) {
                        if let Some(dom_id) = self.structs[sid].index_domain {
                            for s in &body.stmts {
                                if let ast::StmtKind::Let {
                                    pat: ast::Pattern { kind: ast::PatternKind::Name(n), .. },
                                    init: Some(init),
                                    ..
                                } = &s.kind
                                {
                                    if n == "K" {
                                        let len = self.expect(init, &Ty::Int);
                                        let lo = self.int_lit(0, init.span);
                                        out.push(Stmt::DefDomain { id: dom_id, lo, len });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ast::StmtKind::Use {
                path,
                names,
                glob,
                alias,
            } if !self.at_top_level() => {
                // Support block-scoped `use std.segtree.{SegTree}` inside functions/tests.
                let mod_name = path.iter().map(|n| n.text.as_str()).collect::<Vec<_>>().join(".");
                let mod_slash = path.iter().map(|n| n.text.as_str()).collect::<Vec<_>>().join("/");
                let last_cp = path
                    .last()
                    .map(|n| format!("/{}.cp", n.text))
                    .unwrap_or_default();
                let target_mod = self
                    .mods[self.cur_mod]
                    .modules()
                    .iter()
                    .find(|m| {
                        m.path == mod_name
                            || m.path.ends_with(&format!(".{mod_name}"))
                            || m.path.ends_with(&format!("{mod_slash}.cp"))
                            || (!last_cp.is_empty() && m.path.ends_with(&last_cp))
                    })
                    .and_then(|m| self.path_to_mod.get(&m.path).copied());
                if let Some(tmod) = target_mod {
                    if !names.is_empty() {
                        for item in names {
                            if let Some(&sym) = self.mod_syms[tmod].get(&item.text) {
                                self.cx
                                    .local_syms
                                    .last_mut()
                                    .unwrap()
                                    .insert(item.text.clone(), sym);
                            }
                        }
                    } else if *glob {
                        let syms: Vec<(String, Sym)> = self.mod_syms[tmod]
                            .iter()
                            .map(|(k, &v)| (k.clone(), v))
                            .collect();
                        for (k, v) in syms {
                            self.cx.local_syms.last_mut().unwrap().insert(k, v);
                        }
                    } else if let Some(al) = alias {
                        self.cx
                            .local_syms
                            .last_mut()
                            .unwrap()
                            .insert(al.text.clone(), Sym::Mod(tmod));
                    } else if let Some(last) = path.last() {
                        self.cx
                            .local_syms
                            .last_mut()
                            .unwrap()
                            .insert(last.text.clone(), Sym::Mod(tmod));
                    }
                }
            }
            ast::StmtKind::Fn(decl) if !self.at_top_level() => {
                let memo = stmt.decorators.iter().any(|d| d.name.text == "memo");
                self.lower_local_fn(decl, memo, out);
            }
            // Top-level declarations were handled by `collect_all`.
            ast::StmtKind::Fn(decl) => {
                if let Some(table) = super::memo::into_table(stmt) {
                    self.memo_into_table(table, decl, out);
                }
            }
            ast::StmtKind::Struct(_)
            | ast::StmtKind::Enum(_)
            | ast::StmtKind::TypeAlias { .. }
            | ast::StmtKind::Trait { .. }
            | ast::StmtKind::Impl { .. }
            | ast::StmtKind::Use { .. }
            | ast::StmtKind::Law { .. }
            | ast::StmtKind::Lend { .. } => {}
            ast::StmtKind::Domain { name, def } => self.domain(name, def, out),
            ast::StmtKind::Expr(e) => self.tail(e, Tail::Discard, out),
            ast::StmtKind::Assign { target, op, value } => self.assign(target, *op, value, out),
            ast::StmtKind::While { cond, body } => self.while_stmt(cond, body, out),
            ast::StmtKind::For {
                label,
                pat,
                iter,
                body,
            } => {
                // Support compile-time unrolling of heterogeneous literal lists like
                // `for (k, act) in [(0, AddSum), ..., (0, AssignSum)]:` where items have different struct types.
                if let ast::ExprKind::List(items) = &iter.kind {
                    if items.len() >= 2 {
                        let first_ty = self.probe_ty_silent(&items[0]);
                        let is_hetero = first_ty.as_ref().is_some_and(|ft| {
                            items[1..].iter().any(|it| {
                                self.probe_ty_silent(it).is_some_and(|ot| ot != *ft)
                            })
                        });
                        if is_hetero {
                            for item in items {
                                let item_val = self.expr(item, None);
                                self.cx.scopes.push(HashMap::new());
                                self.cx.local_syms.push(HashMap::new());
                                self.bind(pat, item_val, out);
                                out.extend(self.block(body, Tail::Discard).stmts);
                                self.cx.local_syms.pop();
                                self.cx.scopes.pop();
                            }
                            return;
                        }
                    }
                }
                let (iter, elem) = self.iter(iter);
                let id = self.open_loop(label.as_ref().map(|l| l.text.clone()));
                self.cx.scopes.push(HashMap::new());
                self.cx.local_syms.push(HashMap::new());
                let mut stmts = Vec::new();
                let elem_is_int = elem == Ty::Int;
                let slot = self.loop_var(pat, elem, &mut stmts);
                // If an `Int` loop variable is used as a runtime generic value arg (`for d in 0..=6: type F = Fixed<d>`),
                // mirror it into a global so monomorphized methods can read it at run time.
                if elem_is_int
                    && body
                        .stmts
                        .iter()
                        .any(|s| matches!(s.kind, ast::StmtKind::TypeAlias { .. }))
                {
                    if let ast::PatternKind::Name(vname) = &pat.kind {
                        if let Some(vslot) = self.lookup(vname) {
                            let gid = self.globals.len();
                            self.globals.push(Global {
                                name: format!("{vname}_rt_{gid}"),
                                ty: Ty::Int,
                                span: Span::default(),
                            });
                            let vref = self.local(vslot, pat.span);
                            stmts.push(Stmt::Assign(
                                Place {
                                    kind: PlaceKind::Global(gid),
                                    ty: Ty::Int,
                                    span: self.span(pat.span),
                                },
                                AssignOp::Set,
                                vref,
                            ));
                            self.cx
                                .local_syms
                                .last_mut()
                                .unwrap()
                                .insert(vname.clone(), Sym::Global(gid));
                        }
                    }
                }
                stmts.extend(self.block(body, Tail::Discard).stmts);
                self.cx.local_syms.pop();
                self.cx.scopes.pop();
                self.cx.loops.pop();
                out.push(Stmt::For(id, slot, iter, Block { stmts }));
            }
            ast::StmtKind::Return(value) => {
                if !self.cx.dos.is_empty() {
                    self.unsupported(stmt.span, "`return` inside a block that is used as a value");
                    return;
                }
                if let Some((acc, _)) = self.cx.yield_slot {
                    let res = self.local(acc, stmt.span);
                    out.push(Stmt::Return(Some(res)));
                    return;
                }
                match value {
                    None => {
                        match &self.cx.ret {
                            None => self.cx.ret = Some(Ty::Unit),
                            Some(Ty::Unit | Ty::Error) => {}
                            Some(ty) => {
                                let ty = ty.clone();
                                self.mismatch(stmt.span, &ty, &Ty::Unit);
                            }
                        }
                        out.push(Stmt::Return(None));
                    }
                    Some(e) => {
                        let hint = self.cx.ret.clone();
                        let mut value = self.expr(e, hint.as_ref());
                        match hint {
                            Some(ty) => value = self.coerce(value, &ty, e.span),
                            None => self.cx.ret = Some(value.ty.clone()),
                        }
                        out.push(Stmt::Return(Some(value)));
                    }
                }
            }
            ast::StmtKind::Break(label) | ast::StmtKind::Continue(label) => {
                let is_break = matches!(stmt.kind, ast::StmtKind::Break(_));
                let word = if is_break { "break" } else { "continue" };
                let found = match label {
                    None => self.cx.loops.len().checked_sub(1),
                    Some(label) => self
                        .cx
                        .loops
                        .iter()
                        .rposition(|(name, _)| name.as_deref() == Some(label.text.as_str())),
                };
                let Some(index) = found else {
                    let message = match label {
                        Some(label) => format!("no enclosing loop is labeled `{}`", label.text),
                        None => format!("`{word}` outside a loop"),
                    };
                    self.error(codes::UNRESOLVED_NAME, stmt.span, message);
                    return;
                };
                if self.cx.dos.last().is_some_and(|frame| index < frame.loops) {
                    self.unsupported(
                        stmt.span,
                        &format!("`{word}` out of a block that is used as a value"),
                    );
                    return;
                }
                let target = (index + 1 != self.cx.loops.len()).then(|| self.cx.loops[index].1);
                out.push(if is_break {
                    Stmt::Break(target)
                } else {
                    Stmt::Continue(target)
                });
            }
            ast::StmtKind::Yield { value, from } => {
                let Some((acc, elem_ty)) = self.cx.yield_slot.clone() else {
                    self.error(codes::UNSUPPORTED, stmt.span, "`yield` outside a generator");
                    return;
                };
                let acc_place = Place {
                    kind: PlaceKind::Local(acc),
                    ty: Ty::list(elem_ty.clone()),
                    span: self.span(stmt.span),
                };
                if *from {
                    let seq = self.expr(value, None);
                    let push_call = self.mk(
                        ExprKind::Mutate(Mutation::Extend, Box::new(acc_place), vec![seq]),
                        Ty::Unit,
                        stmt.span,
                    );
                    out.push(Stmt::Expr(push_call));
                } else {
                    let v = self.expect(value, &elem_ty);
                    let push_call = self.mk(
                        ExprKind::Mutate(Mutation::Push, Box::new(acc_place), vec![v]),
                        Ty::Unit,
                        stmt.span,
                    );
                    out.push(Stmt::Expr(push_call));
                }
            }
            ast::StmtKind::Assert { cond, message } => {
                let text = self.source(cond.span);
                let line = self.line(cond.span);
                let span = self.span(stmt.span);
                let cond = self.expect(cond, &Ty::Bool);
                let message = message.as_ref().map(|m| self.expect(m, &Ty::Str));
                out.push(Stmt::Assert {
                    cond,
                    message,
                    text,
                    line,
                    span,
                });
            }
            ast::StmtKind::Unchecked(block) => out.extend(self.block(block, Tail::Discard).stmts),
            ast::StmtKind::Error => {}
        }
    }

    pub(super) fn open_loop(&mut self, label: Option<String>) -> LoopId {
        let id = self.next_loop;
        self.next_loop += 1;
        self.cx.loops.push((label, id));
        id
    }

    /// The slot a `for` fills each round. A pattern other than a name is
    /// taken apart at the top of the body.
    pub(super) fn loop_var(&mut self, pat: &ast::Pattern, elem: Ty, body: &mut Vec<Stmt>) -> Slot {
        match &pat.kind {
            ast::PatternKind::Name(name) => {
                let slot = self.new_slot(name, elem);
                self.cx
                    .scopes
                    .last_mut()
                    .expect("a scope")
                    .insert(name.clone(), slot);
                slot
            }
            ast::PatternKind::Wildcard => self.temp(elem),
            _ => {
                let slot = self.temp(elem);
                let value = self.local(slot, pat.span);
                self.bind(pat, value, body);
                slot
            }
        }
    }

    fn domain(&mut self, name: &ast::Name, def: &ast::Expr, out: &mut Vec<Stmt>) {
        let id = if self.at_top_level() {
            match self.mod_syms[self.cur_mod].get(&name.text).copied() {
                Some(Sym::Domain(id)) => id,
                _ => return,
            }
        } else if let Some(Sym::Domain(id)) = self
            .cx
            .local_syms
            .iter()
            .rev()
            .find_map(|s| s.get(&name.text).copied())
        {
            // Reuse pre-allocated domain id (e.g. `domain C = 0..k` inside `scc` returning `domain C`).
            id
        } else {
            let id = self.domains.len();
            self.domains.push(format!("{}_{id}", name.text));
            self.cx
                .local_syms
                .last_mut()
                .unwrap()
                .insert(name.text.clone(), Sym::Domain(id));
            id
        };
        self.init_domain_id(id, def, out);
    }

    pub(super) fn init_domain_id(&mut self, id: DomainId, def: &ast::Expr, out: &mut Vec<Stmt>) {
        let ast::ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
            step: None,
        } = &def.kind
        else {
            self.unsupported(def.span, "a domain that is not a range such as `1..=n`");
            return;
        };
        let lo_e = self.expr(lo, None);
        let lo = if lo_e.ty == Ty::Char {
            self.mk(ExprKind::Cast(Cast::CharToInt, Box::new(lo_e)), Ty::Int, lo.span)
        } else {
            self.coerce(lo_e, &Ty::Int, lo.span)
        };
        let hi_e = self.expr(hi, None);
        let hi = if hi_e.ty == Ty::Char {
            self.mk(ExprKind::Cast(Cast::CharToInt, Box::new(hi_e)), Ty::Int, hi.span)
        } else {
            self.coerce(hi_e, &Ty::Int, hi.span)
        };
        let slot = self.temp(Ty::Int);
        out.push(Stmt::Let(slot, Some(lo)));
        let lo = self.local(slot, def.span);
        let len = self.mk(
            ExprKind::Binary(BinOp::Sub, Box::new(hi), Box::new(lo.clone())),
            Ty::Int,
            def.span,
        );
        let len = if *inclusive {
            self.int_add(len, 1)
        } else {
            len
        };
        out.push(Stmt::DefDomain { id, lo, len });
    }

    fn peek_list_elem_type(&mut self, var_name: &str, rest: &[ast::Stmt]) -> Option<Ty> {
        fn find_push<'a>(var_name: &str, stmts: &'a [ast::Stmt]) -> Option<&'a ast::Expr> {
            for s in stmts {
                match &s.kind {
                    ast::StmtKind::Expr(e) => {
                        if let ast::ExprKind::Call { callee, args } = &e.kind {
                            if let ast::ExprKind::Field { base, name } = &callee.kind {
                                if name.text == "push"
                                    && matches!(&base.kind, ast::ExprKind::Name(n) if n == var_name)
                                    && args.len() == 1
                                {
                                    return Some(&args[0].value);
                                }
                            }
                        }
                        if let ast::ExprKind::If {
                            branches,
                            otherwise,
                        } = &e.kind
                        {
                            for (_, b) in branches {
                                if let Some(e) = find_push(var_name, &b.stmts) {
                                    return Some(e);
                                }
                            }
                            if let Some(b) = otherwise {
                                if let Some(e) = find_push(var_name, &b.stmts) {
                                    return Some(e);
                                }
                            }
                        }
                    }
                    ast::StmtKind::While { body, .. } | ast::StmtKind::For { body, .. } => {
                        if let Some(e) = find_push(var_name, &body.stmts) {
                            return Some(e);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        let arg = find_push(var_name, rest)?;
        self.probe_ty_silent(arg)
    }

    /// Infers the full type of `var x = List.filled(..., none)` (or `var x = none`)
    /// from the enclosing function's return type or subsequent assignments in `rest`.
    fn peek_filled_none_type(
        &mut self,
        var_name: &str,
        depth: usize,
        rest: &[ast::Stmt],
    ) -> Option<Ty> {
        fn has_depth_opt(ty: &Ty, depth: usize) -> bool {
            let mut cur = ty;
            for _ in 0..depth {
                match cur {
                    Ty::List(inner) => cur = inner,
                    _ => return false,
                }
            }
            matches!(cur, Ty::Opt(_))
        }
        fn returns_var(var_name: &str, stmts: &[ast::Stmt]) -> bool {
            if let Some(last) = stmts.last() {
                if let ast::StmtKind::Expr(e) = &last.kind {
                    if matches!(&e.kind, ast::ExprKind::Name(n) if n == var_name) {
                        return true;
                    }
                }
            }
            for s in stmts {
                match &s.kind {
                    ast::StmtKind::Return(Some(e))
                        if matches!(&e.kind, ast::ExprKind::Name(n) if n == var_name) =>
                    {
                        return true;
                    }
                    ast::StmtKind::Expr(e) => {
                        if let ast::ExprKind::If {
                            branches,
                            otherwise,
                        } = &e.kind
                        {
                            if branches.iter().any(|(_, b)| returns_var(var_name, &b.stmts))
                                || otherwise
                                    .as_ref()
                                    .is_some_and(|b| returns_var(var_name, &b.stmts))
                            {
                                return true;
                            }
                        }
                    }
                    ast::StmtKind::While { body, .. } | ast::StmtKind::For { body, .. }
                        if returns_var(var_name, &body.stmts) =>
                    {
                        return true;
                    }
                    _ => {}
                }
            }
            false
        }
        if let Some(ret_ty) = self.cx.ret.clone() {
            if has_depth_opt(&ret_ty, depth) && returns_var(var_name, rest) {
                return Some(ret_ty);
            }
        }
        fn index_depth_of(e: &ast::Expr, var_name: &str) -> Option<usize> {
            match &e.kind {
                ast::ExprKind::Name(n) if n == var_name => Some(0),
                ast::ExprKind::Paren(inner) => index_depth_of(inner, var_name),
                ast::ExprKind::Index { base, indices } if indices.len() == 1 => {
                    index_depth_of(base, var_name).map(|d| d + 1)
                }
                _ => None,
            }
        }
        fn collect_rhs<'a>(
            var_name: &str,
            depth: usize,
            stmts: &'a [ast::Stmt],
            out: &mut Vec<&'a ast::Expr>,
            decls: &mut Vec<(&'a str, &'a ast::Type)>,
        ) {
            for s in stmts {
                match &s.kind {
                    ast::StmtKind::Let {
                        pat:
                            ast::Pattern {
                                kind: ast::PatternKind::Name(n),
                                ..
                            },
                        ty: Some(t),
                        ..
                    } => decls.push((n.as_str(), t)),
                    ast::StmtKind::Assign { target, value, .. } => {
                        if index_depth_of(target, var_name) == Some(depth)
                            && !matches!(value.kind, ast::ExprKind::None)
                        {
                            out.push(value);
                        }
                    }
                    ast::StmtKind::Expr(e) => {
                        if depth == 1 {
                            if let ast::ExprKind::Call { callee, args } = &e.kind {
                                if let ast::ExprKind::Field { base, name } = &callee.kind {
                                    if name.text == "push"
                                        && matches!(&base.kind, ast::ExprKind::Name(n) if n == var_name)
                                        && args.len() == 1
                                        && !matches!(args[0].value.kind, ast::ExprKind::None)
                                    {
                                        out.push(&args[0].value);
                                    }
                                }
                            }
                        }
                        if let ast::ExprKind::If {
                            branches,
                            otherwise,
                        } = &e.kind
                        {
                            for (_, b) in branches {
                                collect_rhs(var_name, depth, &b.stmts, out, decls);
                            }
                            if let Some(b) = otherwise {
                                collect_rhs(var_name, depth, &b.stmts, out, decls);
                            }
                        }
                    }
                    ast::StmtKind::While { body, .. } | ast::StmtKind::For { body, .. } => {
                        collect_rhs(var_name, depth, &body.stmts, out, decls);
                    }
                    _ => {}
                }
            }
        }
        let mut candidates = Vec::new();
        let mut decls = Vec::new();
        collect_rhs(var_name, depth, rest, &mut candidates, &mut decls);
        for rhs in candidates {
            let elem = if let ast::ExprKind::Name(n) = &rhs.kind {
                if let Some((_, ty_ast)) = decls.iter().find(|(dn, _)| *dn == n.as_str()) {
                    let t = self.resolve_ty(ty_ast);
                    (t != Ty::Error).then_some(t)
                } else {
                    self.probe_ty_silent(rhs)
                }
            } else {
                self.probe_ty_silent(rhs)
            };
            if let Some(t) = elem {
                if t == Ty::Error || matches!(&t, Ty::Opt(inner) if **inner == Ty::Error) {
                    continue;
                }
                let mut full = match t {
                    Ty::Opt(_) => t,
                    other => Ty::opt(other),
                };
                for _ in 0..depth {
                    full = Ty::list(full);
                }
                return Some(full);
            }
        }
        None
    }

    fn peek_fn_arg_types(&mut self, fn_name: &str, arity: usize, rest: &[ast::Stmt]) -> Option<Vec<Ty>> {
        fn find_call_in_expr<'a>(
            fn_name: &str,
            arity: usize,
            e: &'a ast::Expr,
        ) -> Option<&'a [ast::Arg]> {
            match &e.kind {
                ast::ExprKind::Call { callee, args } => {
                    if matches!(&callee.kind, ast::ExprKind::Name(n) if n == fn_name)
                        && args.len() == arity
                    {
                        return Some(args);
                    }
                    if let Some(found) = find_call_in_expr(fn_name, arity, callee) {
                        return Some(found);
                    }
                    for a in args {
                        if let Some(found) = find_call_in_expr(fn_name, arity, &a.value) {
                            return Some(found);
                        }
                    }
                    None
                }
                ast::ExprKind::Field { base, .. }
                | ast::ExprKind::Paren(base)
                | ast::ExprKind::Unary { expr: base, .. }
                | ast::ExprKind::Unwrap(base) => find_call_in_expr(fn_name, arity, base),
                ast::ExprKind::Binary { lhs, rhs, .. } => find_call_in_expr(fn_name, arity, lhs)
                    .or_else(|| find_call_in_expr(fn_name, arity, rhs)),
                ast::ExprKind::List(items) | ast::ExprKind::Tuple(items) => {
                    items.iter().find_map(|x| find_call_in_expr(fn_name, arity, x))
                }
                ast::ExprKind::If { branches, otherwise } => {
                    for (cond, block) in branches {
                        if let ast::Cond::Expr(ce) = cond {
                            if let Some(found) = find_call_in_expr(fn_name, arity, ce) {
                                return Some(found);
                            }
                        }
                        if let Some(found) = find_call(fn_name, arity, &block.stmts) {
                            return Some(found);
                        }
                    }
                    otherwise
                        .as_ref()
                        .and_then(|b| find_call(fn_name, arity, &b.stmts))
                }
                ast::ExprKind::Block(b) => find_call(fn_name, arity, &b.stmts),
                _ => None,
            }
        }
        fn find_call<'a>(
            fn_name: &str,
            arity: usize,
            stmts: &'a [ast::Stmt],
        ) -> Option<&'a [ast::Arg]> {
            for s in stmts {
                match &s.kind {
                    ast::StmtKind::Let { init: Some(e), .. }
                    | ast::StmtKind::Expr(e)
                    | ast::StmtKind::Assign { value: e, .. }
                    | ast::StmtKind::Return(Some(e))
                    | ast::StmtKind::Assert { cond: e, .. } => {
                        if let Some(args) = find_call_in_expr(fn_name, arity, e) {
                            return Some(args);
                        }
                    }
                    ast::StmtKind::For { iter, body, .. } => {
                        if let Some(args) = find_call_in_expr(fn_name, arity, iter) {
                            return Some(args);
                        }
                        if let Some(args) = find_call(fn_name, arity, &body.stmts) {
                            return Some(args);
                        }
                    }
                    ast::StmtKind::While { cond, body } => {
                        if let ast::Cond::Expr(e) = cond {
                            if let Some(args) = find_call_in_expr(fn_name, arity, e) {
                                return Some(args);
                            }
                        }
                        if let Some(args) = find_call(fn_name, arity, &body.stmts) {
                            return Some(args);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        if let Some(args) = find_call(fn_name, arity, rest) {
            let mut tys = Vec::with_capacity(arity);
            for a in args {
                let ty = self.probe_ty_silent(&a.value)?;
                tys.push(ty);
            }
            return Some(tys);
        }
        // Also check when `fn_name` is passed as a callback argument to a typed function `outer(..., fn_name, ...)`.
        fn find_passed_in_expr<'a>(
            fn_name: &str,
            e: &'a ast::Expr,
        ) -> Option<(&'a str, usize)> {
            match &e.kind {
                ast::ExprKind::Call { callee, args } => {
                    if let ast::ExprKind::Name(cname) = &callee.kind {
                        for (idx, a) in args.iter().enumerate() {
                            if matches!(&a.value.kind, ast::ExprKind::Name(n) if n == fn_name) {
                                return Some((cname.as_str(), idx));
                            }
                            if let ast::ExprKind::Lambda { params, body } = &a.value.kind {
                                if find_call_in_expr(fn_name, params.len(), body).is_some() {
                                    return Some((cname.as_str(), idx));
                                }
                            }
                        }
                    }
                    find_passed_in_expr(fn_name, callee).or_else(|| {
                        args.iter()
                            .find_map(|a| find_passed_in_expr(fn_name, &a.value))
                    })
                }
                ast::ExprKind::Binary { lhs, rhs, .. } => {
                    find_passed_in_expr(fn_name, lhs).or_else(|| find_passed_in_expr(fn_name, rhs))
                }
                ast::ExprKind::Compare { first, rest } => find_passed_in_expr(fn_name, first)
                    .or_else(|| rest.iter().find_map(|(_, r)| find_passed_in_expr(fn_name, r))),
                ast::ExprKind::Unary { expr, .. }
                | ast::ExprKind::Paren(expr)
                | ast::ExprKind::Try(expr) => find_passed_in_expr(fn_name, expr),
                ast::ExprKind::Tuple(items) | ast::ExprKind::List(items) => {
                    items.iter().find_map(|x| find_passed_in_expr(fn_name, x))
                }
                _ => None,
            }
        }
        fn find_passed<'a>(fn_name: &str, stmts: &'a [ast::Stmt]) -> Option<(&'a str, usize)> {
            for s in stmts {
                match &s.kind {
                    ast::StmtKind::Let { init: Some(e), .. }
                    | ast::StmtKind::Expr(e)
                    | ast::StmtKind::Assign { value: e, .. }
                    | ast::StmtKind::Return(Some(e))
                    | ast::StmtKind::Assert { cond: e, .. } => {
                        if let Some(found) = find_passed_in_expr(fn_name, e) {
                            return Some(found);
                        }
                    }
                    ast::StmtKind::For { iter, body, .. } => {
                        if let Some(found) = find_passed_in_expr(fn_name, iter)
                            .or_else(|| find_passed(fn_name, &body.stmts))
                        {
                            return Some(found);
                        }
                    }
                    ast::StmtKind::While { cond, body } => {
                        if let ast::Cond::Expr(e) = cond {
                            if let Some(found) = find_passed_in_expr(fn_name, e) {
                                return Some(found);
                            }
                        }
                        if let Some(found) = find_passed(fn_name, &body.stmts) {
                            return Some(found);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        let (outer_name, arg_idx) = find_passed(fn_name, rest)?;
        if let Some(Sym::Fn(fid)) = self.get_sym(outer_name) {
            if let Some(param) = self.sigs[fid].params.get(arg_idx) {
                if let Ty::Fn(ps, _) = &param.ty {
                    if ps.len() == arity {
                        return Some(ps.clone());
                    }
                }
            }
        }
        None
    }

    fn peek_map_kv_types(&mut self, var_name: &str, rest: &[ast::Stmt]) -> (Ty, Ty) {
        for s in rest {
            if let ast::StmtKind::Assign {
                target:
                    ast::Expr {
                        kind: ast::ExprKind::Index { base, indices },
                        ..
                    },
                value,
                ..
            } = &s.kind
            {
                if matches!(&base.kind, ast::ExprKind::Name(n) if n == var_name)
                    && indices.len() == 1
                {
                    let diag_len = self.diags.len();
                    let kt = self.expr(&indices[0], None).ty;
                    let vt = self.expr(value, None).ty;
                    self.diags.truncate(diag_len);
                    let kt = if kt == Ty::Error { Ty::Int } else { kt };
                    let vt = if vt == Ty::Error { Ty::Int } else { vt };
                    return (kt, vt);
                }
            }
        }
        (Ty::Int, Ty::Int)
    }

    fn let_value_with_rest(
        &mut self,
        ty: Option<&ast::Type>,
        init: Option<&ast::Expr>,
        pat: &ast::Pattern,
        rest: &[ast::Stmt],
        span: Span,
    ) -> Option<Expr> {
        // Handle untyped `var (path, at) = ([], v)` followed by `path.push(elem)`.
        if ty.is_none() {
            if let (
                ast::PatternKind::Tuple(pats),
                Some(ast::Expr {
                    kind: ast::ExprKind::Tuple(exprs),
                    span: tspan,
                }),
            ) = (&pat.kind, init)
            {
                if pats.len() == exprs.len()
                    && exprs
                        .iter()
                        .any(|e| matches!(&e.kind, ast::ExprKind::List(items) if items.is_empty()))
                {
                    // Lower non-empty elements first so sibling names (e.g. `at` in `var (path, at) = ([], v)`)
                    // are visible when peeking `path.push(at)` in `rest`.
                    let mut slots: Vec<Option<Expr>> = vec![None; exprs.len()];
                    let mut temp_bindings: Vec<(String, Ty)> = Vec::new();
                    for (i, (p, e)) in pats.iter().zip(exprs).enumerate() {
                        if !matches!(&e.kind, ast::ExprKind::List(items) if items.is_empty()) {
                            let v = self.expr(e, None);
                            if let ast::PatternKind::Name(n) = &p.kind {
                                temp_bindings.push((n.clone(), v.ty.clone()));
                            }
                            slots[i] = Some(v);
                        }
                    }
                    // Temporarily bind sibling names in a scratch scope while peeking empty-list element types.
                    let saved_cx = self.cx.clone();
                    self.cx.scopes.push(HashMap::new());
                    for (n, t) in &temp_bindings {
                        let s = self.cx.locals.len();
                        self.cx.locals.push(Local {
                            name: n.clone(),
                            ty: t.clone(),
                        });
                        self.cx.scopes.last_mut().unwrap().insert(n.clone(), s);
                    }
                    let mut inferred_elems: Vec<Option<Ty>> = vec![None; exprs.len()];
                    for (i, (p, e)) in pats.iter().zip(exprs).enumerate() {
                        if slots[i].is_none() {
                            if let ast::PatternKind::Name(vname) = &p.kind {
                                inferred_elems[i] = self.peek_list_elem_type(vname, rest);
                            }
                        }
                        let _ = e;
                    }
                    self.cx = saved_cx;
                    let mut lowered = Vec::with_capacity(exprs.len());
                    let mut tys = Vec::with_capacity(exprs.len());
                    for (i, e) in exprs.iter().enumerate() {
                        if let Some(v) = slots[i].take() {
                            tys.push(v.ty.clone());
                            lowered.push(v);
                        } else if let Some(elem_ty) = inferred_elems[i].take() {
                            let lty = Ty::list(elem_ty);
                            lowered.push(self.mk(ExprKind::Tuple(Vec::new()), lty.clone(), e.span));
                            tys.push(lty);
                        } else {
                            let v = self.expr(e, None);
                            tys.push(v.ty.clone());
                            lowered.push(v);
                        }
                    }
                    return Some(self.mk(ExprKind::Tuple(lowered), Ty::Tuple(tys), *tspan));
                }
            }
            if let (
                ast::PatternKind::Name(vname),
                Some(ast::Expr {
                    kind: ast::ExprKind::List(items),
                    ..
                }),
            ) = (&pat.kind, init)
            {
                if items.is_empty() {
                    if let Some(elem_ty) = self.peek_list_elem_type(vname, rest) {
                        return Some(self.mk(ExprKind::Tuple(Vec::new()), Ty::list(elem_ty), span));
                    }
                }
            }
            if let (ast::PatternKind::Name(vname), Some(init_expr)) = (&pat.kind, init) {
                if let Some(depth) = filled_none_depth(init_expr) {
                    if let Some(hint_ty) = self.peek_filled_none_type(vname, depth, rest) {
                        return Some(self.expr(init_expr, Some(&hint_ty)));
                    }
                }
            }
            // Handle untyped `let f = w => ...` followed by `f("abc")` by peeking call argument types in `rest`.
            if let (
                ast::PatternKind::Name(vname),
                Some(ast::Expr {
                    kind: ast::ExprKind::Lambda { params, body },
                    span: lspan,
                }),
            ) = (&pat.kind, init)
            {
                if let Some(param_tys) = self.peek_fn_arg_types(vname, params.len(), rest) {
                    let hint = Ty::Fn(param_tys, Box::new(Ty::Error));
                    return Some(self.lower_lambda(params, body, Some(&hint), *lspan));
                }
            }
        }
        let val = self.let_value(ty, init, span)?;
        // Handle `let v = stack.pop()` where `stack: List<T>` (non-Opt elem):
        // if `v` is NOT used as an optional in `rest`, unwrap `Pop` -> `PopVal` so stdlib works,
        // while `collections.cp` (`stack.pop() ?? -1`, `print(stack.pop())`) keeps `Pop` -> `?T`!
        if ty.is_none() {
            if let (ExprKind::Mutate(Mutation::Pop, place, args), Ty::Opt(inner)) =
                (&val.kind, &val.ty)
            {
                let should_unwrap = match &pat.kind {
                    ast::PatternKind::Name(vname) => {
                        !matches!(**inner, Ty::Opt(_)) && !uses_as_optional(vname, rest)
                    }
                    ast::PatternKind::Tuple(_) => true,
                    _ => false,
                };
                if should_unwrap {
                    return Some(Expr {
                        kind: ExprKind::Mutate(Mutation::PopVal, place.clone(), args.clone()),
                        ty: (**inner).clone(),
                        span: val.span,
                    });
                }
            }
        }
        Some(val)
    }

    /// The value a `let` binds: its initializer, read or allocated the way
    /// the declared type says.
    fn let_value(
        &mut self,
        ty: Option<&ast::Type>,
        init: Option<&ast::Expr>,
        span: Span,
    ) -> Option<Expr> {
        let Some(ty) = ty else {
            let Some(init) = init else {
                self.error(
                    codes::UNKNOWN_TYPE,
                    span,
                    "a variable needs a type or a value",
                );
                return None;
            };
            return Some(self.expr(init, None));
        };
        let spec = self.resolve(ty, true);
        let want = spec.ty();
        let Some(init) = init else {
            return Some(self.zero(&spec, span));
        };
        if is_read_call(init) {
            return Some(self.read(spec, init.span));
        }
        // `let b: [0..n]Int = read_output()` in a `@checker`: the same
        // layout, read from what the solution printed.
        if is_read_output_call(init) && self.aside == Some(Aside::Checker) {
            return Some(self.read_printed(spec, init.span));
        }
        if let Spec::Array { dims, elem } = &spec {
            let elem_ty = elem.ty();
            if is_literal(init) {
                let value = self.expect(init, &elem_ty);
                return Some(self.fill_with_index_domains(dims.clone(), Some(Box::new(value)), want, init.span));
            }
            if let Some(table) = self.comprehension_table(init, dims, &elem_ty) {
                return Some(table);
            }
            // `var dp: [Bool]Z = [1, 0]`: a list of as many values as the
            // array has cells, when that count is written as a number.
            if let (ast::ExprKind::List(items), [DimSpec::Int(len)]) = (&init.kind, dims.as_slice()) {
                if let (ExprKind::Int(cells), false) = (&len.kind, items.is_empty()) {
                    if *cells != items.len() as i64 {
                        let message = format!(
                            "this array has {}, but the list gives {}",
                            plural(*cells, "cell"),
                            plural(items.len() as i64, "value"),
                        );
                        self.error(codes::TYPE_MISMATCH, init.span, message);
                        return Some(self.err_expr(init.span));
                    }
                    let dims = dims.clone();
                    let at = init.span;
                    return Some(self.do_expr(Some(&want.clone()), at, |lw, out| {
                        let values: Vec<Expr> =
                            items.iter().map(|item| lw.expect(item, &elem_ty)).collect();
                        let mut values = values.into_iter();
                        let first = values.next().map(Box::new);
                        let cells = lw.temp(want.clone());
                        let filled = lw.mk(ExprKind::Fill(dims, first), want.clone(), at);
                        out.push(Stmt::Let(cells, Some(filled)));
                        for (i, value) in values.enumerate() {
                            let index = lw.int_lit(i as i64 + 1, at);
                            let whole = Place { kind: PlaceKind::Local(cells), ty: want.clone(), span: at };
                            let cell = Place {
                                kind: PlaceKind::Index(Box::new(whole), vec![index]),
                                ty: elem_ty.clone(),
                                span: at,
                            };
                            out.push(Stmt::Assign(cell, AssignOp::Set, value));
                        }
                        out.push(Stmt::Give(lw.local(cells, at)));
                    }));
                }
            }
            // `var bags: [R, C]SortedBag<T> = SortedBag()`: a struct built
            // by name is one cell's value, and takes its type from the cell.
            let builds_a_cell = matches!(&init.kind, ast::ExprKind::Call { callee, .. }
                if matches!(&callee.kind, ast::ExprKind::Name(name)
                    if self.lookup(name).is_none()
                        && matches!(self.get_sym(name), Some(Sym::Struct(_) | Sym::GenStruct(_)))));
            if builds_a_cell && matches!(elem_ty, Ty::Struct(_)) {
                let value = self.expect(init, &elem_ty);
                return Some(self.fill_with_index_domains(dims.clone(), Some(Box::new(value)), want, init.span));
            }
            let value = self.expr(init, Some(&want));
            if value.ty == want || value.ty == Ty::Error {
                return Some(value);
            }
            return Some(match self.try_coerce(value, &elem_ty) {
                Ok(value) => self.fill_with_index_domains(dims.clone(), Some(Box::new(value)), want, init.span),
                Err(value) => {
                    self.mismatch(init.span, &want, &value.ty);
                    self.err_expr(init.span)
                }
            });
        }
        if let Ty::Struct(sid) = &want {
            if matches!(&init.kind, ast::ExprKind::List(items) if items.is_empty()) {
                if let Some(&new_fn) = self.structs[*sid].methods.get("new") {
                    if self.sigs[new_fn].params.is_empty() {
                        return Some(self.method_call(new_fn, None, &[], init.span));
                    }
                }
            }
        }
        Some(self.expect(init, &want))
    }

    /// Wraps `ExprKind::Fill` with dynamic `DefDomain` initialization when any dimension
    /// is an `impl Index` struct domain (e.g. `[Tri<D>]Int` where `K = D.K * (D.K + 1) / 2`).
    fn fill_with_index_domains(
        &mut self,
        dims: Vec<DimSpec>,
        fill: Option<Box<Expr>>,
        want: Ty,
        span: Span,
    ) -> Expr {
        let mut init_pairs: Vec<(DomainId, StructId, StructConst<'a>)> = Vec::new();
        for dim in &dims {
            if let DimSpec::Pos(dom_id) = dim {
                if let Some((sid, sc)) = self.structs.iter().enumerate().find_map(|(sid, s)| {
                    if s.index_domain == Some(*dom_id) {
                        s.consts.get("K").cloned().map(|sc| (sid, sc))
                    } else {
                        None
                    }
                }) {
                    init_pairs.push((*dom_id, sid, sc));
                }
            }
        }
        let fill_expr = self.mk(ExprKind::Fill(dims, fill), want.clone(), span);
        if init_pairs.is_empty() {
            return fill_expr;
        }
        self.do_expr(Some(&want), span, |lw, out| {
            for (dom_id, sid, sc) in init_pairs {
                let k_val = lw.eval_struct_const(sid, sc, span);
                let lo = lw.int_lit(0, span);
                out.push(Stmt::DefDomain {
                    id: dom_id,
                    lo,
                    len: k_val,
                });
            }
            out.push(Stmt::Give(fill_expr));
        })
    }

    /// The value a variable declared without one starts with.
    fn zero(&mut self, spec: &Spec, span: Span) -> Expr {
        let ty = spec.ty();
        let kind = match spec {
            Spec::Array { dims, .. } => ExprKind::Fill(dims.clone(), None),
            _ => match &ty {
                Ty::Int => ExprKind::Int(0),
                Ty::Float => ExprKind::Float(0.0),
                Ty::Bool => ExprKind::Bool(false),
                Ty::Str => ExprKind::Str(Vec::new()),
                Ty::Opt(_) => ExprKind::None,
                Ty::List(_) | Ty::Map(..) | Ty::HashMap(..) | Ty::Set(_) | Ty::HashSet(_) => {
                    ExprKind::Tuple(Vec::new())
                }
                Ty::Error => ExprKind::Error,
                _ => {
                    let message = format!(
                        "a `{}` has no starting value: give this variable one",
                        self.show(&ty)
                    );
                    self.error(codes::UNKNOWN_TYPE, span, message);
                    ExprKind::Error
                }
            },
        };
        self.mk(kind, ty, span)
    }

    /// Binds the names in `pat` to `value`.
    pub(super) fn bind(&mut self, pat: &ast::Pattern, value: Expr, out: &mut Vec<Stmt>) {
        self.bind_mut(pat, value, false, out);
    }

    pub(super) fn bind_mut(
        &mut self,
        pat: &ast::Pattern,
        value: Expr,
        mutable: bool,
        out: &mut Vec<Stmt>,
    ) {
        match &pat.kind {
            ast::PatternKind::Name(name) => self.declare(name, value, mutable, out),
            ast::PatternKind::Variant { path, args } if path.len() == 1 && args.is_empty() => {
                // Uppercase variable binding like `let P = p` or `let V = g.V` (parsed as Variant).
                let vname = &path[0].text;
                if value.ty == Ty::DomainVal {
                    if let ExprKind::DomainVal(dom_id) = value.kind {
                        self.cx
                            .local_syms
                            .last_mut()
                            .unwrap()
                            .insert(vname.clone(), Sym::Domain(dom_id));
                        return;
                    }
                    let dom_id = self.domains.len();
                    self.domains.push(format!("{vname}_{dom_id}"));
                    out.push(Stmt::DefDomainFromVal {
                        id: dom_id,
                        val: value,
                    });
                    self.cx
                        .local_syms
                        .last_mut()
                        .unwrap()
                        .insert(vname.clone(), Sym::Domain(dom_id));
                    return;
                }
                self.declare(vname, value, mutable, out);
            }
            ast::PatternKind::Wildcard => out.push(Stmt::Expr(value)),
            ast::PatternKind::Tuple(pats) => {
                let Some(fields) = value.ty.fields() else {
                    if value.ty != Ty::Error {
                        let message = format!(
                            "a `{}` can't be taken apart into {} names",
                            self.show(&value.ty),
                            pats.len()
                        );
                        self.error(codes::TYPE_MISMATCH, pat.span, message);
                    }
                    return;
                };
                if fields.len() != pats.len() {
                    let message = format!(
                        "this pattern has {} names, but the value has {} fields",
                        pats.len(),
                        fields.len()
                    );
                    self.error(codes::TYPE_MISMATCH, pat.span, message);
                    return;
                }
                // Check if any field is `Ty::DomainVal` (`let (C, comp) = scc(...)`):
                // find the concrete `DomainId` carried by the sibling field(s).
                let returned_dom_id = fields.iter().find_map(find_pos_domain_in_ty);
                let base = self.hold(value, out);
                for (i, (p, ty)) in pats.iter().zip(fields).enumerate() {
                    let field = Expr {
                        kind: ExprKind::Field(Box::new(base.clone()), i),
                        ty: ty.clone(),
                        span: base.span,
                    };
                    if ty == Ty::DomainVal {
                        let dname_opt = match &p.kind {
                            ast::PatternKind::Name(n) => Some(n.clone()),
                            ast::PatternKind::Variant { path, args }
                                if args.is_empty() && path.len() == 1 =>
                            {
                                Some(path[0].text.clone())
                            }
                            _ => None,
                        };
                        if let Some(dname) = dname_opt {
                            let dom_id = returned_dom_id.unwrap_or_else(|| {
                                let d = self.domains.len();
                                self.domains.push(format!("{dname}_{d}"));
                                d
                            });
                            if self.at_top_level() {
                                self.mod_syms[self.cur_mod]
                                    .insert(dname.clone(), Sym::Domain(dom_id));
                            } else {
                                self.cx
                                    .local_syms
                                    .last_mut()
                                    .unwrap()
                                    .insert(dname.clone(), Sym::Domain(dom_id));
                            }
                            out.push(Stmt::DefDomainFromVal {
                                id: dom_id,
                                val: field,
                            });
                            continue;
                        }
                    }
                    self.bind_mut(p, field, mutable, out);
                }
            }
            ast::PatternKind::Error => {}
            _ => self.unsupported(pat.span, "this pattern outside a `match`"),
        }
    }

    /// `value` as something cheap to mention twice: itself if it is a
    /// variable, else a temporary that holds it.
    pub(super) fn hold(&mut self, value: Expr, out: &mut Vec<Stmt>) -> Expr {
        if matches!(value.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
            return value;
        }
        let span = value.span;
        let slot = self.temp(value.ty.clone());
        let ty = value.ty.clone();
        out.push(Stmt::Let(slot, Some(value)));
        Expr {
            kind: ExprKind::Local(slot),
            ty,
            span,
        }
    }

    fn assign(
        &mut self,
        target: &ast::Expr,
        op: ast::AssignOp,
        value: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) {
        if let (ast::ExprKind::Tuple(targets), ast::AssignOp::Set) = (&target.kind, op) {
            // `(a[d], b[d]) = read()`: the places say what is read.
            let wanted: Option<Vec<Ty>> =
                targets.iter().map(|t| self.try_place(t).map(|place| place.ty)).collect();
            let wanted = wanted.map(Ty::Tuple);
            // `(a, b) = (b, a)`: the right side is whole before any store.
            let value = self.expr(value, wanted.as_ref());
            let Some(fields) = value.ty.fields().filter(|f| f.len() == targets.len()) else {
                if value.ty != Ty::Error {
                    let message = format!(
                        "{} variables can't take a `{}`",
                        targets.len(),
                        self.show(&value.ty)
                    );
                    self.error(codes::TYPE_MISMATCH, target.span, message);
                }
                return;
            };
            let slot = self.temp(value.ty.clone());
            let base = Expr {
                kind: ExprKind::Local(slot),
                ty: value.ty.clone(),
                span: value.span,
            };
            out.push(Stmt::Let(slot, Some(value)));
            for (i, (t, ty)) in targets.iter().zip(fields).enumerate() {
                let place = self.place(t);
                let field = Expr {
                    kind: ExprKind::Field(Box::new(base.clone()), i),
                    ty,
                    span: base.span,
                };
                let field = self.coerce(field, &place.ty, t.span);
                out.push(Stmt::Assign(place, AssignOp::Set, field));
            }
            return;
        }
        // Check for assignment through a `lends mut` projection (`m.at(i, j) = v`, `t.at(v) += 1`).
        if let ast::ExprKind::Call { callee, args } = &target.kind {
            if let ast::ExprKind::Field { base, name } = &callee.kind {
                if let Some(proj) = self.find_projection_on(base, &name.text) {
                    self.lower_projection_assign(proj, base, args, op, value, target.span, out);
                    return;
                }
            }
        }
        let place = self.place(target);
        // Every index of the target is worked out before the right side,
        // and the place is only reached for after both: `trie[grow()].ends
        // = true` and `trie[o].next ??= trie.push_id(Node())` are safe when
        // the list moves.
        let place = self.settle_indexes(place, out);
        let rhs = match op {
            ast::AssignOp::Set => self.expect(value, &place.ty),
            ast::AssignOp::Binary(BinOp::Coalesce) => match &place.ty {
                Ty::Opt(inner) => {
                    let inner = (**inner).clone();
                    // `(o ??= []).push(x)` goes on with what `o` holds.
                    self.filled.insert((self.cur_mod, target.span.start, target.span.end));
                    self.expect(value, &inner)
                }
                Ty::Error => self.expr(value, None),
                // A map entry: `m[k] ??= []` holds what the map holds.
                entry => {
                    let entry = entry.clone();
                    self.expect(value, &entry)
                }
            },
            _ => self.expr(value, Some(&place.ty)),
        };
        self.emit_assign_place(place, op, rhs, target.span, out);
    }

    /// Moves each index of `place` that does work (a call, a push) into a
    /// local declared in `out`, left to right, so the place can be named
    /// again without doing the work twice.
    pub(super) fn settle_indexes(&mut self, place: Place, out: &mut Vec<Stmt>) -> Place {
        let Place { kind, ty, span } = place;
        let kind = match kind {
            PlaceKind::Index(base, indexes) => {
                let base = self.settle_indexes(*base, out);
                let indexes = indexes
                    .into_iter()
                    .map(|index| {
                        if settled(&index) {
                            return index;
                        }
                        let slot = self.temp(index.ty.clone());
                        let held = Expr {
                            kind: ExprKind::Local(slot),
                            ty: index.ty.clone(),
                            span: index.span,
                        };
                        out.push(Stmt::Let(slot, Some(index)));
                        held
                    })
                    .collect();
                PlaceKind::Index(Box::new(base), indexes)
            }
            PlaceKind::Field(base, field) => {
                PlaceKind::Field(Box::new(self.settle_indexes(*base, out)), field)
            }
            other => other,
        };
        Place { kind, ty, span }
    }

    pub(super) fn emit_assign_place(
        &mut self,
        place: Place,
        op: ast::AssignOp,
        value: Expr,
        span: Span,
        out: &mut Vec<Stmt>,
    ) {
        // Track local mutations for closure capture analysis.
        if let PlaceKind::Local(s) = root_place(&place) {
            self.cx.mut_slots.insert(s);
        }
        let (op, value) = match op {
            ast::AssignOp::Set => (AssignOp::Set, self.coerce(value, &place.ty, span)),
            ast::AssignOp::Max | ast::AssignOp::Min => {
                if let Ty::Opt(inner) = place.ty.clone() {
                    if !matches!(*inner, Ty::Error) && !self.ordered(&inner) {
                        let message = format!(
                            "`max=` and `min=` need values with an order, not `{}`",
                            self.show(&place.ty)
                        );
                        self.error(codes::BAD_OPERANDS, span, message);
                        return;
                    }
                    let is_max = op == ast::AssignOp::Max;
                    let rhs = self.coerce(value, &place.ty, span);
                    let r_slot = self.temp(place.ty.clone());
                    out.push(Stmt::Let(r_slot, Some(rhs)));
                    let r_expr = self.local(r_slot, span);
                    let r_some = self.mk(ExprKind::IsSome(Box::new(r_expr.clone())), Ty::Bool, span);
                    let r_val = self.mk(ExprKind::Unwrap(Box::new(r_expr), false), (*inner).clone(), span);
                    let p_expr = place_to_expr(&place);
                    let p_some = self.mk(ExprKind::IsSome(Box::new(p_expr.clone())), Ty::Bool, span);
                    let p_none = self.mk(ExprKind::Unary(UnOp::Not, Box::new(p_some)), Ty::Bool, span);
                    let p_val = self.mk(ExprKind::Unwrap(Box::new(p_expr), false), (*inner).clone(), span);
                    let cmp_op = if is_max { CmpOp::Gt } else { CmpOp::Lt };
                    let better = self.mk(
                        ExprKind::Cmp(cmp_op, Box::new(r_val.clone()), Box::new(p_val)),
                        Ty::Bool,
                        span,
                    );
                    let should_set = self.mk(
                        ExprKind::Binary(BinOp::Or, Box::new(p_none), Box::new(better)),
                        Ty::Bool,
                        span,
                    );
                    let set_stmt = Stmt::Assign(
                        place,
                        AssignOp::Set,
                        self.mk(ExprKind::Some(Box::new(r_val)), Ty::Opt(inner), span),
                    );
                    out.push(Stmt::If(
                        r_some,
                        Block {
                            stmts: vec![Stmt::If(
                                should_set,
                                Block { stmts: vec![set_stmt] },
                                Block::default(),
                            )],
                        },
                        Block::default(),
                    ));
                    return;
                }
                // `best min= cand` with a `?T` on the right skips `none`.
                if matches!(&value.ty, Ty::Opt(inner) if **inner == place.ty) && self.ordered(&place.ty) {
                    let op = if op == ast::AssignOp::Max {
                        AssignOp::Max
                    } else {
                        AssignOp::Min
                    };
                    let slot = self.temp(value.ty.clone());
                    out.push(Stmt::Let(slot, Some(value)));
                    let held = self.local(slot, span);
                    let some = self.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
                    let inside = self.mk(ExprKind::Unwrap(Box::new(held), false), place.ty.clone(), span);
                    out.push(Stmt::If(
                        some,
                        Block {
                            stmts: vec![Stmt::Assign(place, op, inside)],
                        },
                        Block::default(),
                    ));
                    return;
                }
                if !matches!(place.ty, Ty::Error) && !self.ordered(&place.ty) {
                    let message = format!(
                        "`max=` and `min=` need values with an order, not `{}`",
                        self.show(&place.ty)
                    );
                    self.error(codes::BAD_OPERANDS, span, message);
                }
                let op = if op == ast::AssignOp::Max {
                    AssignOp::Max
                } else {
                    AssignOp::Min
                };
                (op, self.coerce(value, &place.ty, span))
            }
            ast::AssignOp::Binary(bin) => {
                // Handle `+=`, `-=`, `*=`, `/=` on structs implementing `Add`/`Sub`/`Mul`/`Div`.
                if let Ty::Struct(sid) = place.ty {
                    let mname = match bin {
                        BinOp::Add => Some("add"),
                        BinOp::Sub => Some("sub"),
                        BinOp::Mul => Some("mul"),
                        BinOp::Div => Some("div"),
                        BinOp::BitOr => Some("bitor"),
                        BinOp::BitAnd => Some("bitand"),
                        BinOp::BitXor => Some("bitxor"),
                        _ => None,
                    };
                    if let Some(fn_id) = mname.and_then(|m| self.struct_op_fn(sid, m)) {
                        let lhs_expr = place_to_expr(&place);
                        let rhs_val = self.coerce(value, &place.ty, span);
                        let ret = self.ret_of(fn_id, span);
                        let call = Expr {
                            kind: ExprKind::Call(fn_id, vec![Arg::Value(lhs_expr), Arg::Value(rhs_val)]),
                            ty: ret,
                            span: self.span(span),
                        };
                        let call = self.coerce(call, &place.ty, span);
                        out.push(Stmt::Assign(place, AssignOp::Set, call));
                        return;
                    }
                }
                use BinOp::*;
                // `x ??= init`: `init` runs only when `x` is `none`.
                if bin == Coalesce {
                    // `m[k] ??= init`: a key the map does not have gets
                    // `init`.
                    if let PlaceKind::Index(base, keys) = &place.kind {
                        if matches!(base.ty, Ty::Map(..) | Ty::HashMap(..)) {
                            let init = self.coerce(value, &place.ty, span);
                            let has = self.mk(
                                ExprKind::Builtin(
                                    Builtin::Contains,
                                    vec![place_to_expr(base), keys[0].clone()],
                                ),
                                Ty::Bool,
                                span,
                            );
                            let missing =
                                self.mk(ExprKind::Unary(UnOp::Not, Box::new(has)), Ty::Bool, span);
                            let slot = self.temp(place.ty.clone());
                            let fresh = self.local(slot, span);
                            out.push(Stmt::If(
                                missing,
                                Block {
                                    stmts: vec![
                                        Stmt::Let(slot, Some(init)),
                                        Stmt::Assign(place, AssignOp::Set, fresh),
                                    ],
                                },
                                Block::default(),
                            ));
                            return;
                        }
                    }
                    let Ty::Opt(inner) = place.ty.clone() else {
                        if place.ty != Ty::Error {
                            let message = format!(
                                "`??=` fills in an optional, but this is a `{}`",
                                self.show(&place.ty)
                            );
                            self.error(codes::BAD_OPERANDS, span, message);
                        }
                        return;
                    };
                    let init = self.coerce(value, &inner, span);
                    let held = place_to_expr(&place);
                    let some = self.mk(ExprKind::IsSome(Box::new(held)), Ty::Bool, span);
                    let missing = self.mk(ExprKind::Unary(UnOp::Not, Box::new(some)), Ty::Bool, span);
                    // The value is whole before the place is reached for.
                    let slot = self.temp((*inner).clone());
                    let fresh = self.local(slot, span);
                    let filled = self.mk(ExprKind::Some(Box::new(fresh)), place.ty.clone(), span);
                    out.push(Stmt::If(
                        missing,
                        Block {
                            stmts: vec![
                                Stmt::Let(slot, Some(init)),
                                Stmt::Assign(place, AssignOp::Set, filled),
                            ],
                        },
                        Block::default(),
                    ));
                    return;
                }
                // `none` does not pass through `+=` and `-=`: a `none` that
                // stays `none` hides a table that was never filled in.
                if let (Ty::Opt(inner), Add | Sub) = (&place.ty, bin) {
                    if matches!(**inner, Ty::Int | Ty::Float) {
                        let message = format!(
                            "`{}=` can't be applied to a `{}`",
                            bin.text(),
                            self.show(&place.ty)
                        );
                        let help = format!(
                            "write `x = x {} y` when `none` should stay `none`, or `x min= y + w` for a transition",
                            bin.text()
                        );
                        self.error(codes::BAD_OPERANDS, span, message);
                        self.help(help);
                        return;
                    }
                }
                let value = match (&place.ty, bin) {
                    (
                        Ty::Int,
                        Add | Sub | Mul | Div | Rem | Pow | BitAnd | BitOr | BitXor | Shl | Shr,
                    ) => self.coerce(value, &Ty::Int, span),
                    (Ty::I128 | Ty::U64, Shl | Shr) => self.coerce(value, &Ty::Int, span),
                    (wide @ (Ty::I128 | Ty::U64), Add | Sub | Mul | Div | Rem | Pow | BitAnd | BitOr | BitXor) => {
                        self.coerce(value, &wide.clone(), span)
                    }
                    (Ty::Float, Add | Sub | Mul | Div) => self.coerce(value, &Ty::Float, span),
                    (Ty::Bool, BitAnd | BitOr | BitXor) => self.coerce(value, &Ty::Bool, span),
                    (Ty::Str, Add) => self.as_str(value, span),
                    (Ty::Error, _) => value,
                    _ => {
                        let message = format!(
                            "`{}=` can't be applied to a `{}`",
                            bin.text(),
                            self.show(&place.ty)
                        );
                        self.error(codes::BAD_OPERANDS, span, message);
                        value
                    }
                };
                (AssignOp::Binary(bin), value)
            }
        };
        out.push(Stmt::Assign(place, op, value));
    }

    /// True for the `o` of `(o ??= init)`, which holds a value from there on.
    pub(super) fn is_filled(&self, e: &ast::Expr) -> bool {
        self.filled.contains(&(self.cur_mod, e.span.start, e.span.end))
    }

    pub(super) fn try_place(&mut self, e: &ast::Expr) -> Option<Place> {
        let diag_len = self.diags.len();
        let pending = self.cx.pending.len();
        let p = self.place(e);
        if self.diags.len() > diag_len || p.ty == Ty::Error {
            self.diags.truncate(diag_len);
            self.cx.pending.truncate(pending);
            None
        } else {
            Some(p)
        }
    }

    pub(super) fn find_projection_on(&mut self, base: &ast::Expr, method: &str) -> Option<ProjectionInfo<'a>> {
        let diag_len = self.diags.len();
        let b = self.expr(base, None);
        self.diags.truncate(diag_len);
        if let Ty::Struct(sid) = b.ty {
            self.structs[sid].projections.get(method).cloned()
        } else {
            None
        }
    }

    /// `e` as something to store into.
    pub(super) fn place(&mut self, e: &ast::Expr) -> Place {
        let span = self.span(e.span);
        let error = Place {
            kind: PlaceKind::Local(0),
            ty: Ty::Error,
            span,
        };
        match &e.kind {
            ast::ExprKind::Paren(inner) => {
                let place = self.place(inner);
                match &place.ty {
                    Ty::Opt(held) if self.is_filled(inner) => Place {
                        ty: (**held).clone(),
                        kind: PlaceKind::Some(Box::new(place)),
                        span,
                    },
                    _ => place,
                }
            }
            ast::ExprKind::Name(_) | ast::ExprKind::SelfValue => {
                // Inside an inlined `lends` projection, `self` or bare fields of `self` refer to `inline_self`.
                if let Some((ref recv_place, sid)) = self.cx.inline_self.clone() {
                    match &e.kind {
                        ast::ExprKind::SelfValue => return recv_place.clone(),
                        ast::ExprKind::Name(n) if self.lookup(n).is_none() => {
                            if let Some((fi, fty)) = self.field_of(&Ty::Struct(sid), n) {
                                return Place {
                                    kind: PlaceKind::Field(Box::new(recv_place.clone()), fi),
                                    ty: fty,
                                    span,
                                };
                            }
                        }
                        _ => {}
                    }
                }
                let value = self.expr(e, None);
                match value.kind {
                    ExprKind::Local(slot) => Place {
                        kind: PlaceKind::Local(slot),
                        ty: value.ty,
                        span,
                    },
                    ExprKind::Global(id) => Place {
                        kind: PlaceKind::Global(id),
                        ty: value.ty,
                        span,
                    },
                    // A bare field name inside a method.
                    ExprKind::Field(base, index) => match base.kind {
                        ExprKind::Local(slot) => Place {
                            kind: PlaceKind::Field(
                                Box::new(Place {
                                    kind: PlaceKind::Local(slot),
                                    ty: base.ty,
                                    span,
                                }),
                                index,
                            ),
                            ty: value.ty,
                            span,
                        },
                        _ => error,
                    },
                    ExprKind::Error => error,
                    _ => {
                        self.error(
                            codes::ASSIGN_TO_LET,
                            e.span,
                            "this name can't be assigned to",
                        );
                        error
                    }
                }
            }
            // `state!.k += 4`: the value inside, and the same failure as
            // reading it when there is none.
            ast::ExprKind::Unwrap(inner) => {
                let place = self.place(inner);
                match place.ty.clone() {
                    Ty::Opt(held) => {
                        let mut lead = Vec::new();
                        let place = self.settle_indexes(place, &mut lead);
                        let holds = place_to_expr(&place);
                        let some = self.mk(ExprKind::IsSome(Box::new(holds)), Ty::Bool, span);
                        let none = self.mk(ExprKind::Unary(UnOp::Not, Box::new(some)), Ty::Bool, span);
                        lead.push(Stmt::If(
                            none,
                            Block {
                                stmts: vec![Stmt::Trap(NO_VALUE.to_string(), span)],
                            },
                            Block::default(),
                        ));
                        self.cx.pending.extend(lead);
                        Place {
                            ty: *held,
                            kind: PlaceKind::Some(Box::new(place)),
                            span,
                        }
                    }
                    Ty::Error => error,
                    other => {
                        let message = format!(
                            "`!` takes the value out of an optional, but this is a `{}`",
                            self.show(&other)
                        );
                        self.error(codes::TYPE_MISMATCH, e.span, message);
                        error
                    }
                }
            }
            ast::ExprKind::Index { base, indices } => {
                let base = self.place(base);
                let (indices, ty) = self.index_args(&base.ty, indices, e.span);
                Place {
                    kind: PlaceKind::Index(Box::new(base), indices),
                    ty,
                    span,
                }
            }
            ast::ExprKind::Field { base, name } => {
                let base = self.place(base);
                match self.field_of(&base.ty, &name.text) {
                    Some((index, ty)) => Place {
                        kind: PlaceKind::Field(Box::new(base), index),
                        ty,
                        span,
                    },
                    None => {
                        if base.ty != Ty::Error {
                            let message = format!(
                                "`{}` has no field `{}` to assign to",
                                self.show(&base.ty),
                                name.text
                            );
                            self.error(codes::UNKNOWN_FIELD, name.span, message);
                        }
                        error
                    }
                }
            }
            _ => {
                self.error(
                    codes::ASSIGN_TO_LET,
                    e.span,
                    "only a variable, a field, or an element can be assigned to",
                );
                error
            }
        }
    }

    /// A stored field of a struct, record or tuple (`t.0`).
    pub(super) fn field_of(&self, ty: &Ty, name: &str) -> Option<(usize, Ty)> {
        match ty {
            Ty::Struct(id) => {
                let fields = &self.structs[*id].def.fields;
                let index = fields.iter().position(|(f, _)| f == name)?;
                Some((index, fields[index].1.clone()))
            }
            Ty::Record(fields) => {
                let index = fields.iter().position(|(f, _)| f == name)?;
                Some((index, fields[index].1.clone()))
            }
            Ty::Tuple(items) => {
                let index: usize = name.parse().ok()?;
                Some((index, items.get(index)?.clone()))
            }
            _ => None,
        }
    }

    fn if_chain(
        &mut self,
        branches: &[(ast::Cond, ast::Block)],
        otherwise: Option<&ast::Block>,
        tail: Tail,
        out: &mut Vec<Stmt>,
    ) {
        let Some(((cond, body), rest)) = branches.split_first() else {
            if let Some(block) = otherwise {
                out.extend(self.block(block, tail).stmts);
            }
            return;
        };
        match cond {
            ast::Cond::Expr(cond) => {
                let cond = self.expect(cond, &Ty::Bool);
                if matches!(cond.kind, ExprKind::Bool(true)) {
                    out.extend(self.block(body, tail).stmts);
                    return;
                }
                if matches!(cond.kind, ExprKind::Bool(false)) {
                    self.if_chain(rest, otherwise, tail, out);
                    return;
                }
                let then = self.block(body, tail);
                let mut els = Vec::new();
                self.if_chain(rest, otherwise, tail, &mut els);
                out.push(Stmt::If(cond, then, Block { stmts: els }));
            }
            ast::Cond::Let { pat, value, and } => {
                let Some((present, inner)) = self.optional(value, "if let", out) else {
                    return;
                };
                let mut els = Vec::new();
                self.if_chain(rest, otherwise, tail, &mut els);
                self.cx.scopes.push(HashMap::new());
                let mut then = Vec::new();
                self.bind(pat, inner, &mut then);
                if let Some(and_cond) = and {
                    let sub_branch = vec![((**and_cond).clone(), body.clone())];
                    let els_clone = els.clone();
                    let mut inner_out = Vec::new();
                    self.if_chain(&sub_branch, None, tail, &mut inner_out);
                    // Attach fallback `els` if the `and` condition fails.
                    if let Some(Stmt::If(_, _, ref mut inner_els)) = inner_out.last_mut() {
                        if inner_els.stmts.is_empty() {
                            inner_els.stmts = els_clone;
                        }
                    }
                    then.extend(inner_out);
                } else {
                    then.extend(self.block(body, tail).stmts);
                }
                self.cx.scopes.pop();
                out.push(Stmt::If(
                    present,
                    Block { stmts: then },
                    Block { stmts: els },
                ));
            }
            // `if const`: the test is settled here, and only the branch
            // it picks is lowered, so the other costs nothing at run time.
            ast::Cond::Const(cond) => {
                match self.eval_const_bool(cond) {
                    Some(true) => out.extend(self.block(body, tail).stmts),
                    Some(false) => self.if_chain(rest, otherwise, tail, out),
                    None => {
                        self.error(
                            codes::BAD_NUMBER,
                            cond.span,
                            "the test of an `if const` must be a compile-time constant",
                        );
                        self.help(
                            "it can compare literals, `const`s and `<const N>` parameters; \
                             use a plain `if` for a value read at run time",
                        );
                    }
                }
            }
        }
    }

    /// Evaluates the optional `value` into a temporary; gives the test
    /// that it is present and the value inside.
    fn optional(
        &mut self,
        value: &ast::Expr,
        what: &str,
        out: &mut Vec<Stmt>,
    ) -> Option<(Expr, Expr)> {
        let value_expr = self.expr(value, None);
        let Ty::Opt(inner) = value_expr.ty.clone() else {
            if value_expr.ty != Ty::Error {
                let message = format!(
                    "`{what}` takes the value out of an optional, but this is a `{}`",
                    self.show(&value_expr.ty)
                );
                self.error(codes::TYPE_MISMATCH, value.span, message);
            }
            return None;
        };
        let span = value_expr.span;
        let slot = self.temp(value_expr.ty.clone());
        let held = Expr {
            kind: ExprKind::Local(slot),
            ty: value_expr.ty.clone(),
            span,
        };
        out.push(Stmt::Let(slot, Some(value_expr)));
        let present = Expr {
            kind: ExprKind::IsSome(Box::new(held.clone())),
            ty: Ty::Bool,
            span,
        };
        let inside = Expr {
            kind: ExprKind::Unwrap(Box::new(held), false),
            ty: *inner,
            span,
        };
        Some((present, inside))
    }

    fn while_stmt(&mut self, cond: &ast::Cond, body: &ast::Block, out: &mut Vec<Stmt>) {
        match cond {
            ast::Cond::Expr(cond) => {
                let cond = self.expect(cond, &Ty::Bool);
                let id = self.open_loop(None);
                let body = self.block(body, Tail::Discard);
                self.cx.loops.pop();
                out.push(Stmt::While(id, cond, body));
            }
            ast::Cond::Let { pat, value, and } => {
                let id = self.open_loop(None);
                self.cx.scopes.push(HashMap::new());
                let mut stmts = Vec::new();
                if let Some((present, inner)) = self.optional(value, "while let", &mut stmts) {
                    let span = present.span;
                    let absent = Expr {
                        kind: ExprKind::Unary(UnOp::Not, Box::new(present)),
                        ty: Ty::Bool,
                        span,
                    };
                    stmts.push(Stmt::If(
                        absent,
                        Block {
                            stmts: vec![Stmt::Break(None)],
                        },
                        Block::default(),
                    ));
                    self.bind(pat, inner, &mut stmts);
                    if let Some(and_cond) = and {
                        match &**and_cond {
                            ast::Cond::Expr(c) => {
                                let c = self.expect(c, &Ty::Bool);
                                let not_c = Expr {
                                    kind: ExprKind::Unary(UnOp::Not, Box::new(c)),
                                    ty: Ty::Bool,
                                    span,
                                };
                                stmts.push(Stmt::If(
                                    not_c,
                                    Block {
                                        stmts: vec![Stmt::Break(None)],
                                    },
                                    Block::default(),
                                ));
                            }
                            ast::Cond::Let {
                                pat: p2,
                                value: v2,
                                ..
                            } => {
                                if let Some((pr2, in2)) = self.optional(v2, "while let", &mut stmts)
                                {
                                    let ab2 = Expr {
                                        kind: ExprKind::Unary(UnOp::Not, Box::new(pr2)),
                                        ty: Ty::Bool,
                                        span,
                                    };
                                    stmts.push(Stmt::If(
                                        ab2,
                                        Block {
                                            stmts: vec![Stmt::Break(None)],
                                        },
                                        Block::default(),
                                    ));
                                    self.bind(p2, in2, &mut stmts);
                                }
                            }
                            ast::Cond::Const(_) => {}
                        }
                    }
                    stmts.extend(self.block(body, Tail::Discard).stmts);
                }
                self.cx.scopes.pop();
                self.cx.loops.pop();
                let always = self.mk(ExprKind::Bool(true), Ty::Bool, value.span);
                out.push(Stmt::While(id, always, Block { stmts }));
            }
            ast::Cond::Const(cond) => self.unsupported(cond.span, "`while const`"),
        }
    }

    pub(super) fn match_stmt(
        &mut self,
        whole: &ast::Expr,
        scrutinee: &ast::Expr,
        arms: &[ast::Arm],
        tail: Tail,
        out: &mut Vec<Stmt>,
    ) {
        let value = self.expr(scrutinee, None);
        let value = self.hold(value, out);
        let span = self.span(whole.span);
        let trap = Stmt::Trap("no arm of this `match` fits the value".into(), span);
        if arms.iter().all(|arm| arm.guard.is_none()) {
            let mut lowered = Vec::new();
            for arm in arms {
                self.cx.scopes.push(HashMap::new());
                let mut body = Vec::new();
                let test = self.pattern(&arm.pat, &value, &mut body);
                body.extend(self.block(&arm.body, tail).stmts);
                self.cx.scopes.pop();
                lowered.push((test, body));
            }
            let mut chain = vec![trap];
            for (test, body) in lowered.into_iter().rev() {
                chain = match test {
                    None => body,
                    Some(test) => vec![Stmt::If(
                        test,
                        Block { stmts: body },
                        Block { stmts: chain },
                    )],
                };
            }
            out.extend(chain);
            return;
        }
        // With guards an arm can fit the pattern and still decline, so the
        // arms are tried in turn until one has run.
        let done = self.temp(Ty::Bool);
        let done_expr = Expr {
            kind: ExprKind::Local(done),
            ty: Ty::Bool,
            span,
        };
        let not_done = Expr {
            kind: ExprKind::Unary(UnOp::Not, Box::new(done_expr.clone())),
            ty: Ty::Bool,
            span,
        };
        let flag = |value: bool| Expr {
            kind: ExprKind::Bool(value),
            ty: Ty::Bool,
            span,
        };
        out.push(Stmt::Let(done, Some(flag(false))));
        for arm in arms {
            self.cx.scopes.push(HashMap::new());
            let mut body = Vec::new();
            let test = self.pattern(&arm.pat, &value, &mut body);
            let guard = arm.guard.as_ref().map(|g| self.expect(g, &Ty::Bool));
            let mut run = vec![Stmt::Assign(
                Place {
                    kind: PlaceKind::Local(done),
                    ty: Ty::Bool,
                    span,
                },
                AssignOp::Set,
                flag(true),
            )];
            run.extend(self.block(&arm.body, tail).stmts);
            self.cx.scopes.pop();
            match guard {
                Some(guard) => body.push(Stmt::If(guard, Block { stmts: run }, Block::default())),
                None => body.extend(run),
            }
            let test = match test {
                None => not_done.clone(),
                Some(test) => Expr {
                    kind: ExprKind::Binary(BinOp::And, Box::new(not_done.clone()), Box::new(test)),
                    ty: Ty::Bool,
                    span,
                },
            };
            out.push(Stmt::If(test, Block { stmts: body }, Block::default()));
        }
        if tail == Tail::Discard {
            out.push(Stmt::If(
                not_done,
                Block { stmts: vec![trap] },
                Block::default(),
            ));
        } else {
            out.push(trap);
        }
    }

    /// The test that `value` fits `pat` (`None` when it always does), with
    /// the statements that bind the pattern's names once it has.
    fn pattern(&mut self, pat: &ast::Pattern, value: &Expr, binds: &mut Vec<Stmt>) -> Option<Expr> {
        let span = self.span(pat.span);
        let and = |a: Option<Expr>, b: Option<Expr>| match (a, b) {
            (Some(a), Some(b)) => Some(Expr {
                kind: ExprKind::Binary(BinOp::And, Box::new(a), Box::new(b)),
                ty: Ty::Bool,
                span,
            }),
            (a, b) => a.or(b),
        };
        match &pat.kind {
            ast::PatternKind::Wildcard | ast::PatternKind::Error => None,
            ast::PatternKind::Name(name) => {
                self.declare(name, value.clone(), false, binds);
                None
            }
            ast::PatternKind::Literal(lit) => {
                if let (ast::ExprKind::None, Ty::Opt(_)) = (&lit.kind, &value.ty) {
                    let present = Expr {
                        kind: ExprKind::IsSome(Box::new(value.clone())),
                        ty: Ty::Bool,
                        span,
                    };
                    return Some(Expr {
                        kind: ExprKind::Unary(UnOp::Not, Box::new(present)),
                        ty: Ty::Bool,
                        span,
                    });
                }
                let lit = self.expect(lit, &value.ty);
                Some(Expr {
                    kind: ExprKind::Cmp(CmpOp::Eq, Box::new(value.clone()), Box::new(lit)),
                    ty: Ty::Bool,
                    span,
                })
            }
            ast::PatternKind::Variant { path, args } => {
                let name = path.last().expect("a variant pattern has a name");
                let Ty::Enum(id) = value.ty else {
                    if value.ty != Ty::Error {
                        let message = format!(
                            "`{}` is a variant, but the value matched is a `{}`",
                            name.text,
                            self.show(&value.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, pat.span, message);
                    }
                    return None;
                };
                let variants = &self.enums[id].variants;
                let Some(tag) = variants.iter().position(|(v, _)| *v == name.text) else {
                    let message =
                        format!("`{}` has no variant `{}`", self.enums[id].name, name.text);
                    self.error(codes::UNKNOWN_FIELD, name.span, message);
                    return None;
                };
                let fields = variants[tag].1.clone();
                if fields.len() != args.len() {
                    let message = format!(
                        "`{}` holds {} values, but this pattern names {}",
                        name.text,
                        fields.len(),
                        args.len()
                    );
                    self.error(codes::WRONG_ARG_COUNT, pat.span, message);
                    return None;
                }
                let mut test = Some(Expr {
                    kind: ExprKind::IsVariant(Box::new(value.clone()), tag),
                    ty: Ty::Bool,
                    span,
                });
                for (i, (arg, ty)) in args.iter().zip(fields).enumerate() {
                    let payload = Expr {
                        kind: ExprKind::Payload(Box::new(value.clone()), tag, i),
                        ty,
                        span,
                    };
                    test = and(test, self.pattern(arg, &payload, binds));
                }
                test
            }
            ast::PatternKind::Tuple(pats) => {
                let Some(fields) = value.ty.fields().filter(|f| f.len() == pats.len()) else {
                    if value.ty != Ty::Error {
                        let message = format!(
                            "a pattern of {} parts can't match a `{}`",
                            pats.len(),
                            self.show(&value.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, pat.span, message);
                    }
                    return None;
                };
                let mut test = None;
                for (i, (p, ty)) in pats.iter().zip(fields).enumerate() {
                    let field = Expr {
                        kind: ExprKind::Field(Box::new(value.clone()), i),
                        ty,
                        span,
                    };
                    test = and(test, self.pattern(p, &field, binds));
                }
                test
            }
            ast::PatternKind::Or(alts) => {
                let mut test: Option<Expr> = None;
                let mut always = false;
                for alt in alts {
                    let mut names = Vec::new();
                    let one = self.pattern(alt, value, &mut names);
                    if !names.is_empty() {
                        self.unsupported(alt.span, "names bound inside a `|` pattern");
                    }
                    match (one, test.take()) {
                        (None, _) => always = true,
                        (Some(one), None) => test = Some(one),
                        (Some(one), Some(prev)) => {
                            test = Some(Expr {
                                kind: ExprKind::Binary(BinOp::Or, Box::new(prev), Box::new(one)),
                                ty: Ty::Bool,
                                span,
                            })
                        }
                    }
                }
                if always {
                    None
                } else {
                    test
                }
            }
        }
    }
}

/// `read_output()`.
fn is_read_output_call(e: &ast::Expr) -> bool {
    matches!(&e.kind, ast::ExprKind::Call { callee, args }
        if args.is_empty() && matches!(&callee.kind, ast::ExprKind::Name(n) if n == "read_output"))
}

/// `read()` or `read(gen: ...)`.
pub(super) fn is_read_call(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Call { callee, args } => {
            matches!(&callee.kind, ast::ExprKind::Name(n) if n == "read")
                && args
                    .iter()
                    .all(|a| a.name.as_ref().is_some_and(|n| n.text == "gen"))
        }
        _ => false,
    }
}

fn find_pos_domain_in_ty(ty: &Ty) -> Option<DomainId> {
    match ty {
        Ty::Pos(d) | Ty::Bound(d) => Some(*d),
        Ty::Array { elem, .. } | Ty::List(elem) | Ty::Opt(elem) | Ty::Range(elem) => {
            find_pos_domain_in_ty(elem)
        }
        Ty::Tuple(ts) => ts.iter().find_map(find_pos_domain_in_ty),
        Ty::Record(fs) => fs.iter().find_map(|(_, t)| find_pos_domain_in_ty(t)),
        _ => None,
    }
}

fn root_place(place: &Place) -> PlaceKind {
    match &place.kind {
        PlaceKind::Local(s) => PlaceKind::Local(*s),
        PlaceKind::Global(g) => PlaceKind::Global(*g),
        PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
            root_place(base)
        }
    }
}

pub(super) fn place_to_expr(place: &Place) -> Expr {
    let kind = match &place.kind {
        PlaceKind::Local(s) => ExprKind::Local(*s),
        PlaceKind::Global(g) => ExprKind::Global(*g),
        PlaceKind::Field(base, idx) => ExprKind::Field(Box::new(place_to_expr(base)), *idx),
        PlaceKind::Some(base) => ExprKind::Unwrap(Box::new(place_to_expr(base)), false),
        PlaceKind::Index(base, idxs) => {
            ExprKind::Index(Box::new(place_to_expr(base)), idxs.clone())
        }
    };
    Expr {
        kind,
        ty: place.ty.clone(),
        span: place.span,
    }
}

fn uses_as_optional(name: &str, stmts: &[ast::Stmt]) -> bool {
    fn in_expr(name: &str, e: &ast::Expr) -> bool {
        match &e.kind {
            ast::ExprKind::Unwrap(inner) | ast::ExprKind::Try(inner) => {
                matches!(&inner.kind, ast::ExprKind::Name(n) if n == name) || in_expr(name, inner)
            }
            ast::ExprKind::Binary {
                op: BinOp::Coalesce,
                lhs,
                rhs,
                ..
            } => {
                matches!(&lhs.kind, ast::ExprKind::Name(n) if n == name)
                    || in_expr(name, lhs)
                    || in_expr(name, rhs)
            }
            ast::ExprKind::Compare { first, rest } => {
                let first_is_name = matches!(&first.kind, ast::ExprKind::Name(n) if n == name);
                let any_none = rest.iter().any(|(_, r)| matches!(r.kind, ast::ExprKind::None));
                (first_is_name && any_none)
                    || in_expr(name, first)
                    || rest.iter().any(|(_, r)| in_expr(name, r))
            }
            ast::ExprKind::If {
                branches,
                otherwise,
            } => {
                branches.iter().any(|(c, b)| {
                    in_cond(name, c) || uses_as_optional(name, &b.stmts)
                }) || otherwise
                    .as_ref()
                    .is_some_and(|b| uses_as_optional(name, &b.stmts))
            }
            _ => false,
        }
    }
    fn in_cond(name: &str, c: &ast::Cond) -> bool {
        match c {
            ast::Cond::Let { value, and, .. } => {
                matches!(&value.kind, ast::ExprKind::Name(n) if n == name)
                    || in_expr(name, value)
                    || and.as_ref().is_some_and(|a| in_cond(name, a))
            }
            ast::Cond::Expr(e) | ast::Cond::Const(e) => in_expr(name, e),
        }
    }
    stmts.iter().any(|s| match &s.kind {
        ast::StmtKind::Expr(e) => in_expr(name, e),
        ast::StmtKind::Let { init, .. } => init.as_ref().is_some_and(|e| in_expr(name, e)),
        ast::StmtKind::Assign { value, .. } => in_expr(name, value),
        ast::StmtKind::While { cond, body } => {
            in_cond(name, cond) || uses_as_optional(name, &body.stmts)
        }
        ast::StmtKind::For { body, .. } => uses_as_optional(name, &body.stmts),
        ast::StmtKind::Return(Some(e)) => in_expr(name, e),
        _ => false,
    })
}

/// A literal: in `var a: [V]Int = 0` it is the value of every cell.
fn is_literal(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Int(_)
        | ast::ExprKind::Float(_)
        | ast::ExprKind::Char(_)
        | ast::ExprKind::Bool(_)
        | ast::ExprKind::None => true,
        ast::ExprKind::Str(pieces) => pieces.iter().all(|p| matches!(p, ast::StrPiece::Text(_))),
        ast::ExprKind::Unary { expr, .. } | ast::ExprKind::Paren(expr) => is_literal(expr),
        ast::ExprKind::Tuple(items) => items.iter().all(is_literal),
        ast::ExprKind::List(items) => items.is_empty(),
        _ => false,
    }
}

/// Whether working `e` out again gives the same value and does nothing
/// else: names, literals and arithmetic on them.
fn settled(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Char(_) | ExprKind::Local(_) | ExprKind::Global(_) => true,
        ExprKind::Unary(_, inner) | ExprKind::Cast(_, inner) | ExprKind::Field(inner, _) => settled(inner),
        ExprKind::Binary(op, l, r) => {
            !matches!(op, BinOp::Div | BinOp::Rem | BinOp::Pow) && settled(l) && settled(r)
        }
        _ => false,
    }
}

/// The `return`, `break` or `continue` on the right of `x ?? return`.
pub(super) fn diverging_default(e: &ast::Expr) -> Option<&ast::Stmt> {
    let ast::ExprKind::Binary { op: BinOp::Coalesce, rhs, .. } = &e.kind else {
        return None;
    };
    match &rhs.kind {
        ast::ExprKind::Block(block) => match block.stmts.as_slice() {
            [stmt @ ast::Stmt {
                kind: ast::StmtKind::Return(_) | ast::StmtKind::Break(_) | ast::StmtKind::Continue(_),
                ..
            }] => Some(stmt),
            _ => None,
        },
        _ => None,
    }
}

fn plural(n: i64, what: &str) -> String {
    format!("{n} {what}{}", if n == 1 { "" } else { "s" })
}

/// Depth of nested `List.filled(_, ...)` calls wrapping a bare `none` literal.
fn filled_none_depth(e: &ast::Expr) -> Option<usize> {
    match &e.kind {
        ast::ExprKind::None => Some(0),
        ast::ExprKind::Paren(inner) => filled_none_depth(inner),
        ast::ExprKind::Call { callee, args } if args.len() == 2 => {
            if let ast::ExprKind::Field { base, name } = &callee.kind {
                if name.text == "filled" && matches!(&base.kind, ast::ExprKind::Name(n) if n == "List") {
                    return filled_none_depth(&args[1].value).map(|d| d + 1);
                }
            }
            None
        }
        _ => None,
    }
}
