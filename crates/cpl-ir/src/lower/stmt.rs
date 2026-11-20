//! Statements, blocks and patterns.

use super::*;

impl<'a> Lowerer<'a> {
    pub(super) fn block(&mut self, block: &ast::Block, tail: Tail) -> Block {
        self.cx.scopes.push(HashMap::new());
        self.cx.local_syms.push(HashMap::new());
        let mut out = Vec::new();
        match block.stmts.split_last() {
            None if tail == Tail::Give => self.needs_value(block.span),
            None => {}
            Some((last, init)) => {
                for (idx, stmt) in init.iter().enumerate() {
                    self.stmt_with_rest(stmt, &block.stmts[idx + 1..], &mut out);
                }
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

    fn stmt_with_rest(&mut self, stmt: &ast::Stmt, rest: &[ast::Stmt], out: &mut Vec<Stmt>) {
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
                    self.bind_mut(pat, value, *mutable, out);
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
            ast::StmtKind::Fn(decl) if !self.at_top_level() => {
                self.lower_local_fn(decl, out);
            }
            // Top-level declarations were handled by `collect_all`.
            ast::StmtKind::Fn(_)
            | ast::StmtKind::Struct(_)
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
                let (iter, elem) = self.iter(iter);
                let id = self.open_loop(label.as_ref().map(|l| l.text.clone()));
                self.cx.scopes.push(HashMap::new());
                self.cx.local_syms.push(HashMap::new());
                let mut stmts = Vec::new();
                let slot = self.loop_var(pat, elem, &mut stmts);
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
        let lo = self.expect(lo, &Ty::Int);
        let hi = self.expect(hi, &Ty::Int);
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
        if let Spec::Array { dims, elem } = &spec {
            let elem_ty = elem.ty();
            if is_literal(init) {
                let value = self.expect(init, &elem_ty);
                return Some(self.mk(
                    ExprKind::Fill(dims.clone(), Some(Box::new(value))),
                    want,
                    init.span,
                ));
            }
            if let Some(table) = self.comprehension_table(init, dims, &elem_ty) {
                return Some(table);
            }
            let value = self.expr(init, Some(&want));
            if value.ty == want || value.ty == Ty::Error {
                return Some(value);
            }
            return Some(match self.try_coerce(value, &elem_ty) {
                Ok(value) => self.mk(
                    ExprKind::Fill(dims.clone(), Some(Box::new(value))),
                    want,
                    init.span,
                ),
                Err(value) => {
                    self.mismatch(init.span, &want, &value.ty);
                    self.err_expr(init.span)
                }
            });
        }
        Some(self.expect(init, &want))
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
            // `(a, b) = (b, a)`: the right side is whole before any store.
            let value = self.expr(value, None);
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
        let rhs = match op {
            ast::AssignOp::Set => self.expect(value, &place.ty),
            _ => self.expr(value, Some(&place.ty)),
        };
        self.emit_assign_place(place, op, rhs, target.span, out);
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
                let value = match (&place.ty, bin) {
                    (
                        Ty::Int,
                        Add | Sub | Mul | Div | Rem | Pow | BitAnd | BitOr | BitXor | Shl | Shr,
                    ) => self.coerce(value, &Ty::Int, span),
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

    pub(super) fn try_place(&mut self, e: &ast::Expr) -> Option<Place> {
        let diag_len = self.diags.len();
        let p = self.place(e);
        if self.diags.len() > diag_len || p.ty == Ty::Error {
            self.diags.truncate(diag_len);
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
            ast::ExprKind::Paren(inner) => self.place(inner),
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
            ast::Cond::Const(cond) => self.unsupported(cond.span, "`if const`"),
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
        PlaceKind::Index(base, _) | PlaceKind::Field(base, _) => root_place(base),
    }
}

pub(super) fn place_to_expr(place: &Place) -> Expr {
    let kind = match &place.kind {
        PlaceKind::Local(s) => ExprKind::Local(*s),
        PlaceKind::Global(g) => ExprKind::Global(*g),
        PlaceKind::Field(base, idx) => ExprKind::Field(Box::new(place_to_expr(base)), *idx),
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
