//! Statements, blocks and patterns.

use super::*;

/// What writing through `x!` says when `x` is `none`; reading it says so too.
const NO_VALUE: &str = "`!` on `none`: there is no value here";

impl<'a> Lowerer<'a> {
    pub(super) fn block(&mut self, block: &ast::Block, tail: Tail) -> Block {
        self.cx.scopes.push(HashMap::new());
        self.cx.local_syms.push(HashMap::new());
        let present = self.cx.present.len();
        let held = (self.contexts.len(), self.unchecked);
        let mut out = Vec::new();
        match block.stmts.split_last() {
            None if tail == Tail::Give => self.needs_value(block.span),
            None => {}
            Some((last, init)) => {
                let mut terminated = false;
                for (idx, stmt) in init.iter().enumerate() {
                    let rest = &block.stmts[idx + 1..];
                    self.stmt_with_rest(stmt, rest, &mut out);
                    let mutators = self.mutators.clone();
                    let held = narrow::after(stmt, rest, &mut |f, x| mutators.writes(f, x));
                    self.cx.present.extend(held.into_iter().map(String::from));
                    if matches!(out.last(), Some(Stmt::Return(_))) {
                        terminated = true;
                        break;
                    }
                }
                if !terminated {
                    match &last.kind {
                        ast::StmtKind::Expr(e) if tail != Tail::Discard => {
                            self.tail(e, tail, &mut out)
                        }
                        _ => {
                            self.stmt(last, &mut out);
                            // An arm that leaves gives no value and needs none.
                            let leaves =
                                leaves(last) && self.cx.dos.last().is_some_and(|frame| frame.open);
                            if tail == Tail::Give && !leaves {
                                self.needs_value(last.span);
                            }
                        }
                    }
                }
            }
        }
        self.cx.present.truncate(present);
        self.contexts.truncate(held.0);
        self.unchecked = held.1;
        self.cx.local_syms.pop();
        self.cx.scopes.pop();
        let depth = self.cx.scopes.len();
        self.cx.recipes.retain(|_, (d, _)| *d <= depth);
        Block { stmts: out }
    }

    /// A statement that only works out a value: `xs.sorted()`, `s.with(v)`,
    /// `a + b`. Nothing reads it and nothing was changed, so it is a
    /// mistake, most often for the form that changes in place.
    fn unused_value(&mut self, value: &Expr, span: Span) {
        if matches!(value.ty, Ty::Unit | Ty::Error) {
            return;
        }
        let only_a_value = match &value.kind {
            ExprKind::Builtin(b, _) => !matches!(
                b,
                Builtin::Flush
                    | Builtin::CountQuery
                    | Builtin::SwapInput
                    | Builtin::AtEof
                    | Builtin::Rand
            ),
            // A library fn that takes nothing by `mut`. One of the
            // file's own may print or change a global.
            ExprKind::Call(id, args) => {
                let sig = &self.sigs[*id];
                sig.mod_id != 0
                    && sig.receiver != Some(true)
                    && args.iter().all(|a| matches!(a, Arg::Value(_)))
            }
            ExprKind::Binary(..)
            | ExprKind::Unary(..)
            | ExprKind::Cmp(..)
            | ExprKind::Cast(..)
            | ExprKind::Index(..)
            | ExprKind::Field(..)
            | ExprKind::Tuple(..)
            | ExprKind::Local(..)
            | ExprKind::Global(..)
            | ExprKind::Int(..)
            | ExprKind::Str(..) => true,
            _ => false,
        };
        if only_a_value {
            let shown = self.show(&value.ty);
            let span = self.span(span);
            self.warnings.push(Diagnostic::warning(
                codes::UNUSED_VALUE,
                span,
                format!("this `{shown}` is worked out and then dropped"),
            ));
            if let Some(last) = self.warnings.last_mut() {
                last.help =
                    Some("keep it (`let ys = ...`), or call the form that changes in place".into());
            }
        }
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
            _ => {
                let outer_pending = std::mem::take(&mut self.cx.pending);
                let at = out.len();
                self.tail_value(e, tail, out);
                let pending = std::mem::replace(&mut self.cx.pending, outer_pending);
                out.splice(at..at, pending);
                self.lift_leaving(out, at);
            }
        }
    }

    fn tail_value(&mut self, e: &ast::Expr, tail: Tail, out: &mut Vec<Stmt>) {
        match tail {
            Tail::Discard => {
                let value = self.expr(e, None);
                self.unused_value(&value, e.span);
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
        }
    }

    fn is_mut_method_name(name: &str) -> bool {
        matches!(
            name,
            "sort"
                | "reverse"
                | "rotate_left"
                | "rotate_right"
                | "scan_in_place"
                | "suffix_scan_in_place"
                | "next_permutation"
                | "prev_permutation"
                | "nth_element"
                | "swap"
                | "swap_at"
                | "swap_idx"
                | "set"
                | "set_at"
                | "fill"
                | "shuffle"
        )
    }

    fn collect_mut_slices_expr(&mut self, e: &ast::Expr) {
        match &e.kind {
            ast::ExprKind::Call { callee, args, .. } => {
                for a in args {
                    if a.mutable && matches!(a.value.kind, ast::ExprKind::Index { .. }) {
                        let sp = self.span(a.value.span);
                        self.mut_slice_inits.insert(sp);
                    }
                    self.collect_mut_slices_expr(&a.value);
                }
                if let ast::ExprKind::Field { base, name } = &callee.kind {
                    if matches!(base.kind, ast::ExprKind::Index { .. })
                        && Self::is_mut_method_name(&name.text)
                    {
                        let sp = self.span(base.span);
                        self.mut_slice_inits.insert(sp);
                    }
                }
                self.collect_mut_slices_expr(callee);
            }
            ast::ExprKind::Paren(inner) | ast::ExprKind::Unary { expr: inner, .. } => {
                self.collect_mut_slices_expr(inner);
            }
            ast::ExprKind::Binary { lhs, rhs, .. } => {
                self.collect_mut_slices_expr(lhs);
                self.collect_mut_slices_expr(rhs);
            }
            ast::ExprKind::Index { base, indices } => {
                self.collect_mut_slices_expr(base);
                for i in indices {
                    self.collect_mut_slices_expr(i);
                }
            }
            ast::ExprKind::Field { base, .. } => self.collect_mut_slices_expr(base),
            ast::ExprKind::Lambda { body, .. } => self.collect_mut_slices_expr(body),
            _ => {}
        }
    }

    fn collect_mut_slices_stmt(&mut self, stmt: &ast::Stmt, rest: &[ast::Stmt]) {
        match &stmt.kind {
            ast::StmtKind::Let {
                pat,
                init: Some(init),
                ..
            } => {
                if let ast::PatternKind::Name(name) = &pat.kind {
                    if matches!(init.kind, ast::ExprKind::Index { .. })
                        && Self::stmts_mutate_name(rest, name)
                    {
                        let sp = self.span(init.span);
                        self.mut_slice_inits.insert(sp);
                    }
                }
                self.collect_mut_slices_expr(init);
            }
            ast::StmtKind::Assign { target, value, .. } => {
                if let ast::ExprKind::Index { base, .. } = &target.kind {
                    if matches!(base.kind, ast::ExprKind::Index { .. }) {
                        let sp = self.span(base.span);
                        self.mut_slice_inits.insert(sp);
                    }
                }
                self.collect_mut_slices_expr(target);
                self.collect_mut_slices_expr(value);
            }
            ast::StmtKind::Expr(e) | ast::StmtKind::Return(Some(e)) => {
                self.collect_mut_slices_expr(e);
            }
            _ => {}
        }
    }

    fn expr_root_name(e: &ast::Expr) -> Option<&str> {
        match &e.kind {
            ast::ExprKind::Name(n) => Some(n.as_str()),
            ast::ExprKind::Paren(inner) => Self::expr_root_name(inner),
            ast::ExprKind::Index { base, .. } | ast::ExprKind::Field { base, .. } => {
                Self::expr_root_name(base)
            }
            _ => None,
        }
    }

    fn cond_mutates_name(c: &ast::Cond, name: &str) -> bool {
        match c {
            ast::Cond::Expr(e) | ast::Cond::Const(e) => Self::expr_mutates_name(e, name),
            ast::Cond::Let { value, .. } => Self::expr_mutates_name(value, name),
            ast::Cond::Guard { test, and } => {
                Self::expr_mutates_name(test, name) || Self::cond_mutates_name(and, name)
            }
        }
    }

    fn expr_mutates_name(e: &ast::Expr, name: &str) -> bool {
        match &e.kind {
            ast::ExprKind::Call { callee, args, .. } => {
                if args
                    .iter()
                    .any(|a| a.mutable && Self::expr_root_name(&a.value) == Some(name))
                {
                    return true;
                }
                if let ast::ExprKind::Field { base, name: method } = &callee.kind {
                    if Self::expr_root_name(base) == Some(name)
                        && matches!(
                            method.text.as_str(),
                            "sort"
                                | "reverse"
                                | "rotate_left"
                                | "rotate_right"
                                | "scan_in_place"
                                | "suffix_scan_in_place"
                                | "next_permutation"
                                | "prev_permutation"
                                | "nth_element"
                                | "swap"
                                | "swap_at"
                                | "swap_idx"
                                | "set"
                                | "set_at"
                                | "fill"
                                | "shuffle"
                        )
                    {
                        return true;
                    }
                }
                Self::expr_mutates_name(callee, name)
                    || args.iter().any(|a| Self::expr_mutates_name(&a.value, name))
            }
            ast::ExprKind::Block(b) => Self::stmts_mutate_name(&b.stmts, name),
            ast::ExprKind::If {
                branches,
                otherwise,
            } => {
                branches.iter().any(|(c, b)| {
                    Self::cond_mutates_name(c, name) || Self::stmts_mutate_name(&b.stmts, name)
                }) || otherwise
                    .as_ref()
                    .is_some_and(|b| Self::stmts_mutate_name(&b.stmts, name))
            }
            ast::ExprKind::Match { scrutinee, arms } => {
                Self::expr_mutates_name(scrutinee, name)
                    || arms.iter().any(|a| {
                        a.guard
                            .as_ref()
                            .is_some_and(|g| Self::expr_mutates_name(g, name))
                            || Self::stmts_mutate_name(&a.body.stmts, name)
                    })
            }
            ast::ExprKind::Paren(inner) | ast::ExprKind::Unary { expr: inner, .. } => {
                Self::expr_mutates_name(inner, name)
            }
            ast::ExprKind::Binary { lhs, rhs, .. } => {
                Self::expr_mutates_name(lhs, name) || Self::expr_mutates_name(rhs, name)
            }
            ast::ExprKind::Index { base, indices } => {
                Self::expr_mutates_name(base, name)
                    || indices.iter().any(|i| Self::expr_mutates_name(i, name))
            }
            ast::ExprKind::Field { base, .. } => Self::expr_mutates_name(base, name),
            ast::ExprKind::Lambda { body, .. } => Self::expr_mutates_name(body, name),
            _ => false,
        }
    }

    /// `let g = gen(args)` names a generator value, a recipe that runs
    /// afresh on every walk (R06.3.1). It is kept as the call when the
    /// arguments read the same at every walk and no closure or fn captures `g`.
    fn generator_recipe<'s>(
        &self,
        stmt: &'s ast::Stmt,
        rest: &[ast::Stmt],
    ) -> Option<(String, &'s ast::Expr)> {
        let ast::StmtKind::Let {
            mutable: false,
            pat,
            ty: None,
            init: Some(init),
            otherwise: None,
            guard: None,
        } = &stmt.kind
        else {
            return None;
        };
        let ast::PatternKind::Name(name) = &pat.kind else {
            return None;
        };
        let ast::ExprKind::Call { callee, args } = &init.kind else {
            return None;
        };
        let Some(Sym::Fn(id)) = self.sym(callee) else {
            return None;
        };
        if !self.sigs[id].is_generator {
            return None;
        }
        let steady = args.iter().all(|a| match &a.value.kind {
            ast::ExprKind::Int(_) | ast::ExprKind::Bool(_) | ast::ExprKind::Char(_) => true,
            ast::ExprKind::Name(n) => n != name && (a.mutable || !Self::stmts_mutate_name(rest, n)),
            ast::ExprKind::Mut(inner) => matches!(&inner.kind, ast::ExprKind::Name(n) if n != name),
            _ => false,
        });
        if !steady || Self::stmts_mutate_name(rest, name) || captured(rest, name) {
            return None;
        }
        Some((name.clone(), init))
    }

    fn stmts_mutate_name(stmts: &[ast::Stmt], name: &str) -> bool {
        for (i, s) in stmts.iter().enumerate() {
            match &s.kind {
                ast::StmtKind::Assign { target, value, .. } => {
                    if Self::expr_root_name(target) == Some(name)
                        || Self::expr_mutates_name(target, name)
                        || Self::expr_mutates_name(value, name)
                    {
                        return true;
                    }
                }
                ast::StmtKind::Expr(e) => {
                    if Self::expr_mutates_name(e, name) {
                        return true;
                    }
                }
                ast::StmtKind::Let {
                    pat,
                    init: Some(init),
                    ..
                } => {
                    if Self::expr_mutates_name(init, name) {
                        return true;
                    }
                    if let ast::PatternKind::Name(sub) = &pat.kind {
                        if Self::expr_root_name(init) == Some(name)
                            && Self::stmts_mutate_name(&stmts[i + 1..], sub)
                        {
                            return true;
                        }
                        if sub == name {
                            return false;
                        }
                    }
                }
                ast::StmtKind::For { iter, body, .. } => {
                    if Self::expr_mutates_name(iter, name)
                        || Self::stmts_mutate_name(&body.stmts, name)
                    {
                        return true;
                    }
                }
                ast::StmtKind::While { cond, body, .. } => {
                    if Self::cond_mutates_name(cond, name)
                        || Self::stmts_mutate_name(&body.stmts, name)
                    {
                        return true;
                    }
                }
                ast::StmtKind::Return(Some(e)) if Self::expr_mutates_name(e, name) => {
                    return true;
                }
                _ => {}
            }
        }
        false
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
        self.collect_mut_slices_stmt(stmt, rest);
        // `let u = o ?? return -1`: the test and the way out go ahead of
        // the statement, which then reads the value that is there.
        let root = match &stmt.kind {
            ast::StmtKind::Let {
                init: Some(e),
                otherwise: None,
                ..
            }
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
        let way_out = root
            .filter(|e| diverging_default(e).is_some())
            .map(|e| e.span);
        let exits = root.filter(|e| has_leaving_arm(e)).map(|e| e.span);
        let outer = (
            std::mem::replace(&mut self.cx.way_out, way_out),
            std::mem::take(&mut self.cx.pending),
            std::mem::replace(&mut self.cx.exits, exits),
        );
        let at = out.len();
        self.stmt_inner(stmt, rest, out);
        let pending = std::mem::replace(&mut self.cx.pending, outer.1);
        self.cx.way_out = outer.0;
        self.cx.exits = outer.2;
        out.splice(at..at, pending);
        self.lift_leaving(out, at);
    }

    fn stmt_inner(&mut self, stmt: &ast::Stmt, rest: &[ast::Stmt], out: &mut Vec<Stmt>) {
        if let Some((name, init)) = self.generator_recipe(stmt, rest) {
            if let Some(scope) = self.cx.scopes.last_mut() {
                scope.remove(&name);
            }
            let depth = self.cx.scopes.len();
            self.cx.recipes.insert(name, (depth, init.clone()));
            return;
        }
        match &stmt.kind {
            ast::StmtKind::Let {
                mutable,
                pat,
                ty: None,
                init: Some(init),
                otherwise: Some(els_block),
                ..
            } if !*mutable && refutable(pat) => {
                let value = self.expr(init, None);
                if value.ty == Ty::Error {
                    return;
                }
                let tmp = self.temp(value.ty.clone());
                out.push(Stmt::Let(tmp, Some(value)));
                let local = self.local(tmp, init.span);
                let mut binds = Vec::new();
                if let Some(test) = self.pattern(pat, &local, &mut binds) {
                    let fails = Expr {
                        kind: ExprKind::Unary(UnOp::Not, Box::new(test)),
                        ty: Ty::Bool,
                        span: self.span(stmt.span),
                    };
                    let els = self.block(els_block, Tail::Discard);
                    out.push(Stmt::If(fails, els, Block::default()));
                }
                out.extend(binds);
            }
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
                // §127: `let f = (n: Int) -> Int => ...` that refers to `f`
                // recursively lowers like a local `fn` (self-passing generic
                // lambda in C++, no `std::function`).
                if !*mutable {
                    if let (
                        ast::PatternKind::Name(vname),
                        Some(ast::Expr {
                            kind:
                                ast::ExprKind::Lambda {
                                    params,
                                    param_tys,
                                    ret_ty,
                                    body,
                                    ..
                                },
                            span: lspan,
                        }),
                    ) = (&pat.kind, init)
                    {
                        if expr_mentions_name(body, vname)
                            && params
                                .iter()
                                .all(|p| matches!(p.kind, ast::PatternKind::Name(_)))
                        {
                            let fn_ty_parts = match ty {
                                Some(ast::Type {
                                    kind:
                                        ast::TypeKind::Fn {
                                            params: fparams,
                                            ret: fret,
                                        },
                                    ..
                                }) => Some((fparams.as_slice(), Some((**fret).clone()))),
                                _ => None,
                            };
                            let synth_params: Vec<ast::Param> = params
                                .iter()
                                .enumerate()
                                .map(|(i, p)| {
                                    let pname = match &p.kind {
                                        ast::PatternKind::Name(s) => s.clone(),
                                        _ => unreachable!(),
                                    };
                                    let pty =
                                        param_tys.get(i).and_then(|t| t.clone()).or_else(|| {
                                            fn_ty_parts
                                                .as_ref()
                                                .and_then(|(fps, _)| fps.get(i).cloned())
                                        });
                                    ast::Param {
                                        label: None,
                                        name: ast::Name {
                                            text: pname,
                                            span: p.span,
                                        },
                                        ty: pty,
                                        mutable: false,
                                        default: None,
                                        named_only: false,
                                        variadic: false,
                                        lazy: false,
                                        span: p.span,
                                    }
                                })
                                .collect();
                            let synth_ret =
                                ret_ty.clone().or_else(|| fn_ty_parts.and_then(|(_, r)| r));
                            let synth_body = match &body.kind {
                                ast::ExprKind::Block(b) => ast::FnBody::Block(b.clone()),
                                _ => ast::FnBody::Expr((**body).clone()),
                            };
                            let synth = ast::FnDecl {
                                name: ast::Name {
                                    text: vname.clone(),
                                    span: pat.span,
                                },
                                generics: Vec::new(),
                                bounds: Vec::new(),
                                params: synth_params,
                                ret: synth_ret,
                                lends: None,
                                uses: Vec::new(),
                                body: synth_body,
                                sig_span: *lspan,
                            };
                            self.lower_local_fn(&synth, false, None, out);
                            return;
                        }
                    }
                }
                // `let (n, c, m): (1..=100, 1..=100, c..=n * c) = read()`:
                // read and bind each element in order so later element types
                // (e.g. dependent ranges `c..=n * c`) can see earlier bindings.
                if let (
                    ast::PatternKind::Tuple(pats),
                    Some(ast::Type {
                        kind: ast::TypeKind::Tuple(tys),
                        ..
                    }),
                    Some(init_expr),
                ) = (&pat.kind, ty.as_ref(), init.as_ref())
                {
                    if pats.len() == tys.len()
                        && tys.iter().all(|t| {
                            matches!(
                                t.kind,
                                ast::TypeKind::Range { .. } | ast::TypeKind::Named { .. }
                            )
                        })
                        && is_read_call(init_expr)
                        && matches!(&init_expr.kind, ast::ExprKind::Call { args, .. } if args.is_empty())
                    {
                        for (p_i, t_i) in pats.iter().zip(tys) {
                            let spec = self.resolve(t_i, true);
                            let val = self.read(spec, init_expr.span);
                            self.bind_mut(p_i, val, *mutable, out);
                            if let (
                                false,
                                ast::PatternKind::Name(name),
                                ast::TypeKind::Range {
                                    lo,
                                    hi: Some(hi),
                                    inclusive,
                                },
                            ) = (*mutable, &p_i.kind, &t_i.kind)
                            {
                                if self.cx.is_main && self.cx.scopes.len() == 1 {
                                    if let Some(Sym::Global(gid)) =
                                        self.mod_syms[self.cur_mod].get(name).copied()
                                    {
                                        let least = match lo {
                                            Some(lo) => self.ast_bounds(lo).map(|b| b.0),
                                            None => Some(0),
                                        };
                                        let most = self
                                            .ast_bounds(hi)
                                            .map(|b| b.1 - i64::from(!*inclusive));
                                        if let (Some(least), Some(most)) = (least, most) {
                                            self.global_bounds.insert(gid, (least, most));
                                        }
                                    }
                                }
                            }
                            if let Some(rinfo) = self.range_info_from_ast(t_i) {
                                self.bind_range_info(p_i, &rinfo);
                            }
                        }
                        return;
                    }
                }
                // `let st = mut a[i]`: bind `st` as a mutable alias to the place `a[i]`,
                // evaluating any index subexpressions once into temporary locals.
                if let (
                    ast::PatternKind::Name(alias_name),
                    Some(ast::Expr {
                        kind: ast::ExprKind::Mut(target),
                        ..
                    }),
                ) = (&pat.kind, init)
                {
                    let raw = self.place(target);
                    let frozen = self.freeze_place(raw, out);
                    self.cx.mut_aliases.insert(alias_name.clone(), frozen);
                    return;
                }
                // `let v = s.range(r)` (or `let (a, b) = (s.range(r1), s.range(r2))`):
                // bind zero-copy O(1) `SortedView` handles in `self.cx.sorted_views`.
                if let Some(init_expr) = init {
                    if let ast::PatternKind::Name(vname) = &pat.kind {
                        if self.is_sorted_range_expr(init_expr) {
                            if let Some(view) = self.sorted_view_from_expr(init_expr, out) {
                                let slot = self.temp(Ty::Unit);
                                if let Some(scope) = self.cx.scopes.last_mut() {
                                    scope.insert(vname.clone(), slot);
                                }
                                self.cx.sorted_views.insert(slot, view);
                                return;
                            }
                        }
                    } else if let (ast::PatternKind::Tuple(pats), ast::ExprKind::Tuple(exprs)) =
                        (&pat.kind, &init_expr.kind)
                    {
                        if pats.len() == exprs.len()
                            && exprs.iter().any(|e| self.is_sorted_range_expr(e))
                        {
                            for (p, e) in pats.iter().zip(exprs.iter()) {
                                if let ast::PatternKind::Name(vname) = &p.kind {
                                    if self.is_sorted_range_expr(e) {
                                        if let Some(view) = self.sorted_view_from_expr(e, out) {
                                            let slot = self.temp(Ty::Unit);
                                            if let Some(scope) = self.cx.scopes.last_mut() {
                                                scope.insert(vname.clone(), slot);
                                            }
                                            self.cx.sorted_views.insert(slot, view);
                                            continue;
                                        }
                                    }
                                }
                                let val = self.expr(e, None);
                                self.bind_mut(p, val, *mutable, out);
                            }
                            return;
                        }
                    }
                }
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
                            let hashed =
                                matches!(&callee.kind, ast::ExprKind::Name(n) if n == "HashMap");
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
                    let inferred_rinfo = if ty.is_none() {
                        self.expr_range(&value)
                            .filter(|r| r.narrow_elem_layout(&value.ty).is_some())
                    } else {
                        None
                    };
                    let single_name = match &pat.kind {
                        ast::PatternKind::Name(n) => Some(n.clone()),
                        ast::PatternKind::Variant { path, args }
                            if path.len() == 1 && args.is_empty() =>
                        {
                            Some(path[0].text.clone())
                        }
                        _ => None,
                    };
                    let array_ty_ast = if let (Some(ty_ast), Some(_)) = (ty.as_ref(), &single_name)
                    {
                        if matches!(ty_ast.kind, ast::TypeKind::Array { .. }) {
                            Some(ty_ast.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let runtime_const_name =
                        if !*mutable && !self.at_top_level() && value.ty == Ty::Int {
                            single_name.clone().filter(|n| {
                                n.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                            })
                        } else {
                            None
                        };
                    self.bind_mut(pat, value, *mutable, out);
                    if let (
                        false,
                        Some(name),
                        Some(ast::TypeKind::Range {
                            lo,
                            hi: Some(hi),
                            inclusive,
                        }),
                    ) = (*mutable, &single_name, ty.as_ref().map(|t| &t.kind))
                    {
                        if self.cx.is_main && self.cx.scopes.len() == 1 {
                            if let Some(Sym::Global(gid)) =
                                self.mod_syms[self.cur_mod].get(name).copied()
                            {
                                let least = match lo {
                                    Some(lo) => self.ast_bounds(lo).map(|b| b.0),
                                    None => Some(0),
                                };
                                let most =
                                    self.ast_bounds(hi).map(|b| b.1 - i64::from(!*inclusive));
                                if let (Some(least), Some(most)) = (least, most) {
                                    self.global_bounds.insert(gid, (least, most));
                                }
                            }
                        }
                    }
                    if let Some(ty_ast) = ty.as_ref() {
                        if let Some(rinfo) = self.range_info_from_ast(ty_ast) {
                            self.bind_range_info(pat, &rinfo);
                        }
                    } else if let Some(rinfo) = inferred_rinfo {
                        self.bind_range_info(pat, &rinfo);
                    }
                    if let (false, Some(name), None, Some(init)) =
                        (*mutable, &single_name, ty.as_ref(), init.as_ref())
                    {
                        if self.cx.is_main && self.cx.scopes.len() == 1 {
                            if let (Some(Sym::Global(gid)), Some(bounds)) = (
                                self.mod_syms[self.cur_mod].get(name).copied(),
                                self.ast_bounds(init),
                            ) {
                                self.global_bounds.insert(gid, bounds);
                            }
                        }
                    }
                    if let (Some(ty_ast), Some(vname)) = (array_ty_ast, single_name) {
                        if self.cx.is_main && self.cx.scopes.len() == 1 {
                            if let Some(Sym::Global(gid)) =
                                self.mod_syms[self.cur_mod].get(&vname).copied()
                            {
                                self.global_specs.insert(gid, ty_ast);
                            }
                        } else if let Some(slot) = self.lookup(&vname) {
                            self.cx.local_specs.insert(slot, ty_ast);
                        }
                    }
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
                    if let ExprKind::Bool(v) = value.kind {
                        self.const_bools.insert(name.text.clone(), v);
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
                                    pat:
                                        ast::Pattern {
                                            kind: ast::PatternKind::Name(n),
                                            ..
                                        },
                                    init: Some(init),
                                    ..
                                } = &s.kind
                                {
                                    if n == "K" {
                                        let len = self.expect(init, &Ty::Int);
                                        let lo = self.int_lit(0, init.span);
                                        out.push(Stmt::DefDomain {
                                            id: dom_id,
                                            lo,
                                            len,
                                        });
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
                let mod_name = path
                    .iter()
                    .map(|n| n.text.as_str())
                    .collect::<Vec<_>>()
                    .join(".");
                let mod_slash = path
                    .iter()
                    .map(|n| n.text.as_str())
                    .collect::<Vec<_>>()
                    .join("/");
                let last_cp = path
                    .last()
                    .map(|n| format!("/{}.cp", n.text))
                    .unwrap_or_default();
                let target_mod = self.mods[self.cur_mod]
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
                let into = super::memo::into_table(stmt);
                if let Some(table) = into {
                    self.memo_into_table(table, decl, out);
                }
                self.lower_local_fn(decl, memo, into, out);
            }
            // Top-level declarations were handled by `collect_all`.
            ast::StmtKind::Fn(decl) => {
                if let Some(table) = super::memo::into_table(stmt) {
                    self.memo_into_table(table, decl, out);
                }
            }
            ast::StmtKind::Enum(decl) => {
                if let Some(Sym::Domain(d)) = self.get_sym(&decl.name.text) {
                    if self.sum_enums.contains_key(&d) {
                        self.def_sum_enum_domain(d, stmt.span, out);
                    }
                }
            }
            ast::StmtKind::Struct(_)
            | ast::StmtKind::TypeAlias { .. }
            | ast::StmtKind::Trait { .. }
            | ast::StmtKind::Impl { .. }
            | ast::StmtKind::Use { .. }
            | ast::StmtKind::Law { .. }
            | ast::StmtKind::Lend { .. } => {}
            ast::StmtKind::Domain { name, def } => self.domain(name, def, out),
            ast::StmtKind::Expr(e) => {
                if !self.drop_key(e, out) {
                    self.tail(e, Tail::Discard, out)
                }
            }
            ast::StmtKind::Assign { target, op, value } => self.assign(target, *op, value, out),
            ast::StmtKind::While {
                cond,
                body,
                else_body,
            } => self.while_stmt(cond, body, else_body.as_ref(), out),
            ast::StmtKind::For {
                label,
                pat,
                iter,
                body,
                else_body,
            } => {
                if let Some(values) = self.reflected_values(iter) {
                    self.fields_loop(pat, values, body, out);
                    if let Some(else_body) = else_body {
                        out.extend(self.block(else_body, Tail::Discard).stmts);
                    }
                    return;
                }
                // `for (i, j) in V.pairs` and `for (r, c) in Square` over a
                // product domain walk nested integer loops: no list, no div/mod.
                if let Some(dom_id) = self.domain_pairs_iter(iter) {
                    let span = iter.span;
                    let len = self.mk(ExprKind::DomainLen(dom_id), Ty::Int, span);
                    let i_slot = self.temp(Ty::Pos(dom_id));
                    let j_slot = self.temp(Ty::Pos(dom_id));
                    let i_var = self.mk(ExprKind::Local(i_slot), Ty::Pos(dom_id), span);
                    let j_var = self.mk(ExprKind::Local(j_slot), Ty::Pos(dom_id), span);
                    let j_lo = self.int_add(self.cast(Cast::Ord, i_var.clone(), Ty::Int), 1);
                    let heads = vec![
                        (i_slot, self.int_lit(0, span), len.clone()),
                        (j_slot, j_lo, len),
                    ];
                    let pair = self.mk(
                        ExprKind::Tuple(vec![i_var, j_var]),
                        Ty::Tuple(vec![Ty::Pos(dom_id), Ty::Pos(dom_id)]),
                        span,
                    );
                    self.nested_for(
                        label.as_ref(),
                        heads,
                        pat,
                        pair,
                        body,
                        else_body.as_ref(),
                        out,
                    );
                    return;
                }
                if let Some((heads, cell)) = self.product_heads(pat, iter) {
                    self.nested_for(
                        label.as_ref(),
                        heads,
                        pat,
                        cell,
                        body,
                        else_body.as_ref(),
                        out,
                    );
                    return;
                }
                // Support compile-time unrolling of heterogeneous literal lists like
                // `for (k, act) in [(0, AddSum), ..., (0, AssignSum)]:` where items have different struct types.
                if let ast::ExprKind::List(items) = &iter.kind {
                    if items.len() >= 2 {
                        let first_ty = self.probe_ty_silent(&items[0]);
                        let is_hetero = first_ty.as_ref().is_some_and(|ft| {
                            items[1..]
                                .iter()
                                .any(|it| self.probe_ty_silent(it).is_some_and(|ot| ot != *ft))
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
                            if let Some(else_body) = else_body {
                                out.extend(self.block(else_body, Tail::Discard).stmts);
                            }
                            return;
                        }
                    }
                }
                let cells = self.grid_items_head(iter, out);
                let items = if cells.is_some() {
                    None
                } else {
                    self.items_head(iter)
                };
                let map_proj = if cells.is_some() || items.is_some() {
                    None
                } else {
                    self.map_proj_head(iter)
                };
                let (iter, elem) = match (&cells, &items, &map_proj) {
                    (Some(cells), _, _) => (cells.range(), Ty::Unit),
                    (None, Some(items), _) => (items.range(), Ty::Unit),
                    (None, None, Some(proj)) => (proj.iter(), Ty::Unit),
                    (None, None, None) => self.iter(iter),
                };
                let id = self.open_loop(label.as_ref().map(|l| l.text.clone()));
                let else_flag = self.begin_loop_else(id, else_body.as_ref(), out);
                self.cx.scopes.push(HashMap::new());
                self.cx.local_syms.push(HashMap::new());
                let mut stmts = Vec::new();
                let elem_is_int = elem == Ty::Int;
                let slot = match (&cells, &items, &map_proj) {
                    (Some(cells), _, _) => self.grid_items_bind(pat, cells, &mut stmts),
                    (None, Some(items), _) => self.items_bind(pat, items, &mut stmts),
                    (None, None, Some(proj)) => self.map_proj_bind(pat, proj, &mut stmts),
                    (None, None, None) => self.loop_var(pat, elem, &mut stmts),
                };
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
                self.end_loop_else(id, else_flag, else_body.as_ref(), out);
            }
            ast::StmtKind::Return(value) => {
                if self.cx.dos.iter().any(|frame| !frame.open) {
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
                        if matches!(hint, Some(Ty::Unit)) && matches!(value.ty, Ty::Unit) {
                            out.push(Stmt::Expr(value));
                            out.push(Stmt::Return(None));
                            return;
                        }
                        match hint {
                            Some(ty) => value = self.coerce(value, &ty, e.span),
                            None => self.cx.ret = Some(value.ty.clone()),
                        }
                        if matches!(value.ty, Ty::Unit) {
                            out.push(Stmt::Expr(value));
                            out.push(Stmt::Return(None));
                        } else {
                            out.push(Stmt::Return(Some(value)));
                        }
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
                let to = self.cx.loops[index].1;
                let break_id = self.cx.break_out.get(&to).copied().unwrap_or(to);
                if is_break {
                    if let Some(&flag) = self.cx.loop_else.get(&break_id) {
                        out.push(Stmt::Assign(
                            Place {
                                kind: PlaceKind::Local(flag),
                                ty: Ty::Bool,
                                span: self.span(stmt.span),
                            },
                            AssignOp::Set,
                            self.mk(ExprKind::Bool(false), Ty::Bool, stmt.span),
                        ));
                    }
                }
                let target = match self.cx.break_out.get(&to) {
                    Some(&outer) if is_break => Some(outer),
                    _ => (index + 1 != self.cx.loops.len()).then_some(to),
                };
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
            ast::StmtKind::Unchecked(block) => {
                self.unchecked += 1;
                self.overflow_checks_off += 1;
                out.extend(self.block(block, Tail::Discard).stmts);
                self.overflow_checks_off -= 1;
                self.unchecked -= 1;
            }
            ast::StmtKind::Context { name, fields } => {
                let span = stmt.span;
                let mut field_tys = Vec::new();
                let mut field_vals = Vec::new();
                for f in fields {
                    let fty = self.resolve_ty(&f.ty);
                    let val = if let Some(d) = &f.default {
                        self.expect(d, &fty)
                    } else {
                        self.mk(ExprKind::Int(0), fty.clone(), f.name.span)
                    };
                    field_tys.push((f.name.text.clone(), fty));
                    field_vals.push(val);
                }
                let rec_ty = Ty::Record(field_tys.clone());
                let gid = if let Some((existing, _)) = self.context_defs.get(&name.text) {
                    *existing
                } else {
                    let gid = self.globals.len();
                    self.globals.push(Global {
                        name: format!("_ctx_{}", name.text),
                        ty: rec_ty.clone(),
                        span,
                    });
                    self.mut_globals.insert(gid);
                    self.mod_syms[self.cur_mod].insert(name.text.clone(), Sym::Global(gid));
                    self.context_defs.insert(name.text.clone(), (gid, field_tys));
                    gid
                };
                let init = self.mk(ExprKind::Tuple(field_vals), rec_ty.clone(), span);
                out.push(Stmt::Assign(
                    Place {
                        kind: PlaceKind::Global(gid),
                        ty: rec_ty,
                        span,
                    },
                    AssignOp::Set,
                    init,
                ));
            }
            ast::StmtKind::With { ctx, args, body } => {
                let span = stmt.span;
                let held = (self.contexts.len(), self.unchecked);
                self.push_context(ctx, args);
                let saved_overflow = self.overflow_checks_off;
                let saved_sep = self.io_sep.clone();
                let saved_prec = self.io_precision;
                let saved_base = self.io_base;
                let mut saved_ctx_slot: Option<(GlobalId, Slot)> = None;

                match ctx.text.as_str() {
                    "checks" => {
                        for arg in args {
                            let Some(k) = &arg.name else { continue };
                            let v = &arg.value;
                            let is_off = matches!(&v.kind, ast::ExprKind::Name(n) if n == "off");
                            let is_on = matches!(&v.kind, ast::ExprKind::Name(n) if n == "on");
                            if k.text == "overflow" {
                                if is_off {
                                    self.overflow_checks_off += 1;
                                } else if is_on {
                                    self.overflow_checks_off = 0;
                                }
                            } else if is_off {
                                self.unchecked += 1;
                            } else if is_on {
                                self.unchecked = 0;
                            }
                        }
                    }
                    "lint" => {}
                    "io" => {
                        let gid = self.ensure_io_global();
                        let io_ty = self.globals[gid].ty.clone();
                        let save = self.temp(io_ty.clone());
                        out.push(Stmt::Let(
                            save,
                            Some(self.mk(ExprKind::Global(gid), io_ty.clone(), span)),
                        ));
                        saved_ctx_slot = Some((gid, save));
                        let Ty::Record(fields) = &io_ty else { unreachable!() };
                        let fields = fields.clone();
                        for arg in args {
                            let Some(k) = &arg.name else { continue };
                            let v = &arg.value;
                            if k.text == "sep" {
                                if let ast::ExprKind::Str(pieces) = &v.kind {
                                    let mut s = String::new();
                                    for p in pieces {
                                        if let ast::StrPiece::Text(t) = p {
                                            s.push_str(t);
                                        }
                                    }
                                    self.io_sep = Some(s);
                                }
                            } else if k.text == "precision" {
                                if let ast::ExprKind::Int(n) = &v.kind {
                                    self.io_precision = Some((*n).clamp(0, 18) as u32);
                                }
                            } else if k.text == "base" {
                                if let ast::ExprKind::Int(n) = &v.kind {
                                    self.io_base = Some(*n as i64);
                                }
                            }
                            if let Some((idx, (_, fty))) =
                                fields.iter().enumerate().find(|(_, (fnm, _))| fnm == &k.text)
                            {
                                let val = self.expect(v, fty);
                                out.push(Stmt::Assign(
                                    Place {
                                        kind: PlaceKind::Field(
                                            Box::new(Place {
                                                kind: PlaceKind::Global(gid),
                                                ty: io_ty.clone(),
                                                span,
                                            }),
                                            idx,
                                        ),
                                        ty: fty.clone(),
                                        span: v.span,
                                    },
                                    AssignOp::Set,
                                    val,
                                ));
                            }
                        }
                    }
                    other => {
                        if let Some((gid, fields)) = self.context_defs.get(other).cloned() {
                            let ctx_ty = self.globals[gid].ty.clone();
                            let save = self.temp(ctx_ty.clone());
                            out.push(Stmt::Let(
                                save,
                                Some(self.mk(ExprKind::Global(gid), ctx_ty.clone(), span)),
                            ));
                            saved_ctx_slot = Some((gid, save));
                            for arg in args {
                                let Some(k) = &arg.name else { continue };
                                let v = &arg.value;
                                if let Some((idx, (_, fty))) =
                                    fields.iter().enumerate().find(|(_, (fnm, _))| fnm == &k.text)
                                {
                                    let val = self.expect(v, fty);
                                    out.push(Stmt::Assign(
                                        Place {
                                            kind: PlaceKind::Field(
                                                Box::new(Place {
                                                    kind: PlaceKind::Global(gid),
                                                    ty: ctx_ty.clone(),
                                                    span,
                                                }),
                                                idx,
                                            ),
                                            ty: fty.clone(),
                                            span: v.span,
                                        },
                                        AssignOp::Set,
                                        val,
                                    ));
                                }
                            }
                        }
                    }
                }

                if let Some(body) = body {
                    out.extend(self.block(body, Tail::Discard).stmts);
                    self.contexts.truncate(held.0);
                    self.unchecked = held.1;
                    self.overflow_checks_off = saved_overflow;
                    self.io_sep = saved_sep;
                    self.io_precision = saved_prec;
                    self.io_base = saved_base;
                    if let Some((gid, save)) = saved_ctx_slot {
                        let ctx_ty = self.globals[gid].ty.clone();
                        out.push(Stmt::Assign(
                            Place {
                                kind: PlaceKind::Global(gid),
                                ty: ctx_ty,
                                span,
                            },
                            AssignOp::Set,
                            self.local(save, span),
                        ));
                    }
                }
            }
            ast::StmtKind::Error => {}
        }
    }

    /// Puts `with ctx(field: value, ..)` in force. `checks(overflow: off)`
    /// is an `unchecked:` for what it covers. (R09.4.2)
    pub(super) fn push_context(&mut self, ctx: &ast::Name, args: &[ast::Arg]) {
        for arg in args {
            let Some(field) = &arg.name else { continue };
            let off = matches!(&arg.value.kind, ast::ExprKind::Name(v) if v == "off");
            if ctx.text == "checks" && field.text == "overflow" && off {
                self.unchecked += 1;
            }
            let entry = (
                self.cur_mod,
                ctx.text.clone(),
                field.text.clone(),
                arg.value.clone(),
            );
            self.contexts.push(entry);
        }
    }

    /// The bare `with` lines that open the program's file: they hold for
    /// all of it, its fns included.
    pub(super) fn file_contexts(&mut self) {
        let leading: Vec<(ast::Name, Vec<ast::Arg>)> = self
            .mod_items(0)
            .map(|(_, stmt)| stmt)
            .map_while(|stmt| match &stmt.kind {
                ast::StmtKind::With {
                    ctx,
                    args,
                    body: None,
                } => Some((ctx.clone(), args.clone())),
                _ => None,
            })
            .collect();
        let held = self.cur_mod;
        self.cur_mod = 0;
        for (ctx, args) in &leading {
            self.push_context(ctx, args);
        }
        self.cur_mod = held;
    }

    /// The value `with ctx(field: ..)` gives where this code is written.
    pub(super) fn context(&self, ctx: &str, field: &str) -> Option<ast::Expr> {
        self.contexts
            .iter()
            .rev()
            .find(|(m, c, f, _)| *m == self.cur_mod && c == ctx && f == field)
            .map(|(_, _, _, value)| value.clone())
    }

    /// `with io(base: 1)`: counted domains read and print 1-based. (R10.2.2)
    pub(super) fn base_one(&self) -> bool {
        self.context("io", "base")
            .is_some_and(|v| matches!(v.kind, ast::ExprKind::Int(1)))
    }

    /// The loop heads and cell value of `for (r, c) in Square` over a
    /// product domain whose factors the pattern names one by one.
    fn product_heads(
        &mut self,
        pat: &ast::Pattern,
        iter: &ast::Expr,
    ) -> Option<(Vec<LoopHead>, Expr)> {
        let ast::PatternKind::Tuple(pats) = &pat.kind else {
            return None;
        };
        let factors: Vec<ProductFactor> = match (&iter.kind, self.sym(iter)) {
            (_, Some(Sym::Domain(id))) => self
                .product_domains
                .get(&id)?
                .iter()
                .map(|(_, f)| f.clone())
                .collect(),
            (ast::ExprKind::Tuple(items), _) => items
                .iter()
                .map(|it| self.expr_to_product_factor(it))
                .collect::<Option<_>>()?,
            _ => return None,
        };
        if factors.len() != pats.len() {
            return None;
        }
        let span = iter.span;
        let (mut heads, mut parts) = (Vec::new(), Vec::new());
        for f in &factors {
            let ty = match f {
                ProductFactor::Domain(d) => Ty::Pos(*d),
                _ => Ty::Int,
            };
            let slot = self.temp(ty.clone());
            let var = self.mk(ExprKind::Local(slot), ty, span);
            parts.push(match f {
                ProductFactor::Domain(_) => var,
                _ => self.product_factor_from_ord(f, var, span),
            });
            heads.push((
                slot,
                self.int_lit(0, span),
                self.product_factor_len(f, span),
            ));
        }
        let ty = Ty::Tuple(parts.iter().map(|p| p.ty.clone()).collect());
        Some((heads, self.mk(ExprKind::Tuple(parts), ty, span)))
    }

    /// Runs `body` with `pat` bound to `value` inside one loop per head,
    /// outermost first. `break` leaves them all; `continue` steps the
    /// innermost, so together they act as the one loop the user wrote.
    #[allow(clippy::too_many_arguments)]
    fn nested_for(
        &mut self,
        label: Option<&ast::Name>,
        heads: Vec<LoopHead>,
        pat: &ast::Pattern,
        value: Expr,
        body: &ast::Block,
        else_body: Option<&ast::Block>,
        out: &mut Vec<Stmt>,
    ) {
        let mut ids: Vec<LoopId> = heads[1..].iter().map(|_| self.fresh_loop()).collect();
        let inner = self.open_loop(label.map(|l| l.text.clone()));
        let break_id = ids.first().copied().unwrap_or(inner);
        if let Some(&outer) = ids.first() {
            self.cx.break_out.insert(inner, outer);
        }
        let else_flag = self.begin_loop_else(break_id, else_body, out);
        self.cx.scopes.push(HashMap::new());
        self.cx.local_syms.push(HashMap::new());
        let mut stmts = Vec::new();
        match (&pat.kind, value.kind) {
            (ast::PatternKind::Tuple(pats), ExprKind::Tuple(parts))
                if pats.len() == parts.len() =>
            {
                for (p, part) in pats.iter().zip(parts) {
                    match (&p.kind, &part.kind) {
                        (ast::PatternKind::Name(name), ExprKind::Local(slot))
                            if heads.iter().any(|h| h.0 == *slot) =>
                        {
                            self.cx.locals[*slot].name = name.clone();
                            self.cx
                                .scopes
                                .last_mut()
                                .expect("a scope")
                                .insert(name.clone(), *slot);
                        }
                        _ => self.bind(p, part, &mut stmts),
                    }
                }
            }
            (_, kind) => {
                let value = Expr { kind, ..value };
                self.bind(pat, value, &mut stmts)
            }
        }
        stmts.extend(self.block(body, Tail::Discard).stmts);
        self.cx.local_syms.pop();
        self.cx.scopes.pop();
        self.cx.loops.pop();
        self.cx.break_out.remove(&inner);
        ids.push(inner);
        for ((slot, lo, hi), id) in heads.into_iter().zip(ids).rev() {
            let iter = Iter::Range {
                lo,
                hi,
                step: None,
                rev: false,
                closed: false,
            };
            stmts = vec![Stmt::For(id, slot, iter, Block { stmts })];
        }
        out.extend(stmts);
        self.end_loop_else(break_id, else_flag, else_body, out);
    }

    fn begin_loop_else(
        &mut self,
        loop_id: LoopId,
        else_body: Option<&ast::Block>,
        out: &mut Vec<Stmt>,
    ) -> Option<Slot> {
        let eb = else_body?;
        let flag = self.temp(Ty::Bool);
        let init = self.mk(ExprKind::Bool(true), Ty::Bool, eb.span);
        out.push(Stmt::Let(flag, Some(init)));
        self.cx.loop_else.insert(loop_id, flag);
        Some(flag)
    }

    fn end_loop_else(
        &mut self,
        loop_id: LoopId,
        flag: Option<Slot>,
        else_body: Option<&ast::Block>,
        out: &mut Vec<Stmt>,
    ) {
        let (Some(flag), Some(eb)) = (flag, else_body) else {
            return;
        };
        self.cx.loop_else.remove(&loop_id);
        let cond = self.local(flag, eb.span);
        let then = self.block(eb, Tail::Discard);
        out.push(Stmt::If(cond, then, Block::default()));
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

    /// `domain Node = In + Out`: the domains of a disjoint union, in order.
    fn domain_sum_parts(&mut self, def: &ast::Expr) -> Option<Vec<DomainId>> {
        let ast::ExprKind::Binary {
            op: ast::BinOp::Add,
            lhs,
            rhs,
            ..
        } = &def.kind
        else {
            return None;
        };
        let mut parts = self.domain_sum_part(lhs)?;
        parts.extend(self.domain_sum_part(rhs)?);
        Some(parts)
    }

    fn domain_sum_part(&mut self, e: &ast::Expr) -> Option<Vec<DomainId>> {
        match &e.kind {
            ast::ExprKind::Name(name) => self.resolve_domain_name(name).map(|d| vec![d]),
            _ => self.domain_sum_parts(e),
        }
    }

    pub(super) fn init_domain_id(&mut self, id: DomainId, def: &ast::Expr, out: &mut Vec<Stmt>) {
        // `domain E = [m]` is parsed as `0..m`; the brackets mark it counted.
        if self.source(def.span).trim_start().starts_with('[') {
            self.counted.insert(id);
            self.counted_domains.insert(id);
        }
        if let ast::ExprKind::List(items) = &def.kind {
            if items.len() == 1 {
                let val = self.expr(&items[0], Some(&Ty::Int));
                let len = self.coerce(val, &Ty::Int, def.span);
                let lo = self.mk(ExprKind::Int(0), Ty::Int, def.span);
                self.counted.insert(id);
                self.counted_domains.insert(id);
                out.push(Stmt::DefDomain { id, lo, len });
                return;
            }
        }
        if let Some(factors) = self.resolve_product_factors(def) {
            self.product_domains.insert(id, factors.clone());
            let mut len = self.product_factor_len(&factors[0].1, def.span);
            for (_, f) in &factors[1..] {
                let flen = self.product_factor_len(f, def.span);
                len = self.mk(
                    ExprKind::Binary(BinOp::Mul, Box::new(len), Box::new(flen)),
                    Ty::Int,
                    def.span,
                );
            }
            let lo = self.int_lit(0, def.span);
            out.push(Stmt::DefDomain { id, lo, len });
            return;
        }
        if let Some(parts) = self.domain_sum_parts(def) {
            let mut lens = parts
                .into_iter()
                .map(|d| self.mk(ExprKind::DomainLen(d), Ty::Int, def.span));
            let first = lens.next().unwrap();
            let len = lens.fold(first, |acc, l| {
                self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(acc), Box::new(l)),
                    Ty::Int,
                    def.span,
                )
            });
            let lo = self.int_lit(0, def.span);
            out.push(Stmt::DefDomain { id, lo, len });
            return;
        }
        if let Some(base_id) = self.domain_pairs_iter(def) {
            let n = self.mk(ExprKind::DomainLen(base_id), Ty::Int, def.span);
            let n_minus_1 = self.int_add(n.clone(), -1);
            let prod = self.mk(
                ExprKind::Binary(BinOp::Mul, Box::new(n), Box::new(n_minus_1)),
                Ty::Int,
                def.span,
            );
            let two = self.int_lit(2, def.span);
            let len = self.mk(
                ExprKind::Binary(BinOp::Div, Box::new(prod), Box::new(two)),
                Ty::Int,
                def.span,
            );
            let lo = self.int_lit(0, def.span);
            out.push(Stmt::DefDomain { id, lo, len });
            return;
        }
        let ast::ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
            step: None,
        } = &def.kind
        else {
            // `domain V = expr`: supports counts (`Int`), existing domains (`DomainVal`),
            // existential-domain tuples (`(domain C, ...)`), sequences (`List`/`Array`/`Str`),
            // and domain-owning structs (e.g. `Compressed<D, T>` from `compress(xs)`).
            let val = self.expr(def, Some(&Ty::Int));
            match &val.ty {
                Ty::Int => {
                    self.counted.insert(id);
                    let len = self.coerce(val, &Ty::Int, def.span);
                    let lo = self.mk(ExprKind::Int(0), Ty::Int, def.span);
                    self.counted_domains.insert(id);
                    out.push(Stmt::DefDomain { id, lo, len });
                }
                Ty::DomainVal => {
                    out.push(Stmt::DefDomainFromVal { id, val });
                }
                Ty::List(_) | Ty::ListView(_) | Ty::Array { .. } | Ty::Str | Ty::StrView => {
                    let len = self.mk(
                        ExprKind::Builtin(Builtin::Len, vec![val]),
                        Ty::Int,
                        def.span,
                    );
                    let lo = self.mk(ExprKind::Int(0), Ty::Int, def.span);
                    out.push(Stmt::DefDomain { id, lo, len });
                }
                Ty::Tuple(fields) if fields.first() == Some(&Ty::DomainVal) => {
                    let dom_field =
                        self.mk(ExprKind::Field(Box::new(val), 0), Ty::DomainVal, def.span);
                    out.push(Stmt::DefDomainFromVal { id, val: dom_field });
                }
                Ty::Record(fields) if fields.first().is_some_and(|(_, t)| *t == Ty::DomainVal) => {
                    let dom_field =
                        self.mk(ExprKind::Field(Box::new(val), 0), Ty::DomainVal, def.span);
                    out.push(Stmt::DefDomainFromVal { id, val: dom_field });
                }
                Ty::Struct(sid) => {
                    let sid = *sid;
                    // Check if the struct has an array/list/str field (e.g. `Compressed.vals`).
                    let len_expr = self.structs[sid]
                        .def
                        .fields
                        .iter()
                        .enumerate()
                        .find(|(_, (_, t))| matches!(t, Ty::Array { .. } | Ty::List(_) | Ty::Str))
                        .map(|(idx, (_, fty))| {
                            let fty = fty.clone();
                            let field = self.mk(ExprKind::Field(Box::new(val), idx), fty, def.span);
                            self.mk(
                                ExprKind::Builtin(Builtin::Len, vec![field]),
                                Ty::Int,
                                def.span,
                            )
                        });
                    if let Some(len) = len_expr {
                        let lo = self.mk(ExprKind::Int(0), Ty::Int, def.span);
                        out.push(Stmt::DefDomain { id, lo, len });
                    } else {
                        self.unsupported(
                            def.span,
                            "a domain expression that does not produce a range, count, or index space",
                        );
                    }
                }
                Ty::Error => {}
                _ => {
                    self.unsupported(
                        def.span,
                        "a domain expression that does not produce a range, count, or index space",
                    );
                }
            }
            return;
        };
        if !*inclusive
            && matches!(lo.kind, ast::ExprKind::Int(0))
            && lo.span.start == lo.span.end
        {
            self.counted_domains.insert(id);
        }
        if let (Some(l), Some(h)) = (self.ast_bounds(lo), self.ast_bounds(hi)) {
            let extra = i64::from(*inclusive);
            let least = h.0.checked_add(extra).and_then(|h| h.checked_sub(l.1));
            let most = h.1.checked_add(extra).and_then(|h| h.checked_sub(l.0));
            if let (Some(least), Some(most)) = (least, most) {
                self.domain_lens.insert(id, (least.max(0), most.max(0)));
            }
        }
        let lo_e = self.expr(lo, None);
        let lo = if lo_e.ty == Ty::Char {
            self.mk(
                ExprKind::Cast(Cast::CharToInt, Box::new(lo_e)),
                Ty::Int,
                lo.span,
            )
        } else {
            self.coerce(lo_e, &Ty::Int, lo.span)
        };
        let hi_e = self.expr(hi, None);
        let hi = if hi_e.ty == Ty::Char {
            self.mk(
                ExprKind::Cast(Cast::CharToInt, Box::new(hi_e)),
                Ty::Int,
                hi.span,
            )
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
                    ast::StmtKind::Return(Some(e)) if matches!(&e.kind, ast::ExprKind::Name(n) if n == var_name) =>
                    {
                        return true;
                    }
                    ast::StmtKind::Expr(e) => {
                        if let ast::ExprKind::If {
                            branches,
                            otherwise,
                        } = &e.kind
                        {
                            if branches
                                .iter()
                                .any(|(_, b)| returns_var(var_name, &b.stmts))
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

    fn peek_fn_arg_types(
        &mut self,
        fn_name: &str,
        arity: usize,
        rest: &[ast::Stmt],
    ) -> Option<Vec<Ty>> {
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
                ast::ExprKind::List(items) | ast::ExprKind::Tuple(items) => items
                    .iter()
                    .find_map(|x| find_call_in_expr(fn_name, arity, x)),
                ast::ExprKind::If {
                    branches,
                    otherwise,
                } => {
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
                    ast::StmtKind::While {
                        cond,
                        body,
                        else_body,
                    } => {
                        if let ast::Cond::Expr(e) = cond {
                            if let Some(args) = find_call_in_expr(fn_name, arity, e) {
                                return Some(args);
                            }
                        }
                        if let Some(args) = find_call(fn_name, arity, &body.stmts) {
                            return Some(args);
                        }
                        if let Some(eb) = else_body {
                            if let Some(args) = find_call(fn_name, arity, &eb.stmts) {
                                return Some(args);
                            }
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
        fn find_passed_in_expr<'a>(fn_name: &str, e: &'a ast::Expr) -> Option<(&'a str, usize)> {
            match &e.kind {
                ast::ExprKind::Call { callee, args } => {
                    if let ast::ExprKind::Name(cname) = &callee.kind {
                        for (idx, a) in args.iter().enumerate() {
                            if matches!(&a.value.kind, ast::ExprKind::Name(n) if n == fn_name) {
                                return Some((cname.as_str(), idx));
                            }
                            if let ast::ExprKind::Lambda { params, body, .. } = &a.value.kind {
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
                    .or_else(|| {
                        rest.iter()
                            .find_map(|(_, r)| find_passed_in_expr(fn_name, r))
                    }),
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
                    ast::StmtKind::While {
                        cond,
                        body,
                        else_body,
                    } => {
                        if let ast::Cond::Expr(e) = cond {
                            if let Some(found) = find_passed_in_expr(fn_name, e) {
                                return Some(found);
                            }
                        }
                        if let Some(found) = find_passed(fn_name, &body.stmts) {
                            return Some(found);
                        }
                        if let Some(eb) = else_body {
                            if let Some(found) = find_passed(fn_name, &eb.stmts) {
                                return Some(found);
                            }
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
                    kind:
                        ast::ExprKind::Lambda {
                            params,
                            param_tys: ann_tys,
                            param_mut: _,
                            body,
                            ..
                        },
                    span: lspan,
                }),
            ) = (&pat.kind, init)
            {
                if let Some(param_tys) = self.peek_fn_arg_types(vname, params.len(), rest) {
                    let hint = Ty::Fn(param_tys, Box::new(Ty::Error));
                    return Some(self.lower_lambda(params, ann_tys, body, Some(&hint), *lspan));
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
    pub(super) fn let_value(
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
            let z = self.zero(&spec, span);
            if let Some(RangeInfo::Scalar(bounds)) = RangeInfo::from_spec(&spec) {
                return Some(self.check_range(z, &bounds, span));
            }
            return Some(z);
        };
        if is_read_call(init) {
            let mut spec = spec;
            if let ast::ExprKind::Call { args, .. } = &init.kind {
                if let Some(absent) = args
                    .iter()
                    .find(|a| a.name.as_ref().is_some_and(|n| n.text == "none"))
                    .map(|a| self.expect(&a.value, &Ty::Int))
                {
                    if !super::call::reads_none_as(&mut spec, &absent) {
                        let message = format!(
                            "`none:` is for reading a `?T`, and a `{}` has none",
                            self.show(&want)
                        );
                        self.error(codes::UNKNOWN_ARG_NAME, init.span, message);
                    }
                }
            }
            return Some(self.read(spec, init.span));
        }
        if let Some(none_arg) = read_none_arg(init) {
            let mut spec = spec;
            let absent = self.expect(none_arg, &Ty::Int);
            if !super::call::reads_none_as(&mut spec, &absent) {
                let message = format!(
                    "`none:` is for reading a `?T`, and a `{}` has none",
                    self.show(&want)
                );
                self.error(codes::UNKNOWN_ARG_NAME, init.span, message);
            }
            return Some(self.read(spec, init.span));
        }
        // `let b: [0..n]Int = read_output()` in a `@checker`: the same
        // layout, read from what the solution printed.
        if is_read_output_call(init) && self.aside == Some(Aside::Checker) {
            return Some(self.read_printed(spec, init.span));
        }
        if let Spec::Array { dims, elem } = &spec {
            let elem_ty = elem.ty();
            let elem_range = RangeInfo::from_spec(elem).and_then(|r| match r {
                RangeInfo::Scalar(b) => Some(b),
                _ => None,
            });
            if is_literal(init) {
                return Some(self.fill_literal(&spec, init));
            }
            if let Some(table) = self.comprehension_table(init, dims, &elem_ty, elem_range.as_ref())
            {
                return Some(table);
            }
            if let ast::ExprKind::Table {
                domains,
                params,
                body,
            } = &init.kind
            {
                if let Some(bounds) = elem_range.as_ref() {
                    if let Some(tdims) = domains
                        .iter()
                        .map(|d| self.dim(d, true))
                        .collect::<Option<Vec<_>>>()
                    {
                        return Some(self.build_table(
                            tdims,
                            params,
                            body,
                            Some(&elem_ty),
                            Some(bounds),
                            init.span,
                        ));
                    }
                }
            }
            // `var ways: [0..n]List<Z> = [Z(1)]`: where a cell is a list, a
            // list of what it holds fills every cell with a copy.
            if let (Ty::List(_), ast::ExprKind::List(items)) = (&elem_ty, &init.kind) {
                let holds = items.first().and_then(|item| self.probe_ty_silent(item));
                if holds.is_some_and(|ty| !matches!(ty, Ty::List(_))) {
                    let value = self.expect(init, &elem_ty);
                    return Some(self.fill_with_index_domains(
                        dims.clone(),
                        Some(Box::new(value)),
                        want,
                        init.span,
                    ));
                }
            }
            // `var dp: [Bool]Z = [1, 0]`: a list of as many values as the
            // array has cells, when that count is written as a number.
            // `let c: [1..=3]Int = [7, 8, 9]` and `[2..n]Int = [5, 6]` too:
            // a count known only as the program runs is checked then.
            // `let g: [0..2, 0..3]Int = [[1, 2, 3], [4, 5, 6]]`: a list of
            // rows, each as long as the next index says.
            let lens: Option<Vec<&Expr>> = dims
                .iter()
                .map(|dim| match dim {
                    DimSpec::Int(len) | DimSpec::IntFrom(_, len) => Some(&**len),
                    _ => None,
                })
                .collect();
            if let (ast::ExprKind::List(items), Some(lens)) = (&init.kind, lens) {
                let holds_cells = !items.is_empty()
                    && (lens.len() > 1
                        || !(matches!(elem_ty, Ty::List(_))
                            && items
                                .first()
                                .and_then(|item| self.probe_ty_silent(item))
                                .is_some_and(|ty| !matches!(ty, Ty::List(_)))));
                let shape = if holds_cells {
                    literal_shape(init, lens.len())
                } else {
                    None
                };
                if let (true, Some(Err(at))) = (lens.len() > 1, &shape) {
                    self.error(
                        codes::TYPE_MISMATCH,
                        *at,
                        format!("a list for a `{}` is rows of one length, nested as deep as the array has indices", self.show(&want)),
                    );
                    return Some(self.err_expr(init.span));
                }
                if let Some(Ok((counts, cells))) = shape {
                    for (len, &count) in lens.iter().zip(&counts) {
                        let Some(cells) = written(len) else { continue };
                        if cells != count as i64 {
                            let message = if lens.len() == 1 {
                                format!(
                                    "this array has {}, but the list gives {}",
                                    plural(cells, "cell"),
                                    plural(count as i64, "value"),
                                )
                            } else {
                                format!(
                                    "this index has {}, but the list gives {}",
                                    plural(cells, "cell"),
                                    plural(count as i64, "value")
                                )
                            };
                            self.error(codes::TYPE_MISMATCH, init.span, message);
                            return Some(self.err_expr(init.span));
                        }
                    }
                    let unknown: Vec<(Expr, usize)> = lens
                        .iter()
                        .zip(&counts)
                        .filter(|(len, _)| written(len).is_none())
                        .map(|(len, &count)| ((*len).clone(), count))
                        .collect();
                    let dims = dims.clone();
                    let at = init.span;
                    return Some(self.do_expr(Some(&want.clone()), at, |lw, out| {
                        for (len, count) in unknown {
                            let count = lw.int_lit(count as i64, at);
                            let wrong = lw.mk(
                                ExprKind::Cmp(CmpOp::Ne, Box::new(len), Box::new(count)),
                                Ty::Bool,
                                at,
                            );
                            let trap = Stmt::Trap(
                                "an array literal needs one value per position".to_string(),
                                lw.span(at),
                            );
                            out.push(Stmt::If(
                                wrong,
                                Block { stmts: vec![trap] },
                                Block::default(),
                            ));
                        }
                        let values: Vec<Expr> = cells
                            .iter()
                            .map(|item| {
                                let v = lw.expect(item, &elem_ty);
                                match elem_range.as_ref() {
                                    Some(b) => lw.check_range(v, b, item.span),
                                    None => v,
                                }
                            })
                            .collect();
                        let mut values = values.into_iter();
                        let first = values.next().map(Box::new);
                        let held = lw.temp(want.clone());
                        let filled = lw.mk(ExprKind::Fill(dims, first), want.clone(), at);
                        out.push(Stmt::Let(held, Some(filled)));
                        for (i, value) in values.enumerate() {
                            // The cell's place along each index, last index fastest.
                            let mut rest = i + 1;
                            let mut index = Vec::new();
                            for &count in counts.iter().rev() {
                                index.push(lw.int_lit((rest % count) as i64, at));
                                rest /= count;
                            }
                            index.reverse();
                            let whole = Place {
                                kind: PlaceKind::Local(held),
                                ty: want.clone(),
                                span: at,
                            };
                            let cell = Place {
                                kind: PlaceKind::Index(Box::new(whole), index),
                                ty: elem_ty.clone(),
                                span: at,
                            };
                            out.push(Stmt::Assign(cell, AssignOp::Set, value));
                        }
                        out.push(Stmt::Give(lw.local(held, at)));
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
                return Some(self.fill_with_index_domains(
                    dims.clone(),
                    Some(Box::new(value)),
                    want,
                    init.span,
                ));
            }
            let value = self.expr(init, Some(&want));
            if value.ty == want || value.ty == Ty::Error {
                return Some(value);
            }
            if same_cells(&want, &value.ty) {
                return Some(Expr { ty: want, ..value });
            }
            return Some(match self.try_coerce(value, &elem_ty) {
                Ok(value) => {
                    let value = match elem_range.as_ref() {
                        Some(b) => self.check_range(value, b, init.span),
                        None => value,
                    };
                    self.fill_with_index_domains(
                        dims.clone(),
                        Some(Box::new(value)),
                        want,
                        init.span,
                    )
                }
                Err(value) => {
                    if matches!(elem.as_ref(), Spec::Array { .. })
                        && !matches!(value.ty, Ty::Array { .. } | Ty::List(_))
                    {
                        return Some(self.fill_literal(&spec, init));
                    }
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
        let val = self.expect(init, &want);
        if let Some(RangeInfo::Scalar(bounds)) = RangeInfo::from_spec(&spec) {
            return Some(self.check_range(val, &bounds, init.span));
        }
        Some(val)
    }

    /// `var g: [0..n][0..4]Int = 0`: a literal fills every cell, and a
    /// cell that is itself an array is filled the same way.
    pub(super) fn fill_literal(&mut self, spec: &Spec, init: &ast::Expr) -> Expr {
        let Spec::Array { dims, elem } = spec else {
            let v = self.expect(init, &spec.ty());
            if let Some(RangeInfo::Scalar(bounds)) = RangeInfo::from_spec(spec) {
                return self.check_range(v, &bounds, init.span);
            }
            return v;
        };
        let value = self.fill_literal(elem, init);
        self.fill_with_index_domains(dims.clone(), Some(Box::new(value)), spec.ty(), init.span)
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

    /// The values of `fields(a, b, ..)` in a `for` head (decision 28.1).
    pub(super) fn reflected_values<'e>(&self, iter: &'e ast::Expr) -> Option<&'e [ast::Arg]> {
        let ast::ExprKind::Call { callee, args } = &iter.kind else {
            return None;
        };
        let ast::ExprKind::Name(name) = &callee.kind else {
            return None;
        };
        (name == "fields"
            && self.lookup(name).is_none()
            && self.get_sym(name).is_none()
            && !args.is_empty()
            && args.iter().all(|a| a.name.is_none()))
        .then_some(args.as_slice())
    }

    /// `for f in fields(x):`, unrolled: the body once per field of `x`, in
    /// declaration order, with `f` that field. With several values, `f` is
    /// the tuple of their fields at that position.
    fn fields_loop(
        &mut self,
        pat: &ast::Pattern,
        values: &[ast::Arg],
        body: &ast::Block,
        out: &mut Vec<Stmt>,
    ) {
        let mut temps = Vec::new();
        for a in values {
            let v = self.expr(&a.value, None);
            let ty = v.ty.clone();
            let slot = self.temp(ty.clone());
            out.push(Stmt::Let(slot, Some(v)));
            temps.push((slot, ty, a.value.span));
        }
        if let Ty::Enum(eid) = temps[0].1 {
            if let Some((_, ty, span)) = temps.iter().find(|(_, ty, _)| *ty != temps[0].1) {
                let message = format!(
                    "`fields` walks values of one type together; this is a `{}`",
                    self.show(ty)
                );
                self.error(codes::TYPE_MISMATCH, *span, message);
                return;
            }
            // Each value is the same variant, or the body doesn't run.
            let variants = self.enums[eid].variants.clone();
            for (tag, (_, payload)) in variants.iter().enumerate() {
                let mut stmts = Vec::new();
                for (i, fty) in payload.iter().enumerate() {
                    let mut items: Vec<Expr> = temps
                        .iter()
                        .map(|(slot, _, span)| {
                            let base = self.local(*slot, *span);
                            self.mk(
                                ExprKind::Payload(Box::new(base), tag, i),
                                fty.clone(),
                                *span,
                            )
                        })
                        .collect();
                    let value = if items.len() == 1 {
                        items.pop().unwrap()
                    } else {
                        let ty = Ty::Tuple(vec![fty.clone(); items.len()]);
                        self.mk(ExprKind::Tuple(items), ty, temps[0].2)
                    };
                    self.cx.scopes.push(HashMap::new());
                    self.cx.local_syms.push(HashMap::new());
                    self.bind(pat, value, &mut stmts);
                    stmts.extend(self.block(body, Tail::Discard).stmts);
                    self.cx.local_syms.pop();
                    self.cx.scopes.pop();
                }
                if stmts.is_empty() {
                    continue;
                }
                for (slot, _, span) in temps.iter().rev() {
                    let base = self.local(*slot, *span);
                    let cond = self.mk(ExprKind::IsVariant(Box::new(base), tag), Ty::Bool, *span);
                    stmts = vec![Stmt::If(cond, Block { stmts }, Block { stmts: Vec::new() })];
                }
                out.extend(stmts);
            }
            return;
        }
        let field_tys: Vec<Ty> = match &temps[0].1 {
            Ty::Struct(sid) => self.structs[*sid]
                .def
                .fields
                .iter()
                .map(|(_, t)| t.clone())
                .collect(),
            Ty::Tuple(items) => items.clone(),
            Ty::Record(fields) => fields.iter().map(|(_, t)| t.clone()).collect(),
            Ty::Unit => Vec::new(),
            Ty::Error => return,
            ty => {
                let message = format!(
                    "`fields` walks the fields of a struct or a tuple, not of a `{}`",
                    self.show(ty)
                );
                self.error(codes::TYPE_MISMATCH, temps[0].2, message);
                return;
            }
        };
        if let Some((_, ty, span)) = temps.iter().find(|(_, ty, _)| *ty != temps[0].1) {
            let message = format!(
                "`fields` walks values of one type together; this is a `{}`",
                self.show(ty)
            );
            self.error(codes::TYPE_MISMATCH, *span, message);
            return;
        }
        if field_tys.is_empty() {
            let at = out.len() - temps.len();
            let lets: Vec<Stmt> = out.drain(at..).collect();
            for stmt in lets {
                if let Stmt::Let(_, Some(v)) = stmt {
                    let pure = match &v.kind {
                        ExprKind::Unit | ExprKind::Local(_) => true,
                        ExprKind::Tuple(items) => items.is_empty(),
                        _ => false,
                    };
                    if !pure {
                        out.push(Stmt::Expr(v));
                    }
                }
            }
        }
        for (i, fty) in field_tys.iter().enumerate() {
            let mut items: Vec<Expr> = temps
                .iter()
                .map(|(slot, _, span)| {
                    let base = self.local(*slot, *span);
                    self.mk(ExprKind::Field(Box::new(base), i), fty.clone(), *span)
                })
                .collect();
            let value = if items.len() == 1 {
                items.pop().unwrap()
            } else {
                let ty = Ty::Tuple(vec![fty.clone(); items.len()]);
                let span = temps[0].2;
                self.mk(ExprKind::Tuple(items), ty, span)
            };
            self.cx.scopes.push(HashMap::new());
            self.cx.local_syms.push(HashMap::new());
            self.bind(pat, value, out);
            out.extend(self.block(body, Tail::Discard).stmts);
            self.cx.local_syms.pop();
            self.cx.scopes.pop();
        }
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
            ast::PatternKind::Domain(name) => {
                let dname = &name.text;
                let dom_id = if self.at_top_level() {
                    if let Some(Sym::Domain(existing)) =
                        self.mod_syms[self.cur_mod].get(dname).copied()
                    {
                        existing
                    } else {
                        let id = self.domains.len();
                        self.domains.push(format!("{dname}_{id}"));
                        self.mod_syms[self.cur_mod].insert(dname.clone(), Sym::Domain(id));
                        id
                    }
                } else {
                    let id = self.domains.len();
                    self.domains.push(format!("{dname}_{id}"));
                    self.cx
                        .local_syms
                        .last_mut()
                        .unwrap()
                        .insert(dname.clone(), Sym::Domain(id));
                    id
                };
                out.push(Stmt::DefDomainFromVal {
                    id: dom_id,
                    val: value,
                });
            }
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
                if let Ty::Pos(dom_id) = value.ty {
                    if let Some(factors) = self.product_domains.get(&dom_id).cloned() {
                        if factors.len() != pats.len() {
                            let message = format!(
                                "this pattern has {} names, but `{}` has {} factors",
                                pats.len(),
                                self.show(&value.ty),
                                factors.len()
                            );
                            self.error(codes::TYPE_MISMATCH, pat.span, message);
                            return;
                        }
                        let parts = self.decode_product(dom_id, value, pat.span, out);
                        for (p, part) in pats.iter().zip(parts) {
                            self.bind_mut(p, part, mutable, out);
                        }
                        return;
                    }
                }
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
                // Identify the callee's existential domain id(s) if any field is `Ty::DomainVal`.
                let callee_doms: Vec<DomainId> = match &value.kind {
                    ExprKind::Call(fn_id, _) => self.sigs[*fn_id]
                        .new_domain_slots
                        .iter()
                        .map(|(_, d)| *d)
                        .collect(),
                    ExprKind::Tuple(items) => items
                        .iter()
                        .filter_map(|e| match e.kind {
                            ExprKind::DomainVal(d) => Some(d),
                            _ => None,
                        })
                        .collect(),
                    _ => Vec::new(),
                };
                let fallback_dom = fields.iter().find_map(find_pos_domain_in_ty);
                let mut remaps: Vec<(DomainId, DomainId)> = Vec::new();
                let mut dom_idx = 0usize;
                let base = self.hold(value, out);
                for (i, (p, ty)) in pats.iter().zip(fields).enumerate() {
                    let mut field_ty = ty.clone();
                    for &(from, to) in &remaps {
                        field_ty = remap_domain_ty(&field_ty, from, to);
                    }
                    let field = Expr {
                        kind: ExprKind::Field(Box::new(base.clone()), i),
                        ty: field_ty.clone(),
                        span: base.span,
                    };
                    if ty == Ty::DomainVal {
                        let dname_opt = match &p.kind {
                            ast::PatternKind::Domain(n) => Some(n.text.clone()),
                            ast::PatternKind::Name(n) => Some(n.clone()),
                            ast::PatternKind::Variant { path, args }
                                if args.is_empty() && path.len() == 1 =>
                            {
                                Some(path[0].text.clone())
                            }
                            _ => None,
                        };
                        if let Some(dname) = dname_opt {
                            // Reuse the pre-registered top-level domain slot when present so
                            // sibling top-level `fn` signatures refer to the same `DomainId`.
                            // Otherwise allocate a fresh runtime slot per binding site so
                            // multiple calls to the same existential-returning fn never alias.
                            let fresh_id = if self.at_top_level() {
                                if let Some(Sym::Domain(existing)) =
                                    self.mod_syms[self.cur_mod].get(&dname).copied()
                                {
                                    existing
                                } else {
                                    let id = self.domains.len();
                                    self.domains.push(format!("{dname}_{id}"));
                                    self.mod_syms[self.cur_mod]
                                        .insert(dname.clone(), Sym::Domain(id));
                                    id
                                }
                            } else {
                                let id = self.domains.len();
                                self.domains.push(format!("{dname}_{id}"));
                                self.cx
                                    .local_syms
                                    .last_mut()
                                    .unwrap()
                                    .insert(dname.clone(), Sym::Domain(id));
                                id
                            };
                            if let Some(orig) = callee_doms.get(dom_idx).copied().or(fallback_dom) {
                                remaps.push((orig, fresh_id));
                            }
                            dom_idx += 1;
                            out.push(Stmt::DefDomainFromVal {
                                id: fresh_id,
                                val: field,
                            });
                            continue;
                        }
                    }
                    self.bind_mut(p, field, mutable, out);
                }
            }
            ast::PatternKind::Var(inner) => self.bind_mut(inner, value, true, out),
            ast::PatternKind::Error => {}
            _ => self.unsupported(pat.span, "this pattern outside a `match`"),
        }
    }

    /// `value` as something cheap to mention twice: itself if it is a
    /// variable, else a temporary that holds it.
    pub(super) fn hold(&mut self, value: Expr, out: &mut Vec<Stmt>) -> Expr {
        if names_a_place(&value) {
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
        if self.grid_row(target).is_some() {
            // `a[l..r] -= b[l2..r2]`, `a[1..n] min= a[0..n - 1]`, `a[l..r] =
            // b[l2..r2]`: part against part, cell for cell.
            let cellwise = match op {
                ast::AssignOp::Set | ast::AssignOp::Max | ast::AssignOp::Min => true,
                ast::AssignOp::Binary(bin) => {
                    matches!(
                        bin,
                        BinOp::Add
                            | BinOp::Sub
                            | BinOp::Mul
                            | BinOp::BitAnd
                            | BinOp::BitOr
                            | BinOp::BitXor
                    )
                }
            };
            if cellwise && self.grid_row(value).is_some() {
                self.part_map(target, op, value, None, out);
                return;
            }
            if matches!(op, ast::AssignOp::Max | ast::AssignOp::Min) {
                self.grid_keep(target, Some(op == ast::AssignOp::Max), value, false, out);
                return;
            }
            // `a[l..r] = b[l2..r2] + x`: each cell worked out and stored.
            if let (
                ast::AssignOp::Set,
                ast::ExprKind::Binary {
                    op: bin, lhs, rhs, ..
                },
            ) = (op, &value.kind)
            {
                if matches!(
                    bin,
                    BinOp::Add
                        | BinOp::Sub
                        | BinOp::Mul
                        | BinOp::BitAnd
                        | BinOp::BitOr
                        | BinOp::BitXor
                ) {
                    if self.grid_row(lhs).is_some() && self.one_cell(rhs) {
                        self.part_map(target, op, lhs, Some((*bin, rhs, false)), out);
                        return;
                    }
                    if self.grid_row(rhs).is_some() && self.one_cell(lhs) {
                        self.part_map(target, op, rhs, Some((*bin, lhs, true)), out);
                        return;
                    }
                }
            }
            if self.one_cell(value) {
                self.grid_fill(target, op, value, out);
                return;
            }
            // `dp[k, ..] = row`: the row's cells, stored one by one.
            if op == ast::AssignOp::Set {
                self.grid_keep(target, None, value, false, out);
                return;
            }
        }
        if let (ast::ExprKind::Tuple(targets), ast::AssignOp::Set) = (&target.kind, op) {
            // `(a[d], b[d]) = read()`: the places say what is read.
            let wanted: Option<Vec<Ty>> = targets
                .iter()
                .map(|t| self.try_place(t).map(|place| place.ty))
                .collect();
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
                self.emit_assign(place, AssignOp::Set, field, t.span, out);
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
        let settled = out.len();
        let place = self.settle_indexes(place, out);
        // `a op= b` is `a = a op b` for every type that has `op`: a list,
        // a set, anything `a op b` is written for. The numbers, strings
        // and structs below have a shorter way that reads `a` once.
        if let ast::AssignOp::Binary(bin) = op {
            let short = bin == BinOp::Coalesce
                || matches!(
                    place.ty,
                    Ty::Int
                        | Ty::I128
                        | Ty::U64
                        | Ty::Float
                        | Ty::Bool
                        | Ty::Str
                        | Ty::Struct(_)
                        | Ty::Opt(_)
                        | Ty::Error
                );
            // `xs += ys` grows `xs` where it is; it is not a copy of both.
            if bin == BinOp::Add && matches!(place.ty, Ty::List(_)) {
                let more = self.expect(value, &place.ty);
                let extend = ExprKind::Mutate(Mutation::Extend, Box::new(place), vec![more]);
                out.push(Stmt::Expr(self.mk(extend, Ty::Unit, target.span)));
                return;
            }
            // An index that does work is done once, so `a` can't be
            // written out a second time.
            if !short && out.len() == settled {
                let whole = ast::Expr {
                    kind: ast::ExprKind::Binary {
                        op: bin,
                        op_span: target.span,
                        lhs: Box::new(target.clone()),
                        rhs: Box::new(value.clone()),
                    },
                    span: target.span,
                };
                // A step along a domain is checked: `cur += 2` stops the
                // program when it leaves the domain, as `(cur + 2)!` does.
                let rhs = if matches!(place.ty, Ty::Pos(_)) {
                    let stepped = self.expr(&whole, None);
                    if stepped.ty == Ty::opt(place.ty.clone()) {
                        let inside = ExprKind::Unwrap(Box::new(stepped), true);
                        self.mk(inside, place.ty.clone(), target.span)
                    } else {
                        self.coerce(stepped, &place.ty, target.span)
                    }
                } else {
                    self.expect(&whole, &place.ty)
                };
                out.push(Stmt::Assign(place, AssignOp::Set, rhs));
                return;
            }
        }
        let rhs = match op {
            ast::AssignOp::Set => self.expect(value, &place.ty),
            ast::AssignOp::Binary(BinOp::Coalesce) => match &place.ty {
                Ty::Opt(inner) => {
                    let inner = (**inner).clone();
                    // `(o ??= []).push(x)` goes on with what `o` holds.
                    self.filled
                        .insert((self.cur_mod, target.span.start, target.span.end));
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
                        let wrap = match &index.kind {
                            ExprKind::Cast(cast @ (Cast::Raw | Cast::Trust), _) => Some(*cast),
                            _ => None,
                        };
                        let slot = self.temp(index.ty.clone());
                        let ispan = index.span;
                        let ity = index.ty.clone();
                        let held = Expr {
                            kind: ExprKind::Local(slot),
                            ty: ity.clone(),
                            span: ispan,
                        };
                        out.push(Stmt::Let(slot, Some(index)));
                        match wrap {
                            Some(cast) => Expr {
                                kind: ExprKind::Cast(cast, Box::new(held)),
                                ty: ity,
                                span: ispan,
                            },
                            None => held,
                        }
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

    /// True for an array of one dimension, or a list, of values with an
    /// order: what `a max= b` goes through cell by cell.
    pub(super) fn is_row(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Array { dims, elem } if dims.len() == 1 => {
                !matches!(dims[0], Dim::Bound(_)) && self.ordered(elem)
            }
            Ty::List(elem) => self.ordered(elem),
            _ => false,
        }
    }

    /// `g[r, ..]`, `g[.., c]`, `g[r, lo..hi]`, or `a[..]` and `a[lo..hi]`
    /// of an array of one index: the array, its indexes and the axis the
    /// range runs along.
    pub(super) fn grid_row<'e>(
        &mut self,
        e: &'e ast::Expr,
    ) -> Option<(&'e ast::Expr, &'e [ast::Expr], usize)> {
        let ast::ExprKind::Index { base, indices } = &e.kind else {
            return None;
        };
        let ranges: Vec<usize> = (0..indices.len())
            .filter(|&k| super::expr::is_whole(&indices[k]))
            .collect();
        let [axis] = ranges.as_slice() else {
            return None;
        };
        match self.probe_ty_silent(base)? {
            Ty::Array { dims, .. } if dims.len() == indices.len() => Some((base, indices, *axis)),
            Ty::List(_) if indices.len() == 1 => Some((base, indices, *axis)),
            _ => None,
        }
    }

    /// The axes and the cell type of what [`Self::grid_row`] found: a list
    /// is one axis counted from 0.
    fn axes_of(ty: &Ty) -> Option<(Vec<Dim>, Ty)> {
        match ty {
            Ty::Array { dims, elem } => Some((dims.clone(), (**elem).clone())),
            Ty::List(elem) => Some((vec![Dim::Int], (**elem).clone())),
            _ => None,
        }
    }

    /// The indexes of the cell the loop is at, `at` standing where the
    /// range was written.
    fn axis_cell(&mut self, rows: &Ty, indices: &[ast::Expr], at: Slot, span: Span) -> Vec<Expr> {
        if matches!(rows, Ty::List(_)) {
            return vec![self.local(at, span)];
        }
        self.whole_axis = Some(at);
        let (cell, _) = self.index_args(rows, indices, span);
        self.whole_axis = None;
        cell
    }

    /// How many cells `rows` has along `axis`.
    fn axis_len(&mut self, rows: Expr, axis: usize, span: Span) -> Expr {
        let one = match &rows.ty {
            Ty::Array { dims, .. } => dims.len() == 1,
            _ => true,
        };
        if one {
            return self.len_of(rows, span);
        }
        let axis = self.int_lit(axis as i64, span);
        self.mk(
            ExprKind::Builtin(Builtin::AxisLen, vec![rows, axis]),
            Ty::Int,
            span,
        )
    }

    /// `g[r, ..] max= g[v, ..]`: every cell of one row or column of a grid
    /// takes the larger of the two, where it is, in one loop. The other
    /// side is a row of a grid too, or any row of the same cells. When
    /// `asked`, gives the slot that says whether any cell changed.
    pub(super) fn grid_keep(
        &mut self,
        target: &ast::Expr,
        max: Option<bool>,
        value: &ast::Expr,
        asked: bool,
        out: &mut Vec<Stmt>,
    ) -> Option<Slot> {
        let span = target.span;
        let word = match max {
            Some(true) => "max=",
            Some(false) => "min=",
            None => "=",
        };
        let (base, indices, axis) = self.grid_row(target)?;
        let place = self.place(base);
        let place = self.settle_indexes(place, out);
        if let PlaceKind::Local(s) = root_place(&place) {
            self.cx.mut_slots.insert(s);
        }
        let (dims, elem) = Self::axes_of(&place.ty)?;
        if max.is_some() && !self.ordered(&elem) {
            let message = format!(
                "`{word}` needs cells that have an order, and these are `{}`",
                self.show(&elem)
            );
            self.error(codes::TYPE_MISMATCH, span, message);
            return None;
        }
        let slot_ty = |dim: Dim| match dim {
            Dim::Pos(d) => Ty::Pos(d),
            Dim::Bound(d) => Ty::Bound(d),
            Dim::Enum(e) => Ty::Enum(e),
            Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
        };
        let changed = asked.then(|| {
            let changed = self.temp(Ty::Bool);
            out.push(Stmt::Let(
                changed,
                Some(self.mk(ExprKind::Bool(false), Ty::Bool, span)),
            ));
            changed
        });
        // The cell of the target the loop is at.
        let at = self.temp(slot_ty(dims[axis]));
        let cell = self.axis_cell(&place.ty, indices, at, span);
        let cell: Vec<Expr> = cell
            .into_iter()
            .enumerate()
            .map(|(k, index)| {
                if k == axis {
                    index
                } else {
                    self.hold(index, out)
                }
            })
            .collect();
        let grid = place_to_expr(&place);
        let len = self.axis_len(grid, axis, span);
        let (lo, hi) = self.axis_part(&indices[axis], dims[axis], len, out);
        let (lo, hi) = (self.hold(lo, out), self.hold(hi, out));
        let count = self.mk(
            ExprKind::Binary(BinOp::Sub, Box::new(hi.clone()), Box::new(lo.clone())),
            Ty::Int,
            span,
        );
        let cell = Place {
            kind: PlaceKind::Index(Box::new(place), cell),
            ty: elem.clone(),
            span: self.span(span),
        };
        // The cell of the other side that goes with it.
        let mut body = Vec::new();
        let counter = self.cast(Cast::Trust, self.local(at, span), Ty::Int);
        let offset = self.mk(
            ExprKind::Binary(BinOp::Sub, Box::new(counter), Box::new(lo.clone())),
            Ty::Int,
            span,
        );
        let (theirs, their_count) = match self.grid_row(value) {
            Some((other, their_indices, their_axis)) => {
                let other = self.expr(other, None);
                let other = self.hold(other, out);
                let Some((their_dims, their_elem)) = Self::axes_of(&other.ty) else {
                    return changed;
                };
                if their_elem != elem {
                    let message = format!(
                        "`{word}` goes cell by cell: these are `{}`, and the other side has `{}`",
                        self.show(&elem),
                        self.show(&their_elem)
                    );
                    self.error(codes::TYPE_MISMATCH, value.span, message);
                    return changed;
                }
                let their_at = self.temp(slot_ty(their_dims[their_axis]));
                let their_cell = self.axis_cell(&other.ty, their_indices, their_at, value.span);
                let their_cell: Vec<Expr> = their_cell
                    .into_iter()
                    .enumerate()
                    .map(|(k, index)| {
                        if k == their_axis {
                            index
                        } else {
                            self.hold(index, out)
                        }
                    })
                    .collect();
                let their_len = self.axis_len(other.clone(), their_axis, span);
                let (their_lo, their_hi) = self.axis_part(
                    &their_indices[their_axis],
                    their_dims[their_axis],
                    their_len,
                    out,
                );
                let (their_lo, their_hi) = (self.hold(their_lo, out), self.hold(their_hi, out));
                let step = self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(their_lo.clone()), Box::new(offset)),
                    Ty::Int,
                    span,
                );
                let step = self.cast(Cast::Trust, step, slot_ty(their_dims[their_axis]));
                body.push(Stmt::Let(their_at, Some(step)));
                let their_count = self.mk(
                    ExprKind::Binary(BinOp::Sub, Box::new(their_hi), Box::new(their_lo)),
                    Ty::Int,
                    span,
                );
                (
                    self.mk(
                        ExprKind::Index(Box::new(other), their_cell),
                        elem.clone(),
                        value.span,
                    ),
                    Some(their_count),
                )
            }
            // `a[l..r] max= x`: the one value against every cell.
            None if self.one_cell(value) => {
                let one = self.expect(value, &elem);
                (self.hold(one, out), None)
            }
            None => {
                let other = self.expr(value, Some(&Ty::list(elem.clone())));
                let dim = match &other.ty {
                    Ty::Array { dims, elem: theirs } if dims.len() == 1 && **theirs == elem => {
                        dims[0]
                    }
                    Ty::List(theirs) if **theirs == elem => Dim::Int,
                    Ty::Error => return changed,
                    _ => {
                        let message = format!(
                            "`{word}` goes cell by cell: this side is a row of `{}`, and the other is `{}`",
                            self.show(&elem),
                            self.show(&other.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, value.span, message);
                        return changed;
                    }
                };
                let other = self.hold(other, out);
                let their_count = self.len_of(other.clone(), span);
                let index = self.cast(Cast::Trust, offset, slot_ty(dim));
                (
                    self.mk(
                        ExprKind::Index(Box::new(other), vec![index]),
                        elem.clone(),
                        value.span,
                    ),
                    Some(their_count),
                )
            }
        };
        if let Some(their_count) = their_count {
            let differ = self.mk(
                ExprKind::Cmp(CmpOp::Ne, Box::new(their_count), Box::new(count)),
                Ty::Bool,
                span,
            );
            let trap = Stmt::Trap(
                format!("`a {word} b` needs two rows of one length"),
                self.span(span),
            );
            out.push(Stmt::If(
                differ,
                Block { stmts: vec![trap] },
                Block::default(),
            ));
        }
        let Some(max) = max else {
            body.push(Stmt::Assign(cell, AssignOp::Set, theirs));
            let iter = Iter::Range {
                lo,
                hi,
                step: None,
                rev: false,
                closed: false,
            };
            let lid = self.fresh_loop();
            out.push(Stmt::For(lid, at, iter, Block { stmts: body }));
            return changed;
        };
        let theirs = self.hold(theirs, &mut body);
        let better = self.mk(
            ExprKind::Cmp(
                if max { CmpOp::Gt } else { CmpOp::Lt },
                Box::new(theirs.clone()),
                Box::new(place_to_expr(&cell)),
            ),
            Ty::Bool,
            span,
        );
        let mut take = vec![Stmt::Assign(cell, AssignOp::Set, theirs)];
        if let Some(changed) = changed {
            let flag = Place {
                kind: PlaceKind::Local(changed),
                ty: Ty::Bool,
                span: self.span(span),
            };
            take.push(Stmt::Assign(
                flag,
                AssignOp::Set,
                self.mk(ExprKind::Bool(true), Ty::Bool, span),
            ));
        }
        body.push(Stmt::If(better, Block { stmts: take }, Block::default()));
        let iter = Iter::Range {
            lo,
            hi,
            step: None,
            rev: false,
            closed: false,
        };
        let lid = self.fresh_loop();
        out.push(Stmt::For(lid, at, iter, Block { stmts: body }));
        changed
    }

    /// `m.remove(k)` for its effect alone: the value that was there is
    /// not asked for, so it is not looked up. False when `e` is not that.
    fn drop_key(&mut self, e: &ast::Expr, out: &mut Vec<Stmt>) -> bool {
        let ast::ExprKind::Call { callee, args } = &e.kind else {
            return false;
        };
        let ast::ExprKind::Field { base, name } = &callee.kind else {
            return false;
        };
        let [ast::Arg {
            name: None,
            value: key,
            ..
        }] = args.as_slice()
        else {
            return false;
        };
        if name.text != "remove" {
            return false;
        }
        let Some(Ty::Map(key_ty, _) | Ty::HashMap(key_ty, _)) = self.probe_ty_silent(base) else {
            return false;
        };
        let place = self.place(base);
        let place = self.settle_indexes(place, out);
        if let PlaceKind::Local(s) = root_place(&place) {
            self.cx.mut_slots.insert(s);
        }
        let key = self.expect(key, &key_ty);
        let key = self.hold(key, out);
        let remove = self.mk(
            ExprKind::Mutate(Mutation::Remove, Box::new(place), vec![key]),
            Ty::Bool,
            e.span,
        );
        out.push(Stmt::Expr(remove));
        true
    }

    /// True when `value` is one cell's worth and not a row: `a[l..r] = 0`
    /// fills, `a[l..r] = b` copies. A value that has no type until it is
    /// told one (`none`) is one cell's; a list written out is a row.
    fn one_cell(&mut self, value: &ast::Expr) -> bool {
        match self.probe_ty_silent(value) {
            Some(ty) => !matches!(ty, Ty::List(_) | Ty::Array { .. }),
            None => !matches!(
                value.kind,
                ast::ExprKind::List(_) | ast::ExprKind::Comprehension { .. }
            ),
        }
    }

    /// `a[l..r] = b[l2..r2] + x` (and `x + b[..]`, with `-`, `*`, `&`, `|`,
    /// `^`): each cell of the part on the left takes the cell of the part
    /// on the right worked with the one value, in one loop and no copy.
    /// The two parts have one length. When they lie over each other in
    /// one array the loop runs from the end that reads every cell before
    /// it is stored to, so the result is that of working out the right
    /// side first.
    ///
    /// `a[l..r] op= b[l2..r2]` (`=`, `min=`, `max=`, `+=`, `-=`, `*=`, `&=`,
    /// `|=`, `^=`) is the same loop with no value: each cell of the left
    /// part is stored to, or worked, with the cell of the right part.
    fn part_map(
        &mut self,
        target: &ast::Expr,
        store: ast::AssignOp,
        source: &ast::Expr,
        with: Option<(BinOp, &ast::Expr, bool)>,
        out: &mut Vec<Stmt>,
    ) {
        let span = target.span;
        let (Some((base, indices, axis)), Some((from, from_indices, from_axis))) =
            (self.grid_row(target), self.grid_row(source))
        else {
            return;
        };
        let place = self.place(base);
        let place = self.settle_indexes(place, out);
        if let PlaceKind::Local(s) = root_place(&place) {
            self.cx.mut_slots.insert(s);
        }
        // The other array is read where it is when it has a name.
        let rows = match self.try_place(from) {
            Some(theirs) => {
                let theirs = self.settle_indexes(theirs, out);
                place_to_expr(&theirs)
            }
            None => {
                let theirs = self.expr(from, None);
                self.hold(theirs, out)
            }
        };
        let (Some((dims, elem)), Some((from_dims, from_elem))) =
            (Self::axes_of(&place.ty), Self::axes_of(&rows.ty))
        else {
            return;
        };
        let number = matches!(elem, Ty::Int | Ty::I128 | Ty::U64 | Ty::Float);
        let plain = with.is_none() && !matches!(store, ast::AssignOp::Binary(_));
        if elem != from_elem || !(number || plain) {
            if elem != Ty::Error && from_elem != Ty::Error {
                self.unsupported(span, "a part worked with one value, cell for cell, on cells that are not one kind of number");
            }
            return;
        }
        let index_ty = |dim: Dim| match dim {
            Dim::Pos(d) => Ty::Pos(d),
            Dim::Bound(d) => Ty::Bound(d),
            Dim::Enum(e) => Ty::Enum(e),
            Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
        };
        let (at, from_at) = (
            self.temp(index_ty(dims[axis])),
            self.temp(index_ty(from_dims[from_axis])),
        );
        let cell = self.axis_cell(&place.ty, indices, at, span);
        let cell: Vec<Expr> = cell
            .into_iter()
            .enumerate()
            .map(|(k, index)| {
                if k == axis {
                    index
                } else {
                    self.hold(index, out)
                }
            })
            .collect();
        let from_cell = self.axis_cell(&rows.ty, from_indices, from_at, span);
        let from_cell: Vec<Expr> = from_cell
            .into_iter()
            .enumerate()
            .map(|(k, index)| {
                if k == from_axis {
                    index
                } else {
                    self.hold(index, out)
                }
            })
            .collect();
        let len = self.axis_len(place_to_expr(&place), axis, span);
        let (lo, hi) = self.axis_part(&indices[axis], dims[axis], len, out);
        let from_len = self.axis_len(rows.clone(), from_axis, span);
        let (from_lo, from_hi) = self.axis_part(
            &from_indices[from_axis],
            from_dims[from_axis],
            from_len,
            out,
        );
        let sub = |lw: &Self, a: Expr, b: Expr| {
            lw.mk(
                ExprKind::Binary(BinOp::Sub, Box::new(a), Box::new(b)),
                Ty::Int,
                span,
            )
        };
        let count = sub(self, hi, lo.clone());
        let count = self.hold(count, out);
        let theirs = sub(self, from_hi, from_lo.clone());
        let differ = self.mk(
            ExprKind::Cmp(CmpOp::Ne, Box::new(theirs), Box::new(count.clone())),
            Ty::Bool,
            span,
        );
        let written = match (store, with) {
            (_, Some((bin, ..))) => format!("a[l..r] = b[l2..r2] {} x", bin.text()),
            (ast::AssignOp::Set, None) => "a[l..r] = b[l2..r2]".to_string(),
            (ast::AssignOp::Max, None) => "a[l..r] max= b[l2..r2]".to_string(),
            (ast::AssignOp::Min, None) => "a[l..r] min= b[l2..r2]".to_string(),
            (ast::AssignOp::Binary(bin), None) => format!("a[l..r] {}= b[l2..r2]", bin.text()),
        };
        let trap = Stmt::Trap(
            format!("`{written}` needs two parts of one length"),
            self.span(span),
        );
        out.push(Stmt::If(
            differ,
            Block { stmts: vec![trap] },
            Block::default(),
        ));
        let with = with.map(|(bin, scalar, first)| {
            let value = self.expect(scalar, &elem);
            (bin, self.hold(value, out), first)
        });
        let k = self.temp(Ty::Int);
        let step = |lw: &Self, slot: Slot, first: &Expr, dim: Dim| {
            let sum = lw.mk(
                ExprKind::Binary(
                    BinOp::Add,
                    Box::new(first.clone()),
                    Box::new(lw.local(k, span)),
                ),
                Ty::Int,
                span,
            );
            let sum = match index_ty(dim) {
                Ty::Int => sum,
                ty => lw.cast(Cast::Trust, sum, ty),
            };
            Stmt::Let(slot, Some(sum))
        };
        let read = self.mk(
            ExprKind::Index(Box::new(rows), from_cell),
            elem.clone(),
            span,
        );
        let worked = match with {
            Some((bin, value, first)) => {
                let (left, right) = if first { (value, read) } else { (read, value) };
                self.mk(
                    ExprKind::Binary(bin, Box::new(left), Box::new(right)),
                    elem.clone(),
                    span,
                )
            }
            None => read,
        };
        let cell = Place {
            kind: PlaceKind::Index(Box::new(place), cell),
            ty: elem,
            span: self.span(span),
        };
        let mut body = vec![
            step(self, at, &lo, dims[axis]),
            step(self, from_at, &from_lo, from_dims[from_axis]),
        ];
        self.emit_assign_place(cell, store, worked, span, &mut body);
        let run = |lw: &mut Self, rev: bool| {
            let iter = Iter::Range {
                lo: lw.int_lit(0, span),
                hi: count.clone(),
                step: None,
                rev,
                closed: false,
            };
            let lid = lw.fresh_loop();
            Block {
                stmts: vec![Stmt::For(
                    lid,
                    k,
                    iter,
                    Block {
                        stmts: body.clone(),
                    },
                )],
            }
        };
        let (forward, backward) = (run(self, false), run(self, true));
        let later = self.mk(
            ExprKind::Cmp(CmpOp::Gt, Box::new(lo), Box::new(from_lo)),
            Ty::Bool,
            span,
        );
        out.push(Stmt::If(later, backward, forward));
    }

    /// `g[r, lo..hi] = 0`, `a[l..r] += x`, `g[.., c] min= x`: every cell of
    /// one part of a row or column takes the one value, or is worked
    /// with it, where it is, in one loop.
    fn grid_fill(
        &mut self,
        target: &ast::Expr,
        op: ast::AssignOp,
        value: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) {
        let span = target.span;
        let Some((base, indices, axis)) = self.grid_row(target) else {
            return;
        };
        let place = self.place(base);
        let place = self.settle_indexes(place, out);
        if let PlaceKind::Local(s) = root_place(&place) {
            self.cx.mut_slots.insert(s);
        }
        let Some((dims, elem)) = Self::axes_of(&place.ty) else {
            return;
        };
        let at = self.temp(match dims[axis] {
            Dim::Pos(d) => Ty::Pos(d),
            Dim::Bound(d) => Ty::Bound(d),
            Dim::Enum(e) => Ty::Enum(e),
            Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
        });
        let cell = self.axis_cell(&place.ty, indices, at, span);
        let cell: Vec<Expr> = cell
            .into_iter()
            .enumerate()
            .map(|(k, index)| {
                if k == axis {
                    index
                } else {
                    self.hold(index, out)
                }
            })
            .collect();
        let grid = place_to_expr(&place);
        let len = self.axis_len(grid, axis, span);
        let (lo, hi) = self.axis_part(&indices[axis], dims[axis], len, out);
        let (lo, hi) = (self.hold(lo, out), self.hold(hi, out));
        let cell = Place {
            kind: PlaceKind::Index(Box::new(place), cell),
            ty: elem.clone(),
            span: self.span(span),
        };
        let value = self.expect(value, &elem);
        let value = self.hold(value, out);
        let iter = Iter::Range {
            lo,
            hi,
            step: None,
            rev: false,
            closed: false,
        };
        let lid = self.fresh_loop();
        let mut body = Vec::new();
        self.emit_assign_place(cell, op, value, span, &mut body);
        out.push(Stmt::For(lid, at, iter, Block { stmts: body }));
    }

    /// `a max= b` on two rows: every cell of `a` takes the larger of the
    /// two, in place. When `asked`, gives the slot that says whether any
    /// cell changed.
    pub(super) fn row_keep(
        &mut self,
        place: Place,
        max: bool,
        value: Expr,
        asked: bool,
        span: Span,
        out: &mut Vec<Stmt>,
    ) -> Option<Slot> {
        let changed = asked.then(|| {
            let changed = self.temp(Ty::Bool);
            out.push(Stmt::Let(
                changed,
                Some(self.mk(ExprKind::Bool(false), Ty::Bool, span)),
            ));
            changed
        });
        let value = self.coerce(value, &place.ty, span);
        if value.ty != place.ty {
            return changed;
        }
        let (dim, elem) = match &place.ty {
            Ty::Array { dims, elem } => (dims[0], (**elem).clone()),
            Ty::List(elem) => (Dim::Int, (**elem).clone()),
            _ => unreachable!("the caller asked `is_row`"),
        };
        // A variable is read where it is; anything else is worked out once.
        let other = match value.kind {
            ExprKind::Local(_) | ExprKind::Global(_) => value,
            _ => self.hold(value, out),
        };
        let row = place_to_expr(&place);
        let len = match dim {
            Dim::Pos(d) => self.mk(ExprKind::DomainLen(d), Ty::Int, span),
            _ => {
                let len = self.len_of(row.clone(), span);
                let len = self.hold(len, out);
                let theirs = self.len_of(other.clone(), span);
                let same = self.mk(
                    ExprKind::Cmp(CmpOp::Ne, Box::new(theirs), Box::new(len.clone())),
                    Ty::Bool,
                    span,
                );
                let word = if max { "max=" } else { "min=" };
                let trap = Stmt::Trap(
                    format!("`a {word} b` needs two rows of one length"),
                    self.span(span),
                );
                out.push(Stmt::If(
                    same,
                    Block { stmts: vec![trap] },
                    Block::default(),
                ));
                len
            }
        };
        let k = self.temp(Ty::Int);
        let at = match dim {
            // The loop keeps `k` under the domain's length.
            Dim::Pos(d) => self.mk(
                ExprKind::Cast(Cast::Trust, Box::new(self.local(k, span))),
                Ty::Pos(d),
                span,
            ),
            _ => self.local(k, span),
        };
        let cell = Place {
            kind: PlaceKind::Index(Box::new(place), vec![at.clone()]),
            ty: elem.clone(),
            span: self.span(span),
        };
        let theirs = self.mk(ExprKind::Index(Box::new(other), vec![at]), elem, span);
        let better = self.mk(
            ExprKind::Cmp(
                if max { CmpOp::Gt } else { CmpOp::Lt },
                Box::new(theirs.clone()),
                Box::new(place_to_expr(&cell)),
            ),
            Ty::Bool,
            span,
        );
        let mut take = vec![Stmt::Assign(cell, AssignOp::Set, theirs)];
        if let Some(changed) = changed {
            let flag = Place {
                kind: PlaceKind::Local(changed),
                ty: Ty::Bool,
                span: self.span(span),
            };
            take.push(Stmt::Assign(
                flag,
                AssignOp::Set,
                self.mk(ExprKind::Bool(true), Ty::Bool, span),
            ));
        }
        let body = Block {
            stmts: vec![Stmt::If(better, Block { stmts: take }, Block::default())],
        };
        let iter = Iter::Range {
            lo: self.int_lit(0, span),
            hi: len,
            step: None,
            rev: false,
            closed: false,
        };
        let lid = self.fresh_loop();
        out.push(Stmt::For(lid, k, iter, body));
        changed
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
            ast::AssignOp::Max | ast::AssignOp::Min if self.is_row(&place.ty) => {
                self.row_keep(place, op == ast::AssignOp::Max, value, false, span, out);
                return;
            }
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
                    let r_some =
                        self.mk(ExprKind::IsSome(Box::new(r_expr.clone())), Ty::Bool, span);
                    let r_val = self.mk(
                        ExprKind::Unwrap(Box::new(r_expr), false),
                        (*inner).clone(),
                        span,
                    );
                    let p_expr = place_to_expr(&place);
                    let p_some =
                        self.mk(ExprKind::IsSome(Box::new(p_expr.clone())), Ty::Bool, span);
                    let p_none =
                        self.mk(ExprKind::Unary(UnOp::Not, Box::new(p_some)), Ty::Bool, span);
                    let p_val = self.mk(
                        ExprKind::Unwrap(Box::new(p_expr), false),
                        (*inner).clone(),
                        span,
                    );
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
                                Block {
                                    stmts: vec![set_stmt],
                                },
                                Block::default(),
                            )],
                        },
                        Block::default(),
                    ));
                    return;
                }
                // `best min= cand` with a `?T` on the right skips `none`.
                if matches!(&value.ty, Ty::Opt(inner) if **inner == place.ty)
                    && self.ordered(&place.ty)
                {
                    let op = if op == ast::AssignOp::Max {
                        AssignOp::Max
                    } else {
                        AssignOp::Min
                    };
                    let slot = self.temp(value.ty.clone());
                    out.push(Stmt::Let(slot, Some(value)));
                    let held = self.local(slot, span);
                    let some = self.mk(ExprKind::IsSome(Box::new(held.clone())), Ty::Bool, span);
                    let inside = self.mk(
                        ExprKind::Unwrap(Box::new(held), false),
                        place.ty.clone(),
                        span,
                    );
                    let mut branch = Vec::new();
                    self.emit_assign(place, op, inside, span, &mut branch);
                    out.push(Stmt::If(some, Block { stmts: branch }, Block::default()));
                    return;
                }
                if !matches!(place.ty, Ty::Error) && !self.ordered(&place.ty) {
                    let word = if op == ast::AssignOp::Max {
                        "max="
                    } else {
                        "min="
                    };
                    let message = match &place.ty {
                        Ty::Array { dims, .. } if dims.len() > 1 => format!(
                            "`{word}` works on one row at a time, not on a whole grid; write `g[r, ..] {word} h[r, ..]` in a loop over the rows"
                        ),
                        Ty::Array { elem, .. } | Ty::List(elem) if matches!(**elem, Ty::Opt(_)) => format!(
                            "`{word}` on whole rows needs cells with an order, and a `{}` cell may be `none`; write `a[i] {word} b[i]` in a loop over the cells, which skips a `none`",
                            self.show(elem)
                        ),
                        _ => format!(
                            "`max=` and `min=` need values with an order, not `{}`",
                            self.show(&place.ty)
                        ),
                    };
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
                    // `z **= e` is `z = z.pow(e)`, as `z ** e` is.
                    let pow = match (bin, &value.ty) {
                        (BinOp::Pow, Ty::Int) => self.structs[sid].methods.get("pow").copied(),
                        _ => None,
                    };
                    if let Some(fn_id) = pow {
                        self.queue_fn(fn_id);
                        let ret = self.ret_of(fn_id, span);
                        let call = Expr {
                            kind: ExprKind::Call(
                                fn_id,
                                vec![Arg::Value(place_to_expr(&place)), Arg::Value(value)],
                            ),
                            ty: ret,
                            span: self.span(span),
                        };
                        let call = self.coerce(call, &place.ty, span);
                        out.push(Stmt::Assign(place, AssignOp::Set, call));
                        return;
                    }
                    if let Some(fn_id) = mname.and_then(|m| self.struct_op_fn(sid, m)) {
                        let lhs_expr = place_to_expr(&place);
                        let rhs_val = self.coerce(value, &place.ty, span);
                        let ret = self.ret_of(fn_id, span);
                        let call = Expr {
                            kind: ExprKind::Call(
                                fn_id,
                                vec![Arg::Value(lhs_expr), Arg::Value(rhs_val)],
                            ),
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
                    let missing =
                        self.mk(ExprKind::Unary(UnOp::Not, Box::new(some)), Ty::Bool, span);
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
                // `s += c` appends in place; `s + c` would build a new string.
                if let (Ty::Str, Add, Ty::Char) = (&place.ty, bin, &value.ty) {
                    let push = ExprKind::Mutate(Mutation::Push, Box::new(place), vec![value]);
                    out.push(Stmt::Expr(self.mk(push, Ty::Unit, span)));
                    return;
                }
                let value = match (&place.ty, bin) {
                    (
                        Ty::Int,
                        Add | Sub | Mul | Div | Rem | Pow | BitAnd | BitOr | BitXor | Shl | Shr,
                    ) => self.coerce(value, &Ty::Int, span),
                    (Ty::I128 | Ty::U64, Shl | Shr) => self.coerce(value, &Ty::Int, span),
                    (
                        wide @ (Ty::I128 | Ty::U64),
                        Add | Sub | Mul | Div | Rem | Pow | BitAnd | BitOr | BitXor,
                    ) => self.coerce(value, &wide.clone(), span),
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
        self.emit_assign(place, op, value, span, out);
    }

    /// True for the `o` of `(o ??= init)`, which holds a value from there on.
    pub(super) fn is_filled(&self, e: &ast::Expr) -> bool {
        self.filled
            .contains(&(self.cur_mod, e.span.start, e.span.end))
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

    pub(super) fn find_projection_on(
        &mut self,
        base: &ast::Expr,
        method: &str,
    ) -> Option<ProjectionInfo<'a>> {
        let diag_len = self.diags.len();
        let b = self.expr(base, None);
        self.diags.truncate(diag_len);
        if let Ty::Struct(sid) = b.ty {
            self.structs[sid].projections.get(method).cloned()
        } else {
            None
        }
    }

    /// Evaluate index subexpressions in `place` once into temp locals so a named
    /// mutable alias (`let st = mut a[k % 2]`) stays bound to the same element even
    /// if `k` changes later.
    fn freeze_place(&mut self, place: Place, out: &mut Vec<Stmt>) -> Place {
        let Place { kind, ty, span } = place;
        let kind = match kind {
            PlaceKind::Local(_) | PlaceKind::Global(_) => kind,
            PlaceKind::Field(base, fi) => {
                PlaceKind::Field(Box::new(self.freeze_place(*base, out)), fi)
            }
            PlaceKind::Some(base) => PlaceKind::Some(Box::new(self.freeze_place(*base, out))),
            PlaceKind::Index(base, idxs) => {
                let base = self.freeze_place(*base, out);
                let frozen_idxs = idxs
                    .into_iter()
                    .map(|idx| {
                        let wrap = match &idx.kind {
                            ExprKind::Cast(cast @ (Cast::Raw | Cast::Trust), _) => Some(*cast),
                            _ => None,
                        };
                        let slot = self.temp(idx.ty.clone());
                        let ispan = idx.span;
                        let ity = idx.ty.clone();
                        out.push(Stmt::Let(slot, Some(idx)));
                        let held = Expr {
                            kind: ExprKind::Local(slot),
                            ty: ity.clone(),
                            span: ispan,
                        };
                        match wrap {
                            Some(cast) => Expr {
                                kind: ExprKind::Cast(cast, Box::new(held)),
                                ty: ity,
                                span: ispan,
                            },
                            None => held,
                        }
                    })
                    .collect();
                PlaceKind::Index(Box::new(base), frozen_idxs)
            }
        };
        Place { kind, ty, span }
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
                if let ast::ExprKind::Name(n) = &e.kind {
                    if let Some(aliased) = self.cx.mut_aliases.get(n).cloned() {
                        return aliased;
                    }
                }
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
            // `xs.last!` and `xs.first!` are the cells `xs[xs.len - 1]` and
            // `xs[0]`: places, as `xs[i]` is.
            ast::ExprKind::Unwrap(inner)
                if matches!(
                    &inner.kind,
                    ast::ExprKind::Field { name, .. } if matches!(name.text.as_str(), "first" | "last")
                ) =>
            {
                let ast::ExprKind::Field { base, name } = &inner.kind else {
                    unreachable!("matched above")
                };
                let seq = self.place(base);
                let elem = match &seq.ty {
                    Ty::List(elem) => (**elem).clone(),
                    Ty::Array { dims, elem } if matches!(dims.as_slice(), [Dim::Int]) => {
                        (**elem).clone()
                    }
                    Ty::Error => return error,
                    other => {
                        let message = format!(
                            "`.{}!` is a cell of a list or a `[n]T` array, and this is a `{}`",
                            name.text,
                            self.show(other)
                        );
                        self.error(codes::UNKNOWN_FIELD, name.span, message);
                        return error;
                    }
                };
                let at = if name.text == "first" {
                    self.int_lit(0, span)
                } else {
                    let len = ExprKind::Builtin(Builtin::Len, vec![place_to_expr(&seq)]);
                    let len = self.mk(len, Ty::Int, span);
                    self.int_add(len, -1)
                };
                Place {
                    kind: PlaceKind::Index(Box::new(seq), vec![at]),
                    ty: elem,
                    span,
                }
            }
            ast::ExprKind::Unwrap(inner) => {
                let place = self.place(inner);
                match place.ty.clone() {
                    Ty::Opt(held) => {
                        let mut lead = Vec::new();
                        let place = self.settle_indexes(place, &mut lead);
                        let holds = place_to_expr(&place);
                        let some = self.mk(ExprKind::IsSome(Box::new(holds)), Ty::Bool, span);
                        let none =
                            self.mk(ExprKind::Unary(UnOp::Not, Box::new(some)), Ty::Bool, span);
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
                let base_place = self.place(base);
                if indices.len() == 1 && Self::sliceable_mut_elem(&base_place.ty).is_some() {
                    if matches!(indices[0].kind, ast::ExprKind::Range { .. }) {
                        if let Some((lo, hi, elem_ty)) =
                            self.mut_slice_bounds(&base_place, &indices[0], e.span)
                        {
                            let view_ty = Ty::MutView(Box::new(elem_ty));
                            let view_expr = self.mk(
                                ExprKind::MutSlice(
                                    Box::new(base_place),
                                    Box::new(lo),
                                    Box::new(hi),
                                ),
                                view_ty.clone(),
                                e.span,
                            );
                            let tmp = self.temp(view_ty.clone());
                            self.cx.pending.push(Stmt::Let(tmp, Some(view_expr)));
                            return Place {
                                kind: PlaceKind::Local(tmp),
                                ty: view_ty,
                                span,
                            };
                        }
                    } else if matches!(self.probe_ty_silent(&indices[0]), Some(Ty::Range(_))) {
                        let range_val = self.expr(&indices[0], None);
                        if let Some((lo, hi, elem_ty)) =
                            self.mut_slice_range_val_bounds(&base_place, range_val, e.span)
                        {
                            let view_ty = Ty::MutView(Box::new(elem_ty));
                            let view_expr = self.mk(
                                ExprKind::MutSlice(
                                    Box::new(base_place),
                                    Box::new(lo),
                                    Box::new(hi),
                                ),
                                view_ty.clone(),
                                e.span,
                            );
                            let tmp = self.temp(view_ty.clone());
                            self.cx.pending.push(Stmt::Let(tmp, Some(view_expr)));
                            return Place {
                                kind: PlaceKind::Local(tmp),
                                ty: view_ty,
                                span,
                            };
                        }
                    }
                }
                let (indices, ty) = self.index_args(&base_place.ty, indices, e.span);
                let indices = self.vouched(&base_place.ty, indices);
                Place {
                    kind: PlaceKind::Index(Box::new(base_place), indices),
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
                // In the branch the test holds; past it, the test has
                // failed.
                let (test, _) = &branches[0];
                let mark = self.cx.present.len();
                let mutators = self.mutators.clone();
                let held = narrow::in_branch(test, body, &mut |f, x| mutators.writes(f, x));
                self.cx.present.extend(held.into_iter().map(String::from));
                let then = self.block(body, tail);
                self.cx.present.truncate(mark);
                let mutators = self.mutators.clone();
                let failed = narrow::past(test, rest, otherwise, &mut |f, x| mutators.writes(f, x));
                self.cx.present.extend(failed.into_iter().map(String::from));
                let mut els = Vec::new();
                self.if_chain(rest, otherwise, tail, &mut els);
                self.cx.present.truncate(mark);
                out.push(Stmt::If(cond, then, Block { stmts: els }));
            }
            ast::Cond::Let { .. } | ast::Cond::Guard { .. } => {
                let mut els = Vec::new();
                self.if_chain(rest, otherwise, tail, &mut els);
                self.clause_chain(cond, body, tail, &els, out);
            }
            // `if const`: the test is settled here, and only the branch
            // it picks is lowered, so the other costs nothing at run time.
            ast::Cond::Const(cond) => match self.eval_const_bool(cond) {
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
            },
        }
    }

    /// Lowers `if let pat = value` / `value is pat` / `while let pat = value`.
    /// Variant patterns (`Seeking`, `Phase.Seeking`, `Split(base)`) match an
    /// enum value directly via `self.pattern`; all other patterns unwrap an
    /// optional via `self.optional`.
    pub(super) fn cond_let(
        &mut self,
        pat: &ast::Pattern,
        value: &ast::Expr,
        what: &str,
        before: &mut Vec<Stmt>,
        binds: &mut Vec<Stmt>,
    ) -> Option<Expr> {
        if let ast::PatternKind::Literal(lit) = &pat.kind {
            if matches!(lit.kind, ast::ExprKind::None) {
                let (present, _) = self.optional(value, what, before)?;
                let span = present.span;
                return Some(Expr {
                    kind: ExprKind::Unary(UnOp::Not, Box::new(present)),
                    ty: Ty::Bool,
                    span,
                });
            }
        }
        if let ast::PatternKind::Variant { path, args } = &pat.kind {
            if path.len() == 1 && path[0].text == "some" && args.len() == 1 && refutable(&args[0]) {
                let value_expr = self.expr(value, None);
                if value_expr.ty == Ty::Error {
                    return None;
                }
                let tmp = self.temp(value_expr.ty.clone());
                before.push(Stmt::Let(tmp, Some(value_expr)));
                let local = self.local(tmp, value.span);
                let test = self.pattern(pat, &local, binds);
                return Some(
                    test.unwrap_or_else(|| self.mk(ExprKind::Bool(true), Ty::Bool, value.span)),
                );
            }
            if path.len() == 1 && path[0].text == "some" && args.len() == 1 {
                let (present, inner) = self.optional(value, what, before)?;
                self.bind(&args[0], inner, binds);
                return Some(present);
            }
            let value_expr = self.expr(value, None);
            if matches!(value_expr.ty, Ty::Enum(_)) {
                let tmp = self.temp(value_expr.ty.clone());
                before.push(Stmt::Let(tmp, Some(value_expr)));
                let local = self.local(tmp, value.span);
                let test = self.pattern(pat, &local, binds);
                return Some(
                    test.unwrap_or_else(|| self.mk(ExprKind::Bool(true), Ty::Bool, value.span)),
                );
            }
            if value_expr.ty == Ty::Error {
                return None;
            }
        }
        let (present, inner) = self.optional(value, what, before)?;
        self.bind(pat, inner, binds);
        Some(present)
    }

    /// Evaluates the optional `value` into a temporary; gives the test
    /// that it is present and the value inside.
    pub(super) fn optional(
        &mut self,
        value: &ast::Expr,
        what: &str,
        out: &mut Vec<Stmt>,
    ) -> Option<(Expr, Expr)> {
        self.cx.as_written = narrow::asks_optional(value);
        let value_expr = self.expr(value, None);
        self.cx.as_written = false;
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

    /// The clauses of `if let a = x, let b = y, a < b:` after the first
    /// branch's `else` is known: each clause is tried only when those
    /// before it held, and the first that fails goes to `els`.
    fn clause_chain(
        &mut self,
        cond: &ast::Cond,
        body: &ast::Block,
        tail: Tail,
        els: &[Stmt],
        out: &mut Vec<Stmt>,
    ) {
        let els = || Block {
            stmts: els.to_vec(),
        };
        match cond {
            ast::Cond::Expr(test) => {
                let test = self.expect(test, &Ty::Bool);
                let mark = self.cx.present.len();
                let mutators = self.mutators.clone();
                let held = narrow::in_branch(cond, body, &mut |f, x| mutators.writes(f, x));
                self.cx.present.extend(held.into_iter().map(String::from));
                let then = self.block(body, tail);
                self.cx.present.truncate(mark);
                out.push(Stmt::If(test, then, els()));
            }
            ast::Cond::Guard { test, and } => {
                let test = self.expect(test, &Ty::Bool);
                let mut then = Vec::new();
                self.clause_chain(and, body, tail, els().stmts.as_slice(), &mut then);
                out.push(Stmt::If(test, Block { stmts: then }, els()));
            }
            ast::Cond::Let { pat, value, and } => {
                self.cx.scopes.push(HashMap::new());
                let mut then = Vec::new();
                let Some(present) = self.cond_let(pat, value, "if let", out, &mut then) else {
                    self.cx.scopes.pop();
                    return;
                };
                match and {
                    Some(and) => {
                        self.clause_chain(and, body, tail, els().stmts.as_slice(), &mut then)
                    }
                    None => then.extend(self.block(body, tail).stmts),
                }
                self.cx.scopes.pop();
                out.push(Stmt::If(present, Block { stmts: then }, els()));
            }
            ast::Cond::Const(c) => {
                self.unsupported(c.span, "`const` after another clause of a condition")
            }
        }
    }

    /// The clauses of a `while let`, at the top of the loop's body: the
    /// first that fails leaves the loop. False when one could not be read.
    fn while_clauses(&mut self, cond: &ast::Cond, stmts: &mut Vec<Stmt>) -> bool {
        let leave_unless = |lw: &mut Self, holds: Expr, stmts: &mut Vec<Stmt>| {
            let span = holds.span;
            let fails = lw.mk(
                ExprKind::Unary(UnOp::Not, Box::new(holds)),
                Ty::Bool,
                Span::new(0, 0),
            );
            let fails = Expr { span, ..fails };
            stmts.push(Stmt::If(
                fails,
                Block {
                    stmts: vec![Stmt::Break(None)],
                },
                Block::default(),
            ));
        };
        match cond {
            ast::Cond::Expr(test) => {
                let test = self.expect(test, &Ty::Bool);
                leave_unless(self, test, stmts);
                true
            }
            ast::Cond::Guard { test, and } => {
                let test = self.expect(test, &Ty::Bool);
                leave_unless(self, test, stmts);
                self.while_clauses(and, stmts)
            }
            ast::Cond::Let { pat, value, and } => {
                let mut binds = Vec::new();
                let Some(present) = self.cond_let(pat, value, "while let", stmts, &mut binds)
                else {
                    return false;
                };
                leave_unless(self, present, stmts);
                stmts.extend(binds);
                and.as_ref()
                    .is_none_or(|and| self.while_clauses(and, stmts))
            }
            ast::Cond::Const(c) => {
                self.unsupported(c.span, "`const` in a `while` condition");
                false
            }
        }
    }

    fn while_stmt(
        &mut self,
        cond: &ast::Cond,
        body: &ast::Block,
        else_body: Option<&ast::Block>,
        out: &mut Vec<Stmt>,
    ) {
        match cond {
            ast::Cond::Expr(cond) => {
                let cond = self.expect(cond, &Ty::Bool);
                let id = self.open_loop(None);
                let else_flag = self.begin_loop_else(id, else_body, out);
                let body = self.block(body, Tail::Discard);
                self.cx.loops.pop();
                out.push(Stmt::While(id, cond, body));
                self.end_loop_else(id, else_flag, else_body, out);
            }
            ast::Cond::Let { .. } | ast::Cond::Guard { .. } => {
                let id = self.open_loop(None);
                let else_flag = self.begin_loop_else(id, else_body, out);
                self.cx.scopes.push(HashMap::new());
                let mut stmts = Vec::new();
                if self.while_clauses(cond, &mut stmts) {
                    stmts.extend(self.block(body, Tail::Discard).stmts);
                }
                self.cx.scopes.pop();
                self.cx.loops.pop();
                let always = self.mk(ExprKind::Bool(true), Ty::Bool, body.span);
                out.push(Stmt::While(id, always, Block { stmts }));
                self.end_loop_else(id, else_flag, else_body, out);
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
        self.cx.as_written = narrow::asks_optional(scrutinee);
        let value = self.expr(scrutinee, None);
        self.cx.as_written = false;
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
        // An arm that gives its value has left by here, unless the arms
        // are written out as statements that store it.
        let stored = tail == Tail::Give && self.cx.dos.last().is_some_and(|frame| frame.open);
        if tail == Tail::Discard || stored {
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
            ast::PatternKind::Wildcard | ast::PatternKind::Domain(_) | ast::PatternKind::Error => {
                None
            }
            ast::PatternKind::Name(name) => {
                self.declare(name, value.clone(), false, binds);
                None
            }
            ast::PatternKind::Var(inner) => self.pattern(inner, value, binds),
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
                if path.len() == 1 && name.text == "some" {
                    let Ty::Opt(inner_ty) = value.ty.clone() else {
                        if value.ty != Ty::Error {
                            let message = format!(
                                "`some(...)` matches an optional, but the value is a `{}`",
                                self.show(&value.ty)
                            );
                            self.error(codes::TYPE_MISMATCH, pat.span, message);
                        }
                        return None;
                    };
                    if args.len() != 1 {
                        let message = format!(
                            "`some` holds 1 value, but this pattern names {}",
                            args.len()
                        );
                        self.error(codes::WRONG_ARG_COUNT, pat.span, message);
                        return None;
                    }
                    let is_some = self.mk(
                        ExprKind::IsSome(Box::new(value.clone())),
                        Ty::Bool,
                        pat.span,
                    );
                    let unwrapped = self.mk(
                        ExprKind::Unwrap(Box::new(value.clone()), false),
                        *inner_ty,
                        pat.span,
                    );
                    return and(Some(is_some), self.pattern(&args[0], &unwrapped, binds));
                }
                if let Ty::Pos(d) = value.ty {
                    if let Some(info) = self.sum_enums.get(&d).cloned() {
                        let Some(tag) = info.variants.iter().position(|(v, _)| *v == name.text)
                        else {
                            let message =
                                format!("`{}` has no variant `{}`", self.domains[d], name.text);
                            self.error(codes::UNKNOWN_FIELD, name.span, message);
                            return None;
                        };
                        let factors = &info.variants[tag].1;
                        if factors.len() != args.len() {
                            let message = format!(
                                "`{}` holds {} values, but this pattern names {}",
                                name.text,
                                factors.len(),
                                args.len()
                            );
                            self.error(codes::WRONG_ARG_COUNT, pat.span, message);
                            return None;
                        }
                        let mut test = Some(self.is_sum_variant(d, tag, value.clone(), pat.span));
                        for (i, arg) in args.iter().enumerate() {
                            let payload =
                                self.decode_sum_variant_field(d, tag, i, value.clone(), pat.span);
                            test = and(test, self.pattern(arg, &payload, binds));
                        }
                        return test;
                    }
                }
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

/// `read(none: sentinel)` (optionally with `gen: ...`).
fn read_none_arg(e: &ast::Expr) -> Option<&ast::Expr> {
    let ast::ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    if !matches!(&callee.kind, ast::ExprKind::Name(n) if n == "read") {
        return None;
    }
    let mut none_val = None;
    for a in args {
        match a.name.as_ref().map(|n| n.text.as_str()) {
            Some("none") => none_val = Some(&a.value),
            Some("gen") => {}
            _ => return None,
        }
    }
    none_val
}

/// `read()`, `read(gen: ...)`, or `read(none: ...)`.
pub(super) fn is_read_call(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Call { callee, args } => {
            matches!(&callee.kind, ast::ExprKind::Name(n) if n == "read")
                && args.iter().all(|a| {
                    a.name
                        .as_ref()
                        .is_some_and(|n| n.text == "gen" || n.text == "none")
                })
        }
        _ => false,
    }
}

fn find_pos_domain_in_ty(ty: &Ty) -> Option<DomainId> {
    match ty {
        Ty::Pos(d) | Ty::Bound(d) => Some(*d),
        Ty::Array { dims, elem } => dims
            .iter()
            .find_map(|dim| match dim {
                Dim::Pos(d) | Dim::Bound(d) | Dim::Sym(d) => Some(*d),
                Dim::Enum(_) | Dim::Int | Dim::IntFrom(_) => None,
            })
            .or_else(|| find_pos_domain_in_ty(elem)),
        Ty::List(elem) | Ty::Opt(elem) | Ty::Range(elem) => find_pos_domain_in_ty(elem),
        Ty::Tuple(ts) => ts.iter().find_map(find_pos_domain_in_ty),
        Ty::Record(fs) => fs.iter().find_map(|(_, t)| find_pos_domain_in_ty(t)),
        _ => None,
    }
}

pub(super) fn remap_domain_ty(ty: &Ty, from: DomainId, to: DomainId) -> Ty {
    if from == to {
        return ty.clone();
    }
    let map_d = |d: DomainId| if d == from { to } else { d };
    match ty {
        Ty::Pos(d) => Ty::Pos(map_d(*d)),
        Ty::Bound(d) => Ty::Bound(map_d(*d)),
        Ty::Range(inner) => Ty::Range(Box::new(remap_domain_ty(inner, from, to))),
        Ty::Array { dims, elem } => Ty::Array {
            dims: dims
                .iter()
                .map(|dim| match dim {
                    Dim::Pos(d) => Dim::Pos(map_d(*d)),
                    Dim::Bound(d) => Dim::Bound(map_d(*d)),
                    Dim::Sym(d) => Dim::Sym(map_d(*d)),
                    Dim::Enum(e) => Dim::Enum(*e),
                    Dim::Int => Dim::Int,
                    Dim::IntFrom(lo) => Dim::IntFrom(*lo),
                })
                .collect(),
            elem: Box::new(remap_domain_ty(elem, from, to)),
        },
        Ty::List(e) => Ty::List(Box::new(remap_domain_ty(e, from, to))),
        Ty::Opt(e) => Ty::Opt(Box::new(remap_domain_ty(e, from, to))),
        Ty::Tuple(ts) => Ty::Tuple(ts.iter().map(|t| remap_domain_ty(t, from, to)).collect()),
        Ty::Record(fs) => Ty::Record(
            fs.iter()
                .map(|(n, t)| (n.clone(), remap_domain_ty(t, from, to)))
                .collect(),
        ),
        Ty::Map(k, v) => Ty::Map(
            Box::new(remap_domain_ty(k, from, to)),
            Box::new(remap_domain_ty(v, from, to)),
        ),
        Ty::HashMap(k, v) => Ty::HashMap(
            Box::new(remap_domain_ty(k, from, to)),
            Box::new(remap_domain_ty(v, from, to)),
        ),
        Ty::Set(k) => Ty::Set(Box::new(remap_domain_ty(k, from, to))),
        Ty::HashSet(k) => Ty::HashSet(Box::new(remap_domain_ty(k, from, to))),
        Ty::Fn(ps, r) => Ty::Fn(
            ps.iter().map(|t| remap_domain_ty(t, from, to)).collect(),
            Box::new(remap_domain_ty(r, from, to)),
        ),
        other => other.clone(),
    }
}

pub(super) fn root_place(place: &Place) -> PlaceKind {
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
                let any_none = rest
                    .iter()
                    .any(|(_, r)| matches!(r.kind, ast::ExprKind::None));
                (first_is_name && any_none)
                    || in_expr(name, first)
                    || rest.iter().any(|(_, r)| in_expr(name, r))
            }
            ast::ExprKind::If {
                branches,
                otherwise,
            } => {
                branches
                    .iter()
                    .any(|(c, b)| in_cond(name, c) || uses_as_optional(name, &b.stmts))
                    || otherwise
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
            ast::Cond::Guard { test, and } => in_expr(name, test) || in_cond(name, and),
            ast::Cond::Expr(e) | ast::Cond::Const(e) => in_expr(name, e),
        }
    }
    stmts.iter().any(|s| match &s.kind {
        ast::StmtKind::Expr(e) => in_expr(name, e),
        ast::StmtKind::Let { init, .. } => init.as_ref().is_some_and(|e| in_expr(name, e)),
        ast::StmtKind::Assign { value, .. } => in_expr(name, value),
        ast::StmtKind::While {
            cond,
            body,
            else_body,
        } => {
            in_cond(name, cond)
                || uses_as_optional(name, &body.stmts)
                || else_body
                    .as_ref()
                    .is_some_and(|eb| uses_as_optional(name, &eb.stmts))
        }
        ast::StmtKind::For {
            body, else_body, ..
        } => {
            uses_as_optional(name, &body.stmts)
                || else_body
                    .as_ref()
                    .is_some_and(|eb| uses_as_optional(name, &eb.stmts))
        }
        ast::StmtKind::Return(Some(e)) => in_expr(name, e),
        _ => false,
    })
}

/// A literal: in `var a: [V]Int = 0` it is the value of every cell.
pub(super) fn is_literal(e: &ast::Expr) -> bool {
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
pub(super) fn settled(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(_)
        | ExprKind::Bool(_)
        | ExprKind::Char(_)
        | ExprKind::Local(_)
        | ExprKind::Global(_) => true,
        ExprKind::Unary(_, inner) | ExprKind::Cast(_, inner) | ExprKind::Field(inner, _) => {
            settled(inner)
        }
        ExprKind::Binary(op, l, r) => {
            !matches!(op, BinOp::Div | BinOp::Rem | BinOp::Pow) && settled(l) && settled(r)
        }
        _ => false,
    }
}

fn expr_mentions_name(e: &ast::Expr, target: &str) -> bool {
    struct Find<'a> {
        target: &'a str,
        found: bool,
    }
    impl cpl_syntax::visit::Visitor for Find<'_> {
        fn expr(&mut self, expr: &ast::Expr) {
            if !self.found {
                if let ast::ExprKind::Name(n) = &expr.kind {
                    if n == self.target {
                        self.found = true;
                    }
                }
            }
        }
    }
    let mut f = Find {
        target,
        found: false,
    };
    cpl_syntax::visit::walk_expr(e, &mut f);
    f.found
}

fn leaves(stmt: &ast::Stmt) -> bool {
    matches!(
        stmt.kind,
        ast::StmtKind::Return(_) | ast::StmtKind::Break(_) | ast::StmtKind::Continue(_)
    )
}

/// Whether `e` is a `match` or `if` with an arm that ends in `return`,
/// `break` or `continue`.
fn has_leaving_arm(e: &ast::Expr) -> bool {
    let ends = |block: &ast::Block| block.stmts.last().is_some_and(leaves);
    match &e.kind {
        ast::ExprKind::Match { arms, .. } => arms.iter().any(|arm| ends(&arm.body)),
        ast::ExprKind::If {
            branches,
            otherwise: Some(otherwise),
        } => ends(otherwise) || branches.iter().any(|(_, block)| ends(block)),
        _ => false,
    }
}

/// The `return`, `break` or `continue` on the right of `x ?? return`.
pub(super) fn diverging_default(e: &ast::Expr) -> Option<&ast::Stmt> {
    let ast::ExprKind::Binary {
        op: BinOp::Coalesce,
        rhs,
        ..
    } = &e.kind
    else {
        return None;
    };
    match &rhs.kind {
        ast::ExprKind::Block(block) => match block.stmts.as_slice() {
            [stmt @ ast::Stmt {
                kind:
                    ast::StmtKind::Return(_) | ast::StmtKind::Break(_) | ast::StmtKind::Continue(_),
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
                if name.text == "filled"
                    && matches!(&base.kind, ast::ExprKind::Name(n) if n == "List")
                {
                    return filled_none_depth(&args[1].value).map(|d| d + 1);
                }
            }
            None
        }
        _ => None,
    }
}

/// Whether `e` only names where a value lives, a variable or a field of
/// one: reading it again costs nothing, and holding it would copy it.
pub(super) fn names_a_place(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Local(_) | ExprKind::Global(_) => true,
        ExprKind::Field(base, _) => names_a_place(base),
        _ => false,
    }
}

/// The number `e` is written as, when it is one.
/// The count along each index, and the cells in order.
type Rows<'a> = (Vec<usize>, Vec<&'a ast::Expr>);

/// One integer loop: its variable, first value and end.
type LoopHead = (Slot, Expr, Expr);

/// How many values a list written for an array of `depth` indices has
/// along each, and its cells in order; `Err` is where a row is not a
/// list or is not as long as the first.
fn literal_shape(list: &ast::Expr, depth: usize) -> Option<Result<Rows<'_>, Span>> {
    let ast::ExprKind::List(items) = &list.kind else {
        return None;
    };
    if depth == 1 {
        return Some(Ok((vec![items.len()], items.iter().collect())));
    }
    let mut counts = vec![items.len()];
    let mut cells = Vec::new();
    for (k, row) in items.iter().enumerate() {
        match literal_shape(row, depth - 1) {
            None => return Some(Err(row.span)),
            Some(Err(at)) => return Some(Err(at)),
            Some(Ok((inner, _))) if inner.contains(&0) => return Some(Err(row.span)),
            Some(Ok((inner, row_cells))) => {
                if k == 0 {
                    counts.extend(inner);
                } else if counts[1..] != inner[..] {
                    return Some(Err(row.span));
                }
                cells.extend(row_cells);
            }
        }
    }
    Some(Ok((counts, cells)))
}

fn written(e: &Expr) -> Option<i64> {
    match e.kind {
        ExprKind::Int(v) => Some(v),
        _ => None,
    }
}

/// `let (gap, Some(best)) = e else: ..` tests `e` against the pattern;
/// an irrefutable one, `let (a, b) = e else: ..`, unwraps an optional `e`.
fn refutable(pat: &ast::Pattern) -> bool {
    match &pat.kind {
        ast::PatternKind::Variant { .. } | ast::PatternKind::Literal(_) => true,
        ast::PatternKind::Tuple(items) => items.iter().any(refutable),
        ast::PatternKind::Or(alts) => alts.iter().all(refutable),
        _ => false,
    }
}

/// Whether a closure or a fn among `stmts` names `name`.
fn captured(stmts: &[ast::Stmt], name: &str) -> bool {
    struct Names<'n>(&'n str, bool);
    impl cpl_syntax::visit::Visitor for Names<'_> {
        fn expr(&mut self, e: &ast::Expr) {
            if matches!(&e.kind, ast::ExprKind::Name(n) if n == self.0) {
                self.1 = true;
            }
        }
    }
    struct Bodies<'n>(&'n str, bool);
    impl cpl_syntax::visit::Visitor for Bodies<'_> {
        fn expr(&mut self, e: &ast::Expr) {
            if let ast::ExprKind::Lambda { body, .. } = &e.kind {
                let mut names = Names(self.0, false);
                cpl_syntax::visit::walk_expr(body, &mut names);
                self.1 |= names.1;
            }
        }
        fn stmt(&mut self, s: &ast::Stmt) {
            if matches!(s.kind, ast::StmtKind::Fn(_)) {
                let mut names = Names(self.0, false);
                cpl_syntax::visit::walk_stmt(s, &mut names);
                self.1 |= names.1;
            }
        }
    }
    let mut bodies = Bodies(name, false);
    for s in stmts {
        cpl_syntax::visit::walk_stmt(s, &mut bodies);
    }
    bodies.1
}
