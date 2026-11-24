//! Lazy `xs.map(f)` views (R07.4.9): a `let` bound to one calls `f` on each read,
//! and a top-level one keeps its base in a global the file's fns read too.

use super::*;

impl<'a> Lowerer<'a> {
    /// `let name = xs.map(f)` as a lazy view, which calls `f` on every read (R07.4.9); only an
    /// effect-free `f` (see [`effect_free_lambda`]) may be mapped once when `rest` reads it whole.
    pub(in crate::lower) fn map_view_if_indexed(
        &mut self,
        expr: &ast::Expr,
        name: &str,
        rest: &[ast::Stmt],
        out: &mut Vec<Stmt>,
    ) -> Option<MapView> {
        let effect_free = matches!(&expr.kind, ast::ExprKind::Call { args, .. }
            if args.len() == 1 && effect_free_lambda(&args[0].value));
        if effect_free && !only_indexed(name, rest) {
            return None;
        }
        let global = self.cx.is_main && self.cx.scopes.len() == 1;
        self.map_view_from_expr(expr, global.then_some(name), out)
    }

    /// The map view `name` reads, if it names one: a local's, or a top-level `let`'s.
    pub(in crate::lower) fn map_view_named(&self, name: &str) -> Option<MapView> {
        match self.lookup(name) {
            Some(slot) => self.cx.map_views.get(&slot).cloned(),
            None => self
                .global_map_views
                .get(&(self.cur_mod, name.to_string()))
                .cloned(),
        }
    }

    fn map_view_base(&mut self, view: &MapView, span: Span) -> Expr {
        match view.base {
            ViewBase::Local(slot) => self.local(slot, span),
            ViewBase::Global(gid) => self.global_read(gid, span),
        }
    }

    /// `m.rev.take(k)` on a map view `m = xs.map(f)` is `xs.rev.take(k).map(f)`, so `f` runs
    /// only on what the pieces keep, and `then` (`.len`) reads none of it (R07.4.9). A piece is
    /// any library fn that returns a `Slice` or `Rev` of `self`.
    pub(in crate::lower) fn map_view_piece(
        &mut self,
        e: &ast::Expr,
        then: Option<&ast::Name>,
    ) -> Option<Expr> {
        if then.is_none() && matches!(e.kind, ast::ExprKind::Name(_)) {
            return None;
        }
        let hidden = "\u{0}map view base";
        let (mut chain, view) = self.map_view_chain(e, hidden)?;
        if let Some(name) = then {
            chain = ast::Expr {
                kind: ast::ExprKind::Field {
                    base: Box::new(chain),
                    name: name.clone(),
                },
                span: e.span,
            };
        }
        let piece = match view.base {
            ViewBase::Local(slot) => {
                self.cx
                    .scopes
                    .push(HashMap::from([(hidden.to_string(), slot)]));
                let piece = self.expr(&chain, None);
                self.cx.scopes.pop();
                piece
            }
            ViewBase::Global(gid) => {
                let syms = &mut self.mod_syms[self.cur_mod];
                syms.insert(hidden.to_string(), Sym::Global(gid));
                let piece = self.expr(&chain, None);
                self.mod_syms[self.cur_mod].remove(hidden);
                piece
            }
        };
        if then.is_some() {
            return Some(piece);
        }
        let elem_ty = match &piece.ty {
            Ty::List(e) | Ty::ListView(e, _) | Ty::MutView(e, _) | Ty::Keyed(e, _) => (**e).clone(),
            Ty::Str | Ty::StrView => Ty::Char,
            _ => return None,
        };
        Some(self.map_seq(piece, elem_ty, &view.func, e.span))
    }

    /// `e` with the map view at its root renamed to `hidden`, when every step is a piece.
    fn map_view_chain(&mut self, e: &ast::Expr, hidden: &str) -> Option<(ast::Expr, MapView)> {
        let mut out = e.clone();
        let base = match &mut out.kind {
            ast::ExprKind::Name(n) => {
                let view = self.map_view_named(n)?;
                *n = hidden.to_string();
                return Some((out, view));
            }
            ast::ExprKind::Paren(inner) => return self.map_view_chain(inner, hidden),
            ast::ExprKind::Call { callee, .. } => match &mut callee.kind {
                ast::ExprKind::Field { base, name } if self.is_piece(&name.text) => base,
                _ => return None,
            },
            ast::ExprKind::Field { base, name } if self.is_piece(&name.text) => base,
            _ => return None,
        };
        let (inner, view) = self.map_view_chain(base, hidden)?;
        **base = inner;
        Some((out, view))
    }

    fn is_piece(&self, method: &str) -> bool {
        (self.method_syms(method).iter())
            .any(|sym| matches!(sym, Sym::GenFn(gfid) if piece_of_self(self.gen_fns[*gfid].decl)))
    }

    pub(in crate::lower) fn map_view_from_expr(
        &mut self,
        expr: &ast::Expr,
        global: Option<&str>,
        out: &mut Vec<Stmt>,
    ) -> Option<MapView> {
        let ast::ExprKind::Call { callee, args } = &expr.kind else {
            return None;
        };
        let ast::ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        if name.text != "map" || args.len() != 1 || args[0].name.is_some() || args[0].mutable {
            return None;
        }
        let base_ty = self.probe_ty_silent(base)?;
        if !matches!(
            base_ty,
            Ty::List(_) | Ty::Array { .. } | Ty::ListView(..) | Ty::MutView(..) | Ty::Str
        ) {
            return None;
        }
        let base_val = self.expr(base, None);
        if matches!(base_val.ty, Ty::Error) {
            return None;
        }
        let base = match global {
            Some(name) => {
                let gid = self.globals.len();
                self.globals.push(Global {
                    name: format!("_{name}_mapped_{gid}"),
                    ty: base_val.ty.clone(),
                    span: Span::default(),
                });
                let place = Place {
                    kind: PlaceKind::Global(gid),
                    ty: base_val.ty.clone(),
                    span: base_val.span,
                };
                out.push(Stmt::Assign(place, AssignOp::Set, base_val));
                ViewBase::Global(gid)
            }
            None => {
                let slot = self.temp(base_val.ty.clone());
                out.push(Stmt::Let(slot, Some(base_val)));
                ViewBase::Local(slot)
            }
        };
        Some(MapView {
            base,
            func: args[0].value.clone(),
            span: expr.span,
        })
    }

    pub(in crate::lower) fn lower_map_view_index(
        &mut self,
        base: &ast::Expr,
        indices: &[ast::Expr],
        span: Span,
    ) -> Option<Expr> {
        if let ast::ExprKind::Name(n) = &base.kind {
            if let Some(view) = self.map_view_named(n) {
                let base_expr = self.map_view_base(&view, span);
                let elem = self.mapped_elem(base_expr, indices, span);
                return Some(self.apply_map_fn(elem, &view.func, span));
            }
        }
        if let ast::ExprKind::Call { callee, args } = &base.kind {
            if let ast::ExprKind::Field { base: inner, name } = &callee.kind {
                if name.text == "map"
                    && args.len() == 1
                    && args[0].name.is_none()
                    && !args[0].mutable
                {
                    if let Some(inner_elem) = self.lower_map_view_index(inner, indices, span) {
                        return Some(self.apply_map_fn(inner_elem, &args[0].value, span));
                    }
                    if matches!(
                        self.probe_ty_silent(inner),
                        Some(
                            Ty::List(_)
                                | Ty::Array { .. }
                                | Ty::ListView(..)
                                | Ty::MutView(..)
                                | Ty::Str
                        )
                    ) {
                        let base_expr = self.expr(inner, None);
                        let elem = self.mapped_elem(base_expr, indices, span);
                        return Some(self.apply_map_fn(elem, &args[0].value, span));
                    }
                }
            }
        }

        None
    }

    pub(in crate::lower) fn apply_map_fn(
        &mut self,
        elem: Expr,
        func_ast: &ast::Expr,
        span: Span,
    ) -> Expr {
        if matches!(elem.ty, Ty::Error) {
            return self.err_expr(span);
        }
        if let ast::ExprKind::Lambda { params, body, .. } = &func_ast.kind {
            if params.len() == 1 {
                let mut out_ty = Ty::Error;
                let expr = self.do_expr(None, span, |lw, out| {
                    lw.cx.scopes.push(HashMap::new());
                    let mut bind_stmts = Vec::new();
                    let item = lw.loop_var(&params[0], elem.ty.clone(), &mut bind_stmts);
                    out.push(Stmt::Let(item, Some(elem)));
                    out.extend(bind_stmts);
                    let mapped = lw.expr(body, None);
                    out_ty = mapped.ty.clone();
                    lw.cx.scopes.pop();
                    out.push(Stmt::Give(mapped));
                });
                return Expr { ty: out_ty, ..expr };
            }
        }
        let fn_hint = Ty::Fn(vec![elem.ty.clone()], Box::new(Ty::Error));
        let func = self.expr(func_ast, Some(&fn_hint));
        let ret = match &func.ty {
            Ty::Fn(_, ret) if !matches!(ret.as_ref(), Ty::Error) => (**ret).clone(),
            Ty::Error => return self.err_expr(span),
            other => {
                self.error(
                    codes::TYPE_MISMATCH,
                    func_ast.span,
                    format!("`map` expects a function, not `{}`", self.show(other)),
                );
                return self.err_expr(span);
            }
        };
        self.mk(
            ExprKind::CallExpr(Box::new(func), vec![Arg::Value(elem)]),
            ret,
            span,
        )
    }

    pub(in crate::lower) fn lower_map_view_materialize(
        &mut self,
        view: &MapView,
        span: Span,
    ) -> Expr {
        let base_expr = self.map_view_base(view, view.span);
        if matches!(base_expr.ty, Ty::Array { .. }) {
            return self.lower_array_map_expr(base_expr, &view.func, span);
        }
        let elem_ty = match &base_expr.ty {
            Ty::List(e) | Ty::ListView(e, _) | Ty::MutView(e, _) => (**e).clone(),
            Ty::Str => Ty::Char,
            _ => return self.err_expr(span),
        };
        self.map_seq(base_expr, elem_ty, &view.func, span)
    }
}

/// Every read of `name` in `rest` is `name[i]`, `name.len` or `name.is_empty`, which a lazy
/// `MapView` answers in O(1); any other read (`.first`, a loop, a call) would map the whole
/// list again each time, so the `let` maps it once instead.
fn only_indexed(name: &str, rest: &[ast::Stmt]) -> bool {
    struct Reads<'n> {
        name: &'n str,
        all: usize,
        cheap: usize,
    }
    impl cpl_syntax::visit::Visitor for Reads<'_> {
        fn expr(&mut self, e: &ast::Expr) {
            let named = |b: &ast::Expr| matches!(&b.kind, ast::ExprKind::Name(n) if n == self.name);
            match &e.kind {
                ast::ExprKind::Name(n) if n == self.name => self.all += 1,
                ast::ExprKind::Index { base, .. } if named(base) => self.cheap += 1,
                ast::ExprKind::Field { base, name }
                    if named(base) && matches!(name.text.as_str(), "len" | "is_empty") =>
                {
                    self.cheap += 1
                }
                _ => {}
            }
        }
    }
    let mut reads = Reads {
        name,
        all: 0,
        cheap: 0,
    };
    for stmt in rest {
        cpl_syntax::visit::walk_stmt(stmt, &mut reads);
    }
    reads.all == reads.cheap
}

/// A lambda whose body has no calls, writes or I/O and cannot trap (no arithmetic, index or
/// `!`), so calling it once per element or once per read gives the same run.
fn effect_free_lambda(func: &ast::Expr) -> bool {
    struct Quiet(bool);
    impl cpl_syntax::visit::Visitor for Quiet {
        fn expr(&mut self, e: &ast::Expr) {
            self.0 &= matches!(
                e.kind,
                ast::ExprKind::Int(_)
                    | ast::ExprKind::Char(_)
                    | ast::ExprKind::Bool(_)
                    | ast::ExprKind::None
                    | ast::ExprKind::Name(_)
                    | ast::ExprKind::SelfValue
                    | ast::ExprKind::Paren(_)
                    | ast::ExprKind::Tuple(_)
                    | ast::ExprKind::Field { .. }
                    | ast::ExprKind::Compare { .. }
            );
        }
    }
    let ast::ExprKind::Lambda { body, .. } = &func.kind else {
        return false;
    };
    let mut quiet = Quiet(true);
    cpl_syntax::visit::walk_expr(body, &mut quiet);
    quiet.0
}

/// `fn drop<S: Indexed>(self: S, ..) -> Slice<S>`: a piece of the receiver, in some order.
fn piece_of_self(decl: &ast::FnDecl) -> bool {
    fn named(ty: Option<&ast::Type>) -> Option<(&str, &[ast::Type])> {
        match ty.map(|t| &t.kind) {
            Some(ast::TypeKind::Named { name, args }) => Some((name.text.as_str(), args)),
            _ => None,
        }
    }
    let Some(first) = decl.params.first().filter(|p| p.name.text == "self") else {
        return false;
    };
    let Some((s, _)) = named(first.ty.as_ref()) else {
        return false;
    };
    matches!(named(decl.ret.as_ref()), Some(("Slice" | "Rev", args))
        if args.len() == 1 && named(Some(&args[0])).is_some_and(|(a, _)| a == s)
            && decl.generics.iter().any(|g| g.text == s))
}
