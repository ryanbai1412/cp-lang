//! Fn declarations: the signature and body of a `fn`, `@memo` and
//! `@cache` and what they may capture or lend.

use super::*;
use cpl_diag::msgs;

impl<'a> Checker<'a> {
    /// Reports a `@memo` that would not remember anything. Both back ends keep one table of
    /// answers per top-level or local fn, keyed by its arguments, and run the fn as written
    /// when they can't: the program would be right and, silently, as slow as without `@memo`.
    pub(super) fn is_bounded_memo_type(&mut self, ty: &Ty) -> bool {
        match ty {
            Ty::Pos(_) | Ty::Bool | Ty::Range { .. } | Ty::Error | Ty::Unknown => true,
            Ty::Named { name, .. } => {
                name == "Cut" || matches!(self.type_def(name).as_deref(), Some(Def::Enum { .. }))
            }
            Ty::Tuple(ts) => ts.iter().all(|t| self.is_bounded_memo_type(t)),
            Ty::Optional(inner) => self.is_bounded_memo_type(inner),
            _ => false,
        }
    }

    /// Outer `var`s referenced inside a `@memo` fn body.
    pub(super) fn memo_captured_vars(&self, f: &ast::FnDecl) -> Vec<String> {
        let mut param_names = crate::fx::HashSet::default();
        for p in &f.params {
            param_names.insert(p.name.text.clone());
        }
        let mut names = Vec::new();
        match &f.body {
            FnBody::Expr(e) => names.extend(super::lends::names_in(e)),
            FnBody::Block(b) => {
                for s in &b.stmts {
                    let (ns, _) = super::lends::item_names(std::slice::from_ref(s));
                    names.extend(ns);
                }
            }
            FnBody::None => {}
        }
        let mut out = Vec::new();
        for n in names {
            if param_names.contains(&n) || out.contains(&n) {
                continue;
            }
            if self.local(&n).is_some_and(|l| l.mutable) {
                out.push(n);
            }
        }
        out
    }

    pub(crate) fn check_memo_capture_write(&mut self, root: &str, span: Span) {
        if let Some((_, fn_name, memo_span)) = self
            .memo_captured
            .iter()
            .rev()
            .find(|(v, _, _)| v == root)
            .cloned()
        {
            let message = cpl_diag::msgs::check::CAN_T_MODIFIED_WHILE.at(&[&root, &fn_name]);
            if let Some(diag) = self.report_said(span, message) {
                diag.labels.push(cpl_diag::Label {
                    span: memo_span,
                    message: format!("`{fn_name}` is memoized here and reads `{root}`"),
                });
                diag.help = Some(format!(
                    "pass `{root}` as a parameter to `{fn_name}`, or finish updating `{root}` before declaring `{fn_name}`"
                ));
            }
        }
    }

    /// `memo` is the `@memo` and whether it is `@memo(array)`.
    pub(super) fn memo_fits(
        &mut self,
        f: &ast::FnDecl,
        params: &[crate::Param],
        self_ty: Option<&Ty>,
        generator: bool,
        unit: bool,
        (memo, is_array): (Span, bool),
    ) {
        let name = &f.name.text;
        let changed = f.params.iter().find(|p| p.mutable && p.name.text != "self");
        if is_array {
            for (ast_p, ty_p) in f.params.iter().zip(params.iter()) {
                if !self.is_bounded_memo_type(&ty_p.ty) {
                    let pname = &ast_p.name.text;
                    let pty = &ty_p.ty;
                    if let Some(diag) = self.report(
                        ast_p.name.span,
                        msgs::check::BAD_MEMO_MEMO_ARRAY_ON_REQUIRES,
                        &[&name, &pname, &pty],
                    ) {
                        diag.help = Some(
                            "give the parameter a bounded type (a domain, range, `Bool`, or enum), or use `@memo(map)`".into(),
                        );
                    }
                    return;
                }
            }
        }
        let (message, help) = if let Some(owner) = self_ty {
            let owner = owner.to_string();
            (
                cpl_diag::msgs::check::MEMO_FN_OF.at(&[&name, &owner]),
                format!(
                    "only a top-level or local fn keeps a table of its answers: move `{name}` out of `{owner}` and pass it what it reads"
                ),
            )
        } else if let Some(param) = changed {
            let param = &param.name.text;
            (
                cpl_diag::msgs::check::MEMO_CHANGES_ARGUMENT.at(&[&name, &param]),
                format!(
                    "a remembered answer is given back without running the fn, so `{param}` would be left as it was: take `{param}` without `mut`, or drop `@memo`"
                ),
            )
        } else if generator {
            (
                cpl_diag::msgs::check::MEMO_YIELDS.at(&[&name]),
                "collect the values into a list and return that: `-> List<T>`".to_string(),
            )
        } else if !f.generics.is_empty() {
            (
                cpl_diag::msgs::check::MEMO_GENERIC.at(&[&name]),
                format!(
                    "there is one table of answers per fn, and `{name}` is a different fn for each type it is called with: write it for the type it is used with"
                ),
            )
        } else if unit {
            (
                cpl_diag::msgs::check::MEMO_NO_VALUE.at(&[&name]),
                "a fn that only acts has to run every time: drop `@memo`, or return what it computes".to_string(),
            )
        } else {
            return;
        };
        if let Some(diag) = self.report_said(memo, message) {
            diag.help = Some(help);
        }
    }

    /// `memo` is where the fn's `@memo` is written, when it has one, and whether it is `@memo(array)`.
    pub(super) fn fn_decl(
        &mut self,
        f: &ast::FnDecl,
        self_ty: Option<&Ty>,
        local: bool,
        memo: Option<(Span, bool)>,
    ) {
        if let Some(lends) = &f.lends {
            let mut returned = Vec::new();
            match &f.body {
                ast::FnBody::Expr(e) => returned.push(e),
                ast::FnBody::Block(b) => Self::collect_returned_exprs(b, &mut returned),
                ast::FnBody::None => {}
            }
            if !lends.sources.is_empty() {
                for e in &returned {
                    let root = Self::expr_root_name(e);
                    let ok = root.is_some_and(|r| lends.sources.iter().any(|s| s.text == r));
                    if !ok {
                        self.report(
                            e.span,
                            msgs::check::LENT_ESCAPES_LENT_VALUE_MUST_COME,
                            &[&lends
                                .sources
                                .iter()
                                .map(|s| s.text.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")],
                        );
                    }
                }
            }
            if let Some(Ty::Named { name: sname, .. }) = self_ty {
                let cached = match self.type_def(sname).as_deref() {
                    Some(Def::Struct { cached_fields, .. }) => cached_fields.clone(),
                    _ => Vec::new(),
                };
                if !cached.is_empty() {
                    for e in &returned {
                        if let Some(cf) = Self::expr_self_field(e) {
                            if cached.iter().any(|c| c == cf) {
                                self.report(
                                    e.span,
                                    msgs::check::BAD_PROJECTION_CANNOT_LEND_CACHE_FIELD,
                                    &[&cf, &sname],
                                );
                            }
                        }
                    }
                }
            }
        }
        let saved_row_bounds = self.row_bounds.clone();
        let outer_for_row = self.generics.len();
        self.generics
            .extend(f.generics.iter().map(|g| g.text.clone()));
        for b in &f.bounds {
            for t in &b.traits {
                match &t.kind {
                    ast::TypeKind::Row(fields) => {
                        let resolved = fields
                            .iter()
                            .map(|(n, ty)| (n.text.clone(), self.resolve_type(ty)))
                            .collect();
                        self.row_bounds.insert(b.name.text.clone(), resolved);
                    }
                    ast::TypeKind::Named { name: tname, .. } => {
                        if let Some(Def::Trait { fields, .. }) =
                            self.global(&tname.text).as_deref().cloned()
                        {
                            if !fields.is_empty() {
                                self.row_bounds
                                    .entry(b.name.text.clone())
                                    .or_default()
                                    .extend(fields);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        self.generics.truncate(outer_for_row);
        let sig = self.fn_sig(f, self_ty);
        let Ty::Fn { params, ret } = &sig else {
            self.row_bounds = saved_row_bounds;
            return;
        };
        if local {
            self.declare(&f.name.text, f.name.span, sig.clone(), false, "fn");
        } else {
            self.hovers.push(HoverEntry {
                span: f.name.span,
                name: f.name.text.clone(),
                ty: sig.clone(),
                keyword: "fn",
                decl: Some(f.name.span),
                owner: None,
            });
        }
        for ctx in &f.uses {
            if !matches!(ctx.text.as_str(), "checks" | "lint")
                && self.context_record_fields(&ctx.text).is_none()
            {
                self.report(
                    ctx.span,
                    msgs::check::UNRESOLVED_NAME_UNKNOWN_CONTEXT_IN_USES,
                    &[&ctx.text],
                );
            }
        }
        let outer = self.generics.len();
        self.generics
            .extend(f.generics.iter().map(|g| g.text.clone()));
        for g in &f.generics {
            self.note_generic(&g.text, g.span, Some(g.span));
        }
        let outer_bounds = self.enter_fn_bounds(f);
        self.in_fn += 1;
        // A move is a fact about one body: a later fn may name a field or global again.
        let outer_moved = std::mem::take(&mut self.moved);
        self.push_scope();
        // Inside a method the struct's fields can be named without `self.`.
        let receiver = f.params.iter().find(|p| p.name.text == "self");
        if let (Some(receiver), Some(Ty::Named { name, .. })) = (receiver, self_ty) {
            if let Some(Def::Struct { fields, .. }) = self.global(name).as_deref() {
                for field in fields {
                    self.scopes.innermost().insert(
                        field.name.clone(),
                        Local {
                            ty: field.ty.clone(),
                            mutable: receiver.mutable,
                            span: receiver.name.span,
                            keyword: if receiver.mutable { "field" } else { "self" },
                        },
                    );
                }
            }
            self.push_scope();
        }
        if let Some(lends) = &f.lends {
            self.resolve_type(&lends.ty);
            if let Some(at) = lends.bare {
                if let Some(diag) =
                    self.report(at, msgs::check::EXPECTED_TOKEN_EXPECTED_BEFORE_LENDS_A, &[])
                {
                    diag.help = Some("write `-> lends T`".to_string());
                    diag.fixes.push(cpl_diag::Fix {
                        title: "insert `->`".to_string(),
                        edits: vec![(Span::new(at.start, at.start), "-> ".to_string())],
                    });
                }
            }
        }
        let mut sig_params = params.iter();
        for param in &f.params {
            if self_ty.is_some() && param.name.text == "self" {
                let ty = self_ty.cloned().unwrap_or(Ty::Unknown);
                self.declare(
                    "self",
                    param.name.span,
                    ty,
                    param.mutable,
                    if param.mutable { "param" } else { "self" },
                );
                continue;
            }
            let ty = sig_params.next().map_or(Ty::Unknown, |p| p.ty.clone());
            if let Some(default) = &param.default {
                let default_ty = self.infer(default);
                if !self.array_fills(&ty, &default_ty, default) {
                    self.expect_fits(&default_ty, &ty, default, &|c, got, want| {
                        cpl_diag::msgs::check::BUT_DEFAULT.at(&[
                            &param.name.text,
                            &want.with_article(),
                            &c.quote(default.span),
                            &got.with_article(),
                        ])
                    });
                }
            }
            if let Some(ast::Type {
                kind: ast::TypeKind::LiteralUnion(lits),
                ..
            }) = &param.ty
            {
                self.lit_unions
                    .insert(param.name.text.clone(), lits.clone());
            }
            self.declare(
                &param.name.text,
                param.name.span,
                ty,
                param.mutable,
                "param",
            );
        }
        // A generator's body yields its values; it has no value of its own.
        let generator = matches!(
            f.ret.as_ref().map(|t| &t.kind),
            Some(ast::TypeKind::Yields(_))
        );
        if self.scopes.len() > 1 {
            self.local_effects
                .insert(f.name.text.clone(), super::lends::own_effects(f));
        }
        self.note_ret_slot(f);
        let fn_depth = self.scopes.len().saturating_sub(1);
        self.fns.push((
            f.name.text.clone(),
            (**ret).clone(),
            f.ret.is_some() && !generator,
            fn_depth,
        ));
        self.fn_lends.push(f.lends.is_some());
        self.gives.push(false);
        // A fn body knows nothing of what is lent or known where the fn
        // is written: it runs when it is called.
        self.lend_isolated(|c| {
            if self_ty.is_some() {
                let fields: Vec<String> = match self_ty {
                    Some(Ty::Named { name, .. }) => c
                        .members_of(name)
                        .map(|m| m.fields.iter().map(|f| f.name.clone()).collect())
                        .unwrap_or_default(),
                    _ => Vec::new(),
                };
                c.projection_shape(f, &fields);
            }
            match &f.body {
                FnBody::Block(block) => {
                    c.push_scope();
                    let mark = c.lend_mark();
                    let ty = c.stmts(&block.stmts, false);
                    // The last expression is the fn's value. Check returns before
                    // popping the fn body scope so local domain declarations are
                    // still visible for escape checking.
                    if let Some(Stmt {
                        kind: StmtKind::Expr(last),
                        ..
                    }) = block.stmts.last()
                    {
                        // An `if` or `match` whose branches all `return` has no
                        // value; the returns were checked on their own.
                        let statement_like = ty == Ty::Unit
                            && matches!(last.kind, ExprKind::If { .. } | ExprKind::Match { .. });
                        if !statement_like {
                            c.fixed_read(last, ret);
                            c.returns(last, &ty);
                            if !c.fn_lends.last().copied().unwrap_or(false) {
                                c.no_escape(last, "returning it");
                            }
                            c.no_escape_return(last, "returning it");
                        }
                    }
                    c.lend_reset(mark);
                    c.pop_scope();
                }
                FnBody::Expr(expr)
                    if !matches!(expr.kind, ExprKind::Cpp(_) | ExprKind::Intrinsic(_)) =>
                {
                    if matches!(expr.kind, ExprKind::If { .. } | ExprKind::Match { .. }) {
                        c.plain_if = Some(expr.span);
                    }
                    let ty = c.infer(expr);
                    c.fixed_read(expr, ret);
                    c.returns(expr, &ty);
                    if !c.fn_lends.last().copied().unwrap_or(false) {
                        c.no_escape(expr, "returning it");
                    }
                    c.no_escape_return(expr, "returning it");
                }
                FnBody::Expr(Expr {
                    kind: ExprKind::Intrinsic(name),
                    span,
                }) => c.intrinsic_body(f, name, *span, ret),
                // An `@cpp` body is C++: only the signature is checked.
                FnBody::Expr(_) | FnBody::None => {}
            }
        });
        self.fns.pop();
        self.fn_lends.pop();
        let gives = self.gives.pop().unwrap_or(false);
        let memo_fn = memo.is_some();
        if let Some((memo, is_array)) = memo {
            // Written `-> T`, or read off what the body gives.
            let unit = match &f.ret {
                Some(_) => **ret == Ty::Unit,
                None => !gives && !matches!(f.body, FnBody::None),
            };
            self.memo_fits(f, params, self_ty, generator, unit, (memo, is_array));
        }
        // `@memo` refuses a `mut` parameter itself (E0311).
        let plain = self_ty.is_none() && !memo_fn && !matches!(f.body, FnBody::None);
        self.after_fn_body(f, ret, generator, plain);
        self.pop_scope();
        if receiver.is_some() && matches!(self_ty, Some(Ty::Named { .. })) {
            self.pop_scope();
        }
        self.in_fn -= 1;
        self.moved = outer_moved;
        self.generics.truncate(outer);
        self.leave_bounds(outer_bounds);
        self.row_bounds = saved_row_bounds;
    }
}

/// Where a fn's `@memo` is written, and whether it specifies `@memo(array)`.
pub(super) fn memo_of(stmt: &Stmt) -> Option<(Span, bool)> {
    let memo = stmt.decorators.iter().find(|d| d.name.text == "memo")?;
    let is_array = memo
        .args
        .iter()
        .any(|a| a.name.is_none() && matches!(&a.value.kind, ExprKind::Name(n) if n == "array"));
    Some((memo.span, is_array))
}

impl<'a> Checker<'a> {
    pub(super) fn collect_returned_exprs<'b>(block: &'b ast::Block, out: &mut Vec<&'b Expr>) {
        for (i, stmt) in block.stmts.iter().enumerate() {
            match &stmt.kind {
                StmtKind::Return(Some(e)) | StmtKind::Lend { place: e, .. } => out.push(e),
                StmtKind::Expr(e) if i + 1 == block.stmts.len() => out.push(e),
                StmtKind::Expr(Expr {
                    kind:
                        ExprKind::If {
                            branches,
                            otherwise,
                        },
                    ..
                }) => {
                    for (_, b) in branches {
                        Self::collect_returned_exprs(b, out);
                    }
                    if let Some(b) = otherwise {
                        Self::collect_returned_exprs(b, out);
                    }
                }
                _ => {}
            }
        }
    }

    pub(super) fn expr_root_name(mut e: &Expr) -> Option<&str> {
        loop {
            match &e.kind {
                ExprKind::Name(n) => return Some(n.as_str()),
                ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => e = base,
                ExprKind::Paren(inner) => e = inner,
                _ => return None,
            }
        }
    }

    pub(super) fn expr_self_field(mut e: &Expr) -> Option<&str> {
        loop {
            match &e.kind {
                ExprKind::Field { base, name } if matches!(&base.kind, ExprKind::SelfValue) => {
                    return Some(name.text.as_str());
                }
                ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => e = base,
                ExprKind::Paren(inner) => e = inner,
                _ => return None,
            }
        }
    }

    pub(super) fn check_cache_lend_body(&mut self, block: &ast::Block, cache_field: &str) {
        for stmt in &block.stmts {
            self.check_cache_lend_stmt(stmt, cache_field);
        }
    }

    pub(super) fn check_cache_lend_stmt(&mut self, stmt: &Stmt, cache_field: &str) {
        match &stmt.kind {
            StmtKind::Expr(e) | StmtKind::Return(Some(e)) => {
                self.check_cache_lend_expr(e, cache_field);
            }
            StmtKind::Let { init: Some(e), .. } => self.check_cache_lend_expr(e, cache_field),
            StmtKind::Assign { target, value, .. } => {
                self.check_cache_lend_expr(target, cache_field);
                self.check_cache_lend_expr(value, cache_field);
            }
            StmtKind::For { iter, body, .. } => {
                self.check_cache_lend_expr(iter, cache_field);
                self.check_cache_lend_body(body, cache_field);
            }
            StmtKind::While { body, .. } => {
                self.check_cache_lend_body(body, cache_field);
            }
            _ => {}
        }
    }

    pub(super) fn check_cache_lend_expr(&mut self, expr: &Expr, cache_field: &str) {
        match &expr.kind {
            ExprKind::Call { callee, args } => {
                let bad_self_method = matches!(
                    &callee.kind,
                    ExprKind::Field { base, .. }
                        if matches!(&base.kind, ExprKind::SelfValue)
                );
                let bad_fn_param = match &callee.kind {
                    ExprKind::Name(n) => {
                        self.local(n).is_some_and(|l| matches!(l.ty, Ty::Fn { .. }))
                    }
                    _ => false,
                };
                if bad_self_method || bad_fn_param {
                    self.report(
                        expr.span,
                        msgs::check::USED_WHILE_LENT_CANNOT_CALL_WHILE_CACHE,
                        &[&self.quote(callee.span), &cache_field],
                    );
                }
                self.check_cache_lend_expr(callee, cache_field);
                for a in args {
                    self.check_cache_lend_expr(&a.value, cache_field);
                }
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                self.check_cache_lend_expr(lhs, cache_field);
                self.check_cache_lend_expr(rhs, cache_field);
            }
            ExprKind::Compare { first, rest } => {
                self.check_cache_lend_expr(first, cache_field);
                for (_, e) in rest {
                    self.check_cache_lend_expr(e, cache_field);
                }
            }
            ExprKind::Unary { expr: inner, .. }
            | ExprKind::Paren(inner)
            | ExprKind::Try(inner)
            | ExprKind::Unwrap(inner) => self.check_cache_lend_expr(inner, cache_field),
            ExprKind::If {
                branches,
                otherwise,
            } => {
                for (c, b) in branches {
                    if let ast::Cond::Expr(e) | ast::Cond::Const(e) = c {
                        self.check_cache_lend_expr(e, cache_field);
                    }
                    self.check_cache_lend_body(b, cache_field);
                }
                if let Some(b) = otherwise {
                    self.check_cache_lend_body(b, cache_field);
                }
            }
            _ => {}
        }
    }
}

impl Checker<'_> {
    /// A hover for a fn's type parameter, at its declaration or a use.
    pub(crate) fn note_generic(&mut self, name: &str, span: Span, decl: Option<Span>) {
        self.hovers.push(HoverEntry {
            span,
            name: name.to_string(),
            ty: Ty::Param(name.to_string()),
            keyword: "generic",
            decl,
            owner: None,
        });
    }
}
