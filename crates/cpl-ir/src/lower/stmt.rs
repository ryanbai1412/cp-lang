//! Statements, blocks and patterns.

mod assign;
mod bind_mut;
mod cond;
mod domain_decl;
mod lambda_tuple;
mod lends;
mod let_infer;
mod let_value;
mod loops;
mod mutation;
mod place;
mod rows;
mod slice_place;

use super::*;
use crate::walk::{find_pos_domain_in_ty, given};
use cpl_diag::msgs;
use place::{names_a_place, root_place};

/// What writing through `x!` says when `x` is `none`; reading it says so too.
const NO_VALUE: &str = "`!` on `None`: there is no value here";

impl<'a> Lowerer<'a> {
    pub(super) fn block(&mut self, block: &ast::Block, tail: Tail) -> Block {
        self.cx.scopes.push(HashMap::new());
        self.cx.local_syms.push(HashMap::new());
        let present = self.cx.present.len();
        let held = (self.contexts.len(), self.unchecked);
        let mut out = Vec::new();
        if !self.at_top_level() {
            for s in &block.stmts {
                if let ast::StmtKind::Fn(decl) = &s.kind {
                    if !decl.generics.is_empty() {
                        self.cx
                            .local_gen_fns
                            .push((decl.name.text.clone(), decl.clone()));
                    }
                }
            }
            self.predeclare_local_fns(&block.stmts, &mut out);
        }
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
        self.cx.recipes.retain(|(_, d, _)| *d <= depth);
        Block { stmts: out }
    }

    /// A statement that only works out a value: `xs.sorted()`, `s.with(v)`,
    /// `a + b`. Nothing reads it and nothing was changed, so it is a
    /// mistake, most often for the form that changes in place.
    /// The token of `@tag("Q")` or `@tag(1)` on a variant.
    pub(super) fn variant_tag(&self, variant: &ast::Variant) -> Option<String> {
        let tag = variant.tag.as_ref()?;
        match &tag.kind {
            ast::ExprKind::Char(c) => return Some(c.to_string()),
            ast::ExprKind::Str(pieces) => {
                if let [ast::StrPiece::Text(text)] = pieces.as_slice() {
                    return Some(text.clone());
                }
            }
            _ => {}
        }
        let text = self.source(tag.span);
        let text = text.trim();
        Some(
            text.strip_prefix('"')
                .and_then(|t| t.strip_suffix('"'))
                .unwrap_or(text)
                .to_string(),
        )
    }

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
            self.warnings
                .push(cpl_diag::msgs::check::VALUE_DROPPED.warning(span, &[&shown]));
            if let Some(last) = self.warnings.last_mut() {
                last.help =
                    Some("use it (`let y = ...` or `print(...)`), or delete the line".into());
            }
        }
    }

    fn needs_value(&mut self, span: Span) {
        self.report(span, msgs::ir::TYPE_MISMATCH_THIS_BLOCK_IS_USED, &[]);
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
                    self.report(e.span, msgs::ir::TYPE_MISMATCH_AN_IF_USED_AS, &[]);
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

    /// `e` lowered as what the fn returns, checked into its range return type (R03.1.4).
    fn ret_value(&mut self, e: &ast::Expr, hint: Option<&Ty>) -> Expr {
        let cells = self.cx.ret_bounds.clone();
        self.store_cells(e, cells, &mut |l| l.expr(e, hint))
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
                let value = self.ret_value(e, hint.as_ref());
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
                out.push_give(value);
            }
        }
    }

    /// `fn even` calling `fn odd` declared after it: `odd` gets its slot first.
    fn predeclare_local_fns(&mut self, stmts: &[ast::Stmt], out: &mut Vec<Stmt>) {
        for (idx, stmt) in stmts.iter().enumerate() {
            let ast::StmtKind::Fn(decl) = &stmt.kind else {
                continue;
            };
            let plain =
                decl.generics.is_empty() && !stmt.decorators.iter().any(|d| d.name.text == "memo");
            if !plain || !captured(&stmts[..idx], &decl.name.text) {
                continue;
            }
            let (_, _, fn_ty) = self.local_fn_ty(decl);
            let slot = self.new_slot(&decl.name.text, fn_ty);
            self.cx
                .scopes
                .top_mut()
                .insert(decl.name.text.clone(), slot);
            self.cx.mut_slots.insert(slot);
            self.cx.predeclared.insert(decl.name.text.clone(), slot);
            out.push(Stmt::Let(slot, None));
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
        self.collect_mut_slices_stmt(stmt, rest);
        self.note_later_calls(stmt, rest);
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
            std::mem::replace(&mut self.cx.rhs_extent, stmt.rhs_extent()),
            std::mem::take(&mut self.cx.after),
        );
        let at = out.len();
        self.stmt_inner(stmt, rest, out);
        let pending = std::mem::replace(&mut self.cx.pending, outer.1);
        let after = std::mem::replace(&mut self.cx.after, outer.4);
        (self.cx.way_out, self.cx.exits, self.cx.rhs_extent) = (outer.0, outer.2, outer.3);
        out.splice(at..at, pending);
        out.extend(after);
        self.lift_leaving(out, at);
    }

    fn stmt_inner(&mut self, stmt: &ast::Stmt, rest: &[ast::Stmt], out: &mut Vec<Stmt>) {
        if let Some((name, init)) = self.generator_recipe(stmt, rest) {
            let scope = self.cx.scopes.top_mut();
            scope.remove(&name);
            let depth = self.cx.scopes.len();
            self.forget_recipe(&name);
            self.cx.recipes.push((name, depth, init.clone()));
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
                out.push_let(tmp, value);
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
                if self.let_lambda_tuple(stmt, rest, out) {
                    return;
                }
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
                        if lambda_tuple::expr_mentions_name(body, vname)
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
                                .filter_map(|(i, p)| {
                                    let ast::PatternKind::Name(pname) = &p.kind else {
                                        return None;
                                    };
                                    let pname = pname.clone();
                                    let pty =
                                        param_tys.get(i).and_then(|t| t.clone()).or_else(|| {
                                            fn_ty_parts
                                                .as_ref()
                                                .and_then(|(fps, _)| fps.get(i).cloned())
                                        });
                                    Some(ast::Param {
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
                                    })
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
                    if let PlaceKind::Index(map, keys) = &frozen.kind {
                        if let (Ty::Map(..) | Ty::HashMap(..), [key]) = (&map.ty, keys.as_slice()) {
                            if let ExprKind::Local(k) = key.kind {
                                self.cx.locals[k].binds = Some((**map).clone());
                            }
                        }
                    }
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
                                let scope = self.cx.scopes.top_mut();
                                scope.insert(vname.clone(), slot);
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
                                            let scope = self.cx.scopes.top_mut();
                                            scope.insert(vname.clone(), slot);
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
                // `let m = xs.map(f)` (v0.3): bind a lazy `MapView` so each read `m[i]`
                // calls `f(xs[i])` without eagerly mapping the whole sequence.
                if !*mutable && ty.is_none() {
                    if let (Some(init_expr), ast::PatternKind::Name(vname)) = (init, &pat.kind) {
                        if let Some(view) = self.map_view_if_indexed(init_expr, vname, rest, out) {
                            if let ViewBase::Global(_) = view.base {
                                self.mod_syms[self.cur_mod].remove(vname);
                                self.global_map_views
                                    .insert((self.cur_mod, vname.clone()), view);
                                return;
                            }
                            let slot = self.temp(Ty::Unit);
                            let scope = self.cx.scopes.top_mut();
                            scope.insert(vname.clone(), slot);
                            self.cx.map_views.insert(slot, view);
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
                        // `Map<K, V>()` states its type where it is called.
                        let stated = match init.as_ref().map(|e| &e.kind) {
                            Some(ast::ExprKind::Call { callee, args }) if args.is_empty() => {
                                match &callee.kind {
                                    ast::ExprKind::Generic { base, args: tys } => {
                                        match &base.kind {
                                            ast::ExprKind::Name(n) => {
                                                self.range_info_from_ast(&ast::Type {
                                                    kind: ast::TypeKind::Named {
                                                        name: ast::Name {
                                                            text: n.clone(),
                                                            span: base.span,
                                                        },
                                                        args: tys.clone(),
                                                    },
                                                    span: callee.span,
                                                })
                                            }
                                            _ => None,
                                        }
                                    }
                                    _ => None,
                                }
                            }
                            _ => None,
                        };
                        stated
                            .or_else(|| self.expr_range(&value))
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
                    let array_ty_ast = ty.as_ref().filter(|t| {
                        single_name.is_some() && matches!(t.kind, ast::TypeKind::Array { .. })
                    });
                    let array_ty_ast = array_ty_ast.cloned();
                    let runtime_const_name =
                        if !*mutable && !self.at_top_level() && value.ty == Ty::Int {
                            single_name.clone().filter(|n| {
                                n.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                            })
                        } else {
                            None
                        };
                    let bounds =
                        self.let_bounds(*mutable || ty.is_some() || self.at_top_level(), &value);
                    self.bind_mut(pat, value, *mutable, out);
                    self.note_let_bounds(bounds, single_name.as_deref());
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
                            self.cx.local_syms.top_mut().insert(cname, Sym::Global(gid));
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
            ast::StmtKind::TypeAlias {
                name, generics, ty, ..
            } if !self.at_top_level() => {
                if generics.is_empty() {
                    self.mirror_value_args(ty, out);
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
                    if let (Some(sym), scope) = (sym, self.cx.local_syms.top_mut()) {
                        scope.insert(name.text.clone(), sym);
                    }
                }
            }
            ast::StmtKind::Impl {
                trait_name: Some(tn),
                target,
                body,
                ..
            } if matches!(&tn.kind, ast::TypeKind::Named { name, .. } if name.text == "Index") => {
                // Freeze `let count = <expr>` for `impl Index for S`.
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
                                    if n == "count" {
                                        let len = self.expr_as(init, &Ty::Int);
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
                                self.cx.local_syms.top_mut().insert(item.text.clone(), sym);
                            }
                        }
                    } else if *glob {
                        let syms: Vec<(String, Sym)> = self.mod_syms[tmod]
                            .iter()
                            .map(|(k, &v)| (k.clone(), v))
                            .collect();
                        for (k, v) in syms {
                            self.cx.local_syms.top_mut().insert(k, v);
                        }
                    } else if let Some(al) = alias {
                        self.cx
                            .local_syms
                            .top_mut()
                            .insert(al.text.clone(), Sym::Mod(tmod));
                    } else if let Some(last) = path.last() {
                        self.cx
                            .local_syms
                            .top_mut()
                            .insert(last.text.clone(), Sym::Mod(tmod));
                    }
                }
            }
            ast::StmtKind::Fn(decl) if !self.at_top_level() && !decl.generics.is_empty() => {
                // A template outlives this borrow of the block, so it keeps its
                // own copy of the decl; it is instantiated per call like a
                // top-level generic fn, and sees no locals of the enclosing fn.
                let decl: &'static ast::FnDecl = Box::leak(Box::new(decl.clone()));
                let gfid = self.gen_fns.len();
                self.gen_fns.push(super::mono::GenFnTemplate {
                    mod_id: self.cur_mod,
                    offset: self.offset,
                    owner: None,
                    owner_env: HashMap::new(),
                    decl,
                });
                let scope = self.cx.local_syms.top_mut();
                scope.insert(decl.name.text.clone(), Sym::GenFn(gfid));
            }
            ast::StmtKind::Fn(decl) if !self.at_top_level() => {
                let memo = stmt.decorators.iter().any(|d| d.name.text == "memo");
                let into = stmt.memo_into().map(|(table, _)| table);
                if let Some(table) = into {
                    self.memo_into_table(table, decl, out);
                }
                self.lower_local_fn(decl, memo, into, out);
            }
            // Top-level declarations were handled by `collect_all`.
            ast::StmtKind::Fn(decl) => {
                if let Some(table) = stmt.memo_into().map(|(table, _)| table) {
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
            ast::StmtKind::Struct(decl) if !self.at_top_level() => {
                let id = self.structs.len();
                let sname = format!("_local_{}_{}", decl.name.text, id);
                let fields: Vec<(String, Ty)> = decl
                    .fields
                    .iter()
                    .map(|f| (f.name.text.clone(), self.resolve_ty(&f.ty)))
                    .collect();
                self.structs.push(StructInfo {
                    def: StructDef {
                        name: sname,
                        fields,
                        ..Default::default()
                    },
                    decl: None,
                    mod_id: self.cur_mod,
                    offset: self.offset,
                    origin_gen: None,
                    env: HashMap::new(),
                    methods: HashMap::new(),
                    trait_methods: HashMap::new(),
                    op_methods: Vec::new(),
                    gen_methods: HashMap::new(),
                    open_impls: Vec::new(),
                    getters: HashSet::new(),
                    projections: HashMap::new(),
                    consts: HashMap::new(),
                    owned_domains: HashMap::new(),
                    owned_domain_inits: Vec::new(),
                    laws: Vec::new(),
                    field_ranges: Vec::new(),
                    index_domain: None,
                });
                let scope = self.cx.local_syms.top_mut();
                scope.insert(decl.name.text.clone(), Sym::Struct(id));
            }
            ast::StmtKind::Struct(_)
            | ast::StmtKind::TypeAlias { .. }
            | ast::StmtKind::Trait { .. }
            | ast::StmtKind::Mod { .. }
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
                        (pair, None),
                        body,
                        else_body.as_ref(),
                        out,
                    );
                    return;
                }
                // `for r in V.ranges`: by length, then left end, so every sub-piece comes first.
                if let Some(dom_id) = self.domain_ranges_iter(iter) {
                    let span = iter.span;
                    let n = self.mk(ExprKind::DomainLen(dom_id), Ty::Int, span);
                    let (len_slot, lo_slot) = (self.temp(Ty::Int), self.temp(Ty::Int));
                    let len_var = self.mk(ExprKind::Local(len_slot), Ty::Int, span);
                    let lo_var = self.mk(ExprKind::Local(lo_slot), Ty::Int, span);
                    let n1 = self.int_add(n, 1);
                    let lo_end = BinOp::Sub;
                    let lo_end = self.mk_binary(lo_end, n1.clone(), len_var.clone(), Ty::Int, span);
                    let heads = vec![
                        (len_slot, self.int_lit(1, span), n1),
                        (lo_slot, self.int_lit(0, span), lo_end),
                    ];
                    let hi = BinOp::Add;
                    let hi = self.mk_binary(hi, lo_var.clone(), len_var, Ty::Int, span);
                    let piece = ExprKind::Range {
                        lo: Box::new(lo_var),
                        hi: Box::new(hi),
                    };
                    let piece = self.mk(piece, Ty::range(Ty::Pos(dom_id)), span);
                    let (label, else_body) = (label.as_ref(), else_body.as_ref());
                    let piece = (piece, Some(RangeInfo::Piece));
                    self.nested_for(label, heads, pat, piece, body, else_body, out);
                    return;
                }
                if let Some((heads, cell, parts)) = self.product_name_heads(pat, iter, out) {
                    self.cx.loop_parts = Some(parts);
                    let (label, else_body) = (label.as_ref(), else_body.as_ref());
                    self.nested_for(label, heads, pat, (cell, None), body, else_body, out);
                    return;
                }
                if let Some((heads, cell)) = self.product_heads(pat, iter) {
                    self.nested_for(
                        label.as_ref(),
                        heads,
                        pat,
                        (cell, None),
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
                    (Some(cells), _, _) => (cells.range().into(), Ty::Unit),
                    (None, Some(items), _) => (items.range().into(), Ty::Unit),
                    (None, None, Some(proj)) => (proj.iter().into(), Ty::Unit),
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
                // An element of a range-typed list keeps its range, so storing it back
                // into a cell of that range needs no check.
                if let (ast::PatternKind::Name(_), Head::Iter(Iter::Seq(seq, _))) =
                    (&pat.kind, &iter)
                {
                    if let Some(RangeInfo::Elem(inner)) = self.expr_range(seq) {
                        self.cx.local_ranges.insert(slot, *inner);
                    }
                }
                if let (ast::PatternKind::Name(_), true) = (&pat.kind, elem_is_int) {
                    if let Some(r) = self.loop_var_range(&iter) {
                        self.cx.local_ranges.insert(slot, r);
                    }
                }
                if elem_is_int
                    && body
                        .stmts
                        .iter()
                        .any(|s| matches!(s.kind, ast::StmtKind::TypeAlias { .. }))
                {
                    if let ast::PatternKind::Name(vname) = &pat.kind {
                        self.mirror_local_int(vname, pat.span, &mut stmts);
                    }
                }
                stmts.extend(self.block(body, Tail::Discard).stmts);
                self.cx.local_syms.pop();
                self.cx.scopes.pop();
                self.cx.loops.pop();
                let iter = match iter {
                    Head::Iter(iter) => Head::Iter(loops::hoist_call_args(iter, out)),
                    walk => walk,
                };
                self.push_for(out, id, slot, iter, Block { stmts });
                self.end_loop_else(id, else_flag, else_body.as_ref(), out);
            }
            ast::StmtKind::Return(value) => {
                if self.cx.dos.iter().any(|frame| !frame.open) {
                    self.report(
                        stmt.span,
                        msgs::check::LEAVES_A_PART_OF_A_VALUE,
                        &[&"return"],
                    );
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
                        let mut value = self.ret_value(e, hint.as_ref());
                        if matches!(hint, Some(Ty::Unit)) && matches!(value.ty, Ty::Unit) {
                            out.push(Stmt::Expr(value));
                            out.push(Stmt::Return(None));
                            return;
                        }
                        match hint {
                            Some(ty) => {
                                value = self.coerce(value, &ty, e.span);
                            }
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
                        Some(label) => {
                            cpl_diag::msgs::ir::NO_ENCLOSING_LOOP_LABELED.at(&[&label.text])
                        }
                        None => cpl_diag::msgs::ir::OUTSIDE_LOOP.at(&[&word]),
                    };
                    self.report_said(stmt.span, message);
                    return;
                };
                if self.cx.dos.last().is_some_and(|frame| index < frame.loops) {
                    self.report(stmt.span, msgs::check::LEAVES_A_PART_OF_A_VALUE, &[&word]);
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
            ast::StmtKind::Yield { value, from } => self.yield_stmt(value, *from, stmt.span, out),
            ast::StmtKind::Assert { cond, message } => {
                let (mut text, line, note) = self.assert_place(cond.span);
                let span = self.span(stmt.span);
                let key_test = self.key_test_report(cond);
                let checks = if key_test.is_some() {
                    "bounds"
                } else {
                    "asserts"
                };
                let cond_ast = cond;
                let cond = self.expr_as(cond, &Ty::Bool);
                let mut message = message.as_ref().map(|m| self.expr_as(m, &Ty::Str));
                if key_test.is_some() {
                    (text, message) = (String::new(), key_test);
                } else {
                    let values = self.assert_values(cond_ast);
                    message = self.with_values(message, values);
                }
                let kept = self
                    .context("checks", checks)
                    .is_some_and(|v| matches!(&v.kind, ast::ExprKind::Name(n) if n == "on"));
                out.push(Stmt::Assert {
                    cond,
                    message,
                    text,
                    line,
                    note,
                    span,
                    kept,
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
                        self.expr_as(d, &fty)
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
                    self.context_defs
                        .insert(name.text.clone(), (gid, field_tys));
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
                        out.push_let(save, self.mk(ExprKind::Global(gid), io_ty.clone(), span));
                        saved_ctx_slot = Some((gid, save));
                        let fields = match &io_ty {
                            Ty::Record(fields) => fields.clone(),
                            _ => Default::default(),
                        };
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
                            if let Some((idx, (_, fty))) = fields
                                .iter()
                                .enumerate()
                                .find(|(_, (fnm, _))| fnm == &k.text)
                            {
                                let val = self.expr_as(v, fty);
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
                            out.push_let(
                                save,
                                self.mk(ExprKind::Global(gid), ctx_ty.clone(), span),
                            );
                            saved_ctx_slot = Some((gid, save));
                            for arg in args {
                                let Some(k) = &arg.name else { continue };
                                let v = &arg.value;
                                if let Some((idx, (_, fty))) = fields
                                    .iter()
                                    .enumerate()
                                    .find(|(_, (fnm, _))| fnm == &k.text)
                                {
                                    let val = self.expr_as(v, fty);
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
        self.file_contexts_len = self.contexts.len();
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
        self.gen_env.contains_key(instance::IO_BASE_ONE)
            || self
                .context("io", "base")
                .is_some_and(|v| matches!(v.kind, ast::ExprKind::Int(1)))
    }

    /// Binds the names in `pat` to `value`.
    pub(super) fn bind(&mut self, pat: &ast::Pattern, value: Expr, out: &mut Vec<Stmt>) {
        self.bind_mut(pat, value, false, out);
    }
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

pub(super) fn remap_domain_ty(ty: &Ty, from: DomainId, to: DomainId) -> Ty {
    if from == to {
        return ty.clone();
    }
    let map_d = |d: DomainId| if d == from { to } else { d };
    match ty {
        Ty::Pos(d) => Ty::Pos(map_d(*d)),
        Ty::Bound(d) => Ty::Bound(map_d(*d)),
        Ty::Range(inner) => Ty::range(remap_domain_ty(inner, from, to)),
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
        Ty::List(e) => Ty::list(remap_domain_ty(e, from, to)),
        Ty::Opt(e) => Ty::opt(remap_domain_ty(e, from, to)),
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

fn leaves(stmt: &ast::Stmt) -> bool {
    matches!(
        stmt.kind,
        ast::StmtKind::Return(_) | ast::StmtKind::Break(_) | ast::StmtKind::Continue(_)
    )
}

/// Whether `e` is a `match`, an `if` with an `else`, or a block with a
/// `return`, `break` or `continue` that leaves it: as a statement's whole
/// value its arms then run as statements.
fn has_leaving_arm(e: &ast::Expr) -> bool {
    matches!(
        e.kind,
        ast::ExprKind::Match { .. }
            | ast::ExprKind::If {
                otherwise: Some(_),
                ..
            }
            | ast::ExprKind::Block(_)
    ) && !cpl_syntax::visit::leaves_out_of(e).is_empty()
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

pub(super) fn plural(n: i64, what: &str) -> String {
    format!("{n} {what}{}", if n == 1 { "" } else { "s" })
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
        ast::PatternKind::Record { fields, .. } => fields.iter().any(|(_, p)| refutable(p)),
        ast::PatternKind::Or(alts) => alts.iter().all(refutable),
        _ => false,
    }
}

/// `stmts` up to the last one that names `name`: a lend through `name` ends there.
fn until_last_use<'s>(stmts: &'s [ast::Stmt], name: &str) -> &'s [ast::Stmt] {
    struct Names<'n>(&'n str, bool);
    impl cpl_syntax::visit::Visitor for Names<'_> {
        fn expr(&mut self, e: &ast::Expr) {
            if matches!(&e.kind, ast::ExprKind::Name(n) if n == self.0) {
                self.1 = true;
            }
        }
    }
    let last = stmts.iter().rposition(|s| {
        let mut names = Names(name, false);
        cpl_syntax::visit::walk_stmt(s, &mut names);
        names.1
    });
    &stmts[..last.map_or(0, |i| i + 1)]
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
