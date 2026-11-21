//! Statements, blocks, and declarations.

use super::types::pattern_types;
use super::*;
use cpl_syntax::ast::{
    self, AssignOp, Block, Expr, ExprKind, FnBody, Pattern, PatternKind, Stmt, StmtKind,
};

impl<'a> Checker<'a> {
    /// Checks the statements of one top-level item.
    pub(crate) fn item(&mut self, stmts: &[Stmt]) {
        for replay in self.env.lent_before() {
            self.replay(&replay, stmts);
        }
        self.mutators = narrow::Mutators::of(stmts);
        let mut steps = cpl_syntax::order::Steps::default();
        steps.read(stmts, 0, &mut |f, x| self.call_writes(f, x));
        self.steps = steps;
        self.stmts(stmts, true);
    }

    /// Checks a block in its own scope. Its type is that of its last
    /// expression statement.
    pub(crate) fn block(&mut self, block: &Block) -> Ty {
        self.push_scope();
        let mark = self.lend_mark();
        let last = self.stmts(&block.stmts, false);
        self.lend_reset(mark);
        self.pop_scope();
        last
    }

    /// Checks statements that run one after another. A slice or row
    /// bound by `let` keeps its array lent until the last statement that
    /// names it.
    fn stmts(&mut self, stmts: &[Stmt], top_level: bool) -> Ty {
        let mut last = Ty::Unit;
        let present = self.present.len();
        for (k, stmt) in stmts.iter().enumerate() {
            last = self.stmt(stmt, top_level);
            // `if x == none: return` leaves `x` holding a value.
            if !top_level {
                let held = narrow::after(stmt, &stmts[k + 1..], &mut |f, x| self.call_writes(f, x));
                self.present.extend(held.into_iter().map(String::from));
            }
            if let StmtKind::Let {
                mutable: false,
                pat:
                    Pattern {
                        kind: PatternKind::Name(name),
                        ..
                    },
                init: Some(init),
                ..
            } = &stmt.kind
            {
                self.begin_hold(stmt, name, init, &stmts[k + 1..]);
            }
            self.end_holds(stmt);
        }
        self.present.truncate(present);
        last
    }

    /// Binds the names in `pat` to parts of a value of type `ty`.
    pub(crate) fn bind(&mut self, pat: &Pattern, ty: &Ty, mutable: bool, keyword: &'static str) {
        match &pat.kind {
            PatternKind::Name(_) | PatternKind::Tuple(_) => {
                let mut names = Vec::new();
                pattern_types(pat, ty, &mut |name, ty| names.push((name.to_string(), ty)));
                let mut spans = Vec::new();
                name_spans(pat, &mut spans);
                for ((name, ty), span) in names.into_iter().zip(spans) {
                    self.declare(&name, span, ty, mutable, keyword);
                }
            }
            PatternKind::Variant { path, args } => {
                let fields = match path
                    .last()
                    .and_then(|name| self.env.global(&name.text))
                    .as_deref()
                {
                    Some(Def::Variant { fields, .. }) => fields.clone(),
                    _ => Vec::new(),
                };
                for (i, arg) in args.iter().enumerate() {
                    let ty = fields.get(i).cloned().unwrap_or(Ty::Unknown);
                    self.bind(arg, &ty, mutable, keyword);
                }
            }
            PatternKind::Literal(expr) => {
                self.infer(expr);
            }
            PatternKind::Or(alts) => {
                if let Some(first) = alts.first() {
                    self.bind(first, ty, mutable, keyword);
                }
            }
            PatternKind::Wildcard | PatternKind::Error => {}
        }
    }

    /// `let g: [Row, Col]Tile = read()` reads each `Tile` as text, so the
    /// enum says how its variants are written: nothing is read as a
    /// number or by guessing.
    fn readable(&mut self, declared: &Ty, init: &ast::Expr) {
        let reads = matches!(
            &init.kind,
            ExprKind::Call { callee, args }
                if args.is_empty() && matches!(&callee.kind, ExprKind::Name(name) if name == "read")
        );
        if !reads {
            return;
        }
        let mut todo = vec![declared];
        while let Some(ty) = todo.pop() {
            match ty {
                Ty::Array { elem, .. } | Ty::Optional(elem) => todo.push(elem),
                Ty::Tuple(items) => todo.extend(items),
                Ty::Named { name, args } => {
                    todo.extend(args);
                    let Some(Def::Enum {
                        variants,
                        tagged: false,
                        ..
                    }) = self.env.type_def(name).as_deref().cloned()
                    else {
                        continue;
                    };
                    if variants.iter().any(|(_, fields)| !fields.is_empty()) {
                        continue;
                    }
                    let tags: Vec<String> =
                        variants.iter().map(|(v, _)| format!("{v:?}")).collect();
                    let message = format!(
                        "reading a `{name}` needs to know how its variants are written in the input"
                    );
                    if let Some(diag) = self.error(codes::TYPE_MISMATCH, init.span, message) {
                        diag.help =
                            Some(format!("put `@tags({})` above the enum", tags.join(", ")));
                    }
                }
                _ => {}
            }
        }
    }

    /// What `let pat = init else: ...` binds: the value in each optional.
    /// A tuple of values is taken apart, `let (x, y) = (a, b) else: ...`.
    fn let_else(&mut self, pat: &ast::Pattern, init: &ast::Expr, ty: Ty) -> Ty {
        let inside = |ty: Ty| match ty {
            Ty::Optional(inner) => Ok(*inner),
            Ty::Error => Ok(Ty::Error),
            ty if ty.is_vague() || matches!(ty, Ty::Param(_)) => Ok(Ty::Unknown),
            ty => Err(ty),
        };
        match (&pat.kind, &init.kind, ty) {
            (ast::PatternKind::Tuple(_), ExprKind::Tuple(items), Ty::Tuple(tys))
                if items.len() == tys.len() =>
            {
                let mut out = Vec::new();
                for (item, ty) in items.iter().zip(tys) {
                    out.push(inside(ty).unwrap_or_else(|ty| {
                        let message = format!(
                            "let-else needs a ?T for `{}`, and it is {}",
                            self.quote(item.span),
                            ty.with_article()
                        );
                        self.error(codes::TYPE_MISMATCH, item.span, message);
                        Ty::Error
                    }));
                }
                Ty::Tuple(out)
            }
            (ast::PatternKind::Name(_) | ast::PatternKind::Tuple(_), _, ty) => inside(ty)
                .unwrap_or_else(|ty| {
                    let message = format!(
                        "let-else needs a ?T, and `{}` is {}",
                        self.quote(init.span),
                        ty.with_article()
                    );
                    self.error(codes::TYPE_MISMATCH, init.span, message);
                    Ty::Error
                }),
            (_, _, ty) => inside(ty).unwrap_or(Ty::Unknown),
        }
    }

    /// `m[k] f= x` on a map folds `x` into the entry, and a key that is
    /// not there starts from the identity of `f` (decisions §13). `min`
    /// and `max` store `x`; an operation with no identity is an error.
    fn map_fold(&mut self, target: &ast::Expr, op: AssignOp, value: &ast::Expr) {
        let ExprKind::Index { base, indices } = &target.kind else {
            return;
        };
        let fold = match (op, &value.kind) {
            (AssignOp::Binary(op), _) => {
                if matches!(
                    op,
                    ast::BinOp::Add
                        | ast::BinOp::Sub
                        | ast::BinOp::Mul
                        | ast::BinOp::BitOr
                        | ast::BinOp::BitXor
                        // `m[k] ??= init` is the entry itself: a missing
                        // key gets `init`.
                        | ast::BinOp::Coalesce
                ) {
                    return;
                }
                op.text().to_string()
            }
            // `m[k] f= x`, which the parser wrote as `m[k] = f(m[k], x)`.
            (AssignOp::Set, ExprKind::Call { callee, args })
                if args.first().is_some_and(|first| first.value == *target) =>
            {
                match &callee.kind {
                    ExprKind::Name(name) if !matches!(name.as_str(), "gcd" | "lcm") => name.clone(),
                    _ => return,
                }
            }
            _ => return,
        };
        let map = self.quietly(|c| c.infer(base));
        let Ty::Named { name, args } = &map else {
            return;
        };
        let plain = matches!(args.as_slice(), [_, value] if !matches!(value, Ty::Optional(_)) && !value.is_vague());
        if !matches!(name.as_str(), "Map" | "HashMap") || !plain {
            return;
        }
        let (target_text, key) = (self.quote(target.span), self.quote(indices[0].span));
        let message = format!(
            "`{target_text} {fold}= x` may meet a key that is not in the map yet, and `{fold}` has no identity to start it from"
        );
        let base = self.quote(base.span);
        let help = match op {
            AssignOp::Binary(_) => format!(
                "say what a missing key means: `{target_text} = ({base}.get({key}) ?? start) {fold} x`"
            ),
            _ => format!(
                "say what a missing key means: `{target_text} = {fold}({base}.get({key}) ?? start, x)`"
            ),
        };
        if let Some(diag) = self.error(codes::BAD_OPERANDS, target.span, message) {
            diag.help = Some(help);
        }
    }

    /// A domain written with two literal ends holds at most 2^31 - 1
    /// values: a position is an `i32` in the generated code.
    fn domain_size(&mut self, def: &ast::Expr) {
        let ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
            ..
        } = &def.kind
        else {
            return;
        };
        let (ExprKind::Int(lo), ExprKind::Int(hi)) = (&lo.kind, &hi.kind) else {
            return;
        };
        let len = *hi - *lo + i128::from(*inclusive);
        if len > i128::from(i32::MAX) {
            let message = format!("a domain holds at most 2^31 - 1 values, and this one has {len}");
            self.error(codes::TYPE_MISMATCH, def.span, message);
        }
    }

    fn stmt(&mut self, stmt: &Stmt, top_level: bool) -> Ty {
        // A statement is straight-line code again, whatever it is inside;
        // an `if` or `match` standing as a statement is one too.
        let branch = match &stmt.kind {
            StmtKind::Expr(expr) => {
                if matches!(expr.kind, ExprKind::If { .. } | ExprKind::Match { .. }) {
                    self.plain_if = Some(expr.span);
                }
                self.branch
            }
            _ => std::mem::take(&mut self.branch),
        };
        let ty = self.stmt_kind(stmt, top_level);
        self.branch = branch;
        ty
    }

    fn stmt_kind(&mut self, stmt: &Stmt, top_level: bool) -> Ty {
        match &stmt.kind {
            StmtKind::Let {
                mutable,
                pat,
                ty,
                init,
                otherwise,
                guard,
            } => {
                let keyword = if *mutable { "var" } else { "let" };
                let declared = ty.as_ref().map(|ty| self.resolve_type(ty));
                let mut init_ty = init.as_ref().map(|init| self.infer(init));
                if let (Some(declared), Some(init)) = (&declared, init) {
                    self.readable(declared, init);
                }
                // `let v = u else: return`: past here `v` is the value in `u`.
                if let Some(otherwise) = otherwise {
                    init_ty = match (init_ty, init) {
                        (Some(ty), Some(init)) => Some(self.let_else(pat, init, ty)),
                        (ty, _) => ty,
                    };
                    self.block(otherwise);
                }
                // `var seen: [V]Bool = false` fills the array with the value.
                // A comprehension builds the array when one is declared.
                let fills = match (&declared, &init_ty) {
                    (Some(Ty::Array { elem, .. }), Some(Ty::Named { name, args }))
                        if name == "List" && args.len() == 1 =>
                    {
                        // `[1, 2, 3]` is the cells; `var ways: [0..n]List<Z> = [Z(1)]`
                        // fills each cell with the list.
                        self.fits(&args[0], elem) || self.fits(init_ty.as_ref().expect("matched"), elem)
                    }
                    (Some(Ty::Array { elem, .. }), Some(init_ty)) => {
                        // A literal fills the cells of an array of arrays too.
                        let mut cell = &**elem;
                        let literal = init.as_ref().is_some_and(|e| {
                            matches!(e.kind, ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Float(_))
                        });
                        while let (true, Ty::Array { elem, .. }) = (literal, cell) {
                            cell = elem;
                        }
                        self.fits(init_ty, cell)
                    }
                    _ => false,
                };
                if let (Some(declared), Some(init_ty), Some(init), false) =
                    (&declared, &init_ty, init, fills)
                {
                    let name = self.quote(pat.span);
                    self.expect_fits(init_ty, declared, init, |c, got, want| {
                        format!(
                            "`{name}` is declared {}, but `{}` is {}",
                            want.with_article(),
                            c.quote(init.span),
                            got.with_article()
                        )
                    });
                }
                let ty = declared.or(init_ty).unwrap_or(Ty::Error);
                if let (true, Some(init)) = (*mutable, init) {
                    let what = format!("`var {}`", self.quote(pat.span));
                    self.no_escape(init, &what);
                }
                // A new binding of a name: what was known about the old
                // one no longer applies.
                let mut bound = Vec::new();
                pattern_types(pat, &ty, &mut |name, _| bound.push(name.to_string()));
                self.drop_facts(&bound);
                match super::types::domain_binding(pat) {
                    // `let LIM = 100`: with no `else` there is no variant
                    // to match, so a capitalized name is bound to the value.
                    Some(name)
                        if otherwise.is_none()
                            && !matches!(
                                self.env.global(name).as_deref(),
                                Some(Def::Variant { .. })
                            ) =>
                    {
                        self.declare(name, pat.span, ty.clone(), *mutable, keyword);
                    }
                    _ => self.bind(pat, &ty, *mutable, keyword),
                }
                if let Some(guard) = guard {
                    let ty = self.infer(guard);
                    self.require_bool(&ty, guard, "`where`");
                }
            }
            StmtKind::Const {
                name,
                generics,
                bounds,
                ty,
                init,
            } => {
                let outer = self.generics.len();
                self.generics
                    .extend(generics.iter().map(|g| g.text.clone()));
                self.bounds_exist(bounds);
                let declared = ty.as_ref().map(|ty| self.resolve_type(ty));
                let init_ty = init.as_ref().map(|init| self.infer(init));
                if let (Some(declared), Some(init_ty), Some(init)) = (&declared, &init_ty, init) {
                    self.expect_fits(init_ty, declared, init, |c, got, want| {
                        format!(
                            "`{}` is declared {}, but `{}` is {}",
                            name.text,
                            want.with_article(),
                            c.quote(init.span),
                            got.with_article()
                        )
                    });
                }
                let ty = declared.or(init_ty).unwrap_or(Ty::Error);
                self.generics.truncate(outer);
                self.declare(&name.text, name.span, ty, false, "const");
            }
            StmtKind::Fn(f) => {
                // `@memo(into: dp)` on a fn inside a fn or a loop: the
                // table is a `var` where the fn stands.
                if let (false, Some((table, at))) = (top_level, crate::check::memo_table(stmt)) {
                    if let Ty::Fn { params, ret } = self.fn_sig(f, None) {
                        let ty = Ty::Array {
                            index: params.iter().map(|p| p.ty.clone()).collect(),
                            elem: Box::new(Ty::Optional(Box::new(ret.widen()))),
                        };
                        self.declare(table, at, ty, true, "var");
                    }
                }
                self.fn_decl(f, None, !top_level, memo_of(stmt))
            }
            StmtKind::Struct(s) => {
                let generics: Vec<String> = s.generics.iter().map(|g| g.text.clone()).collect();
                let self_ty = Ty::Named {
                    name: s.name.text.clone(),
                    args: generics.iter().cloned().map(Ty::Param).collect(),
                };
                let body = self.enter_body(&generics, self_ty.clone());
                self.bounds_exist(&s.bounds);
                self.owned(&s.name.text, &s.methods);
                for field in &s.fields {
                    let ty = self.resolve_type(&field.ty);
                    self.hovers.push(HoverEntry {
                        span: field.name.span,
                        name: field.name.text.clone(),
                        ty: ty.clone(),
                        keyword: "field",
                        decl: Some(field.name.span),
                    });
                    if let Some(default) = &field.default {
                        let default_ty = self.infer(default);
                        self.expect_fits(&default_ty, &ty, default, |c, got, want| {
                            format!(
                                "`{}` is {}, but its default `{}` is {}",
                                field.name.text,
                                want.with_article(),
                                c.quote(default.span),
                                got.with_article()
                            )
                        });
                    }
                }
                self.type_hover(&s.name);
                self.methods(&s.methods, &self_ty);
                self.leave_body(body);
            }
            StmtKind::Enum(e) => {
                let generics: Vec<String> = e.generics.iter().map(|g| g.text.clone()).collect();
                let self_ty = Ty::Named {
                    name: e.name.text.clone(),
                    args: generics.iter().cloned().map(Ty::Param).collect(),
                };
                let body = self.enter_body(&generics, self_ty.clone());
                for variant in &e.variants {
                    for (_, ty) in &variant.fields {
                        self.resolve_type(ty);
                    }
                }
                self.type_hover(&e.name);
                self.methods(&e.methods, &self_ty);
                self.leave_body(body);
            }
            StmtKind::Domain { name, def } => {
                self.infer(def);
                self.domain_size(def);
                self.type_hover(name);
                if !top_level {
                    self.declare(
                        &name.text,
                        name.span,
                        Ty::Domain(name.text.clone()),
                        false,
                        "domain",
                    );
                }
            }
            StmtKind::TypeAlias { name, generics, ty } => {
                let outer = self.generics.len();
                self.generics
                    .extend(generics.iter().map(|g| g.text.clone()));
                let aliased = self.resolve_type(ty);
                self.generics.truncate(outer);
                self.type_hover(name);
                if !top_level {
                    self.scopes.last_mut().unwrap().insert(
                        name.text.clone(),
                        Local {
                            ty: aliased,
                            mutable: false,
                            span: name.span,
                            keyword: "type",
                        },
                    );
                }
            }
            StmtKind::Trait {
                name,
                generics,
                supers,
                body,
            } => {
                let generics: Vec<String> = generics.iter().map(|g| g.text.clone()).collect();
                let self_ty = Ty::Param("Self".into());
                let scope = self.enter_body(&generics, self_ty.clone());
                for sup in supers {
                    self.trait_exists(sup);
                }
                self.type_hover(name);
                self.owned(&name.text, &body.stmts);
                self.methods(&body.stmts, &self_ty);
                self.leave_body(scope);
            }
            StmtKind::Impl {
                trait_name,
                target,
                bounds,
                body,
            } => {
                let generics = self.impl_generics(target);
                let scope = self.enter_body(&generics, Ty::Unknown);
                let self_ty = self.resolve_type(target);
                self.self_ty = Some(self_ty.clone());
                self.bounds_exist(bounds);
                if let Some(trait_name) = trait_name {
                    self.impl_matches_trait(trait_name, target, &body.stmts);
                    if type_head(trait_name).as_deref() == Some("Index") {
                        self.index_size(&body.stmts);
                    }
                }
                let owner = type_head(target).unwrap_or_default();
                let owned_from_struct: Vec<(String, Ty)> =
                    if let Some(Def::Struct { owned, .. }) = self.env.type_def(&owner).as_deref() {
                        owned.clone()
                    } else {
                        Vec::new()
                    };
                for (oname, oty) in owned_from_struct {
                    self.scopes.last_mut().unwrap().insert(
                        oname,
                        Local {
                            ty: oty,
                            mutable: false,
                            span: target.span,
                            keyword: "let",
                        },
                    );
                }
                self.owned(&owner, &body.stmts);
                self.methods(&body.stmts, &self_ty);
                self.leave_body(scope);
            }
            StmtKind::Use { .. } | StmtKind::Break(_) | StmtKind::Continue(_) | StmtKind::Error => {
            }
            StmtKind::Expr(expr) => return self.infer(expr),
            StmtKind::Assign { target, op, value } => {
                let target_ty = self.infer(target);
                let value_ty = self.infer(value);
                self.optional_fold(target, &target_ty, *op);
                self.map_fold(target, *op, value);
                self.assign_alias(target, *op, value);
                self.require_mutable(target, "assigned");
                self.check_access(target, true);
                if let Some((root, _)) = Self::root_name(target) {
                    self.drop_facts(&[root.to_string()]);
                }
                if *op == AssignOp::Set {
                    self.no_escape(value, "this assignment");
                }
                if *op == AssignOp::Set {
                    self.expect_fits(&value_ty, &target_ty, value, |c, got, want| {
                        format!(
                            "`{}` is {}, but `{}` is {}",
                            c.quote(target.span),
                            want.with_article(),
                            c.quote(value.span),
                            got.with_article()
                        )
                    });
                }
            }
            StmtKind::While { cond, body } => {
                self.push_scope();
                self.drop_facts_written(body, None);
                self.cond(cond);
                let mark = self.lend_mark();
                self.add_cond_facts(cond, false);
                self.block(body);
                self.lend_reset(mark);
                self.pop_scope();
            }
            StmtKind::For {
                pat, iter, body, ..
            } => {
                let iter_ty = self.infer(iter);
                let elem = self.element_ty(&iter_ty);
                self.push_scope();
                self.drop_facts_written(body, Some(pat));
                self.bind(pat, &elem, false, "for");
                let mark = self.lend_mark();
                self.hold_iterated(stmt, iter, &iter_ty);
                self.block(body);
                self.lend_reset(mark);
                self.pop_scope();
            }
            StmtKind::Return(value) => {
                let value_ty = value.as_ref().map(|v| self.infer(v));
                if let (Some(value), true) = (value, self.fns.is_empty()) {
                    // Decision 20.3: there is no caller to receive it.
                    let diag = self.error(
                        codes::UNEXPECTED_TOKEN,
                        value.span,
                        "a top-level `return` takes no value".to_string(),
                    );
                    if let Some(diag) = diag {
                        diag.help = Some(
                            "write a bare `return`: it ends the program, or the test case below `@!multitest`"
                                .to_string(),
                        );
                    }
                }
                if let (Some(value), Some(value_ty)) = (value, value_ty) {
                    self.returns(value, &value_ty);
                    self.no_escape(value, "`return`");
                }
            }
            StmtKind::Yield { value, .. } => {
                self.infer(value);
            }
            StmtKind::Assert { cond, message } => {
                let ty = self.infer(cond);
                self.require_bool(&ty, cond, "`assert`");
                self.add_facts(cond, false);
                if let Some(message) = message {
                    self.infer(message);
                }
            }
            StmtKind::Unchecked(body) => return self.block(body),
            StmtKind::Law { params, body, .. } => {
                self.push_scope();
                for param in params {
                    let ty = match &param.ty {
                        Some(ty) => self.resolve_type(ty),
                        None => Ty::Unknown,
                    };
                    self.declare(&param.name.text, param.name.span, ty, false, "param");
                }
                self.infer(body);
                self.pop_scope();
            }
            StmtKind::Lend { place, .. } => {
                self.infer(place);
                self.lend_in_place(stmt);
            }
        }
        Ty::Unit
    }

    /// Checks a returned value against the enclosing fn's declared type.
    fn returns(&mut self, value: &Expr, value_ty: &Ty) {
        let Some((name, ret, declared)) = self.fns.last().cloned() else {
            return;
        };
        // `print(..)` is checked loosely, as a call of unknown type: it
        // gives nothing.
        let prints = matches!(&value.kind, ExprKind::Call { callee, .. }
            if matches!(&callee.kind, ExprKind::Name(name) if name == "print"));
        if let (Some(gives), false) = (self.gives.last_mut(), *value_ty == Ty::Unit || prints) {
            *gives = true;
        }
        if !declared {
            return;
        }
        self.expect_fits(value_ty, &ret, value, |c, got, want| {
            format!(
                "`{name}` returns {}, but `{}` is {}",
                want.with_article(),
                c.quote(value.span),
                got.with_article()
            )
        });
    }

    fn type_hover(&mut self, name: &ast::Name) {
        self.hovers.push(HoverEntry {
            span: name.span,
            name: name.text.clone(),
            ty: Ty::TypeName(name.text.clone()),
            keyword: "type",
            decl: Some(name.span),
        });
    }

    /// Checks the fns, laws and constants of a struct, enum, trait or impl
    /// body. The struct's fields can be named in them without `self.`.
    fn methods(&mut self, methods: &[Stmt], self_ty: &Ty) {
        let fields = match self_ty {
            Ty::Named { name, .. } => match self.env.global(name).as_deref() {
                Some(Def::Struct { fields, .. }) => fields.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        for method in methods {
            match &method.kind {
                StmtKind::Fn(f) => self.fn_decl(f, Some(self_ty), false, memo_of(method)),
                StmtKind::Law { name, .. } => {
                    self.push_scope();
                    for field in &fields {
                        self.scopes.last_mut().unwrap().insert(
                            field.name.clone(),
                            Local {
                                ty: field.ty.clone(),
                                mutable: false,
                                span: name.span,
                                keyword: "self",
                            },
                        );
                    }
                    self.stmt(method, false);
                    self.pop_scope();
                }
                // Owned domains and constants were declared on entry; this
                // checks their values.
                StmtKind::Domain { def, .. } => {
                    self.infer(def);
                }
                StmtKind::Const {
                    init: Some(init), ..
                }
                | StmtKind::Let {
                    init: Some(init), ..
                } => {
                    self.infer(init);
                }
                _ => {
                    self.stmt(method, false);
                }
            }
        }
    }

    /// The `K` of an `impl Index`: how many values the type has. `const K`
    /// is known at compile time; `let K` is evaluated once, where the
    /// impl is declared, and frozen, so it can't read a `var`
    /// (plan/19 §3.1).
    fn index_size(&mut self, body: &[Stmt]) {
        for stmt in body {
            match &stmt.kind {
                StmtKind::Const {
                    name,
                    init: Some(init),
                    ..
                } if name.text == "K" => {
                    let known = matches!(self.bounds(init), (Some(lo), Some(hi)) if lo == hi);
                    if !known {
                        let value = self.quote(init.span);
                        let message = format!(
                            "`const K = {value}`: `const` is compile-time only, and `{value}` is known only at run time"
                        );
                        if let Some(diag) = self.error(codes::BAD_INDEX_IMPL, stmt.span, message) {
                            diag.help = Some(format!(
                                "write `let K = {value}`; an associated `let` is evaluated once, where the impl is declared, and is frozen from then on"
                            ));
                        }
                    }
                }
                StmtKind::Let {
                    pat,
                    init: Some(init),
                    ..
                } if super::types::domain_binding(pat) == Some("K") => {
                    for name in super::lends::names_in(init) {
                        let mutable = match self.local(&name) {
                            Some(local) => local.mutable,
                            None => matches!(
                                self.env.global(&name).as_deref(),
                                Some(Def::Value { mutable: true, .. })
                            ),
                        };
                        if mutable {
                            let message = format!(
                                "`K` can't be sized by the var `{name}`: it could change after arrays are made"
                            );
                            if let Some(diag) =
                                self.error(codes::BAD_INDEX_IMPL, init.span, message)
                            {
                                diag.help = Some(format!("freeze it first: `let n = {name}`"));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Every trait a bound names must exist: `where T: Ord + Bounded`.
    fn bounds_exist(&mut self, bounds: &[ast::Bound]) {
        for bound in bounds {
            for trait_ty in &bound.traits {
                self.trait_exists(trait_ty);
            }
        }
    }

    /// Reports a trait name that is not a trait. Returns the trait's
    /// methods when it is declared in cp source.
    fn trait_exists(&mut self, trait_ty: &ast::Type) -> Option<Vec<crate::def::TraitMethod>> {
        let ast::TypeKind::Named { name, args } = &trait_ty.kind else {
            return None;
        };
        for arg in args {
            self.resolve_type(arg);
        }
        match self.env.global(&name.text).as_deref() {
            Some(Def::Trait { methods, .. }) => {
                self.refs.push(GlobalRef {
                    span: name.span,
                    name: name.text.clone(),
                    top_level: false,
                });
                return Some(methods.clone());
            }
            Some(_) => {
                let message = format!("`{}` is not a trait", name.text);
                self.error(codes::UNKNOWN_TYPE, name.span, message);
            }
            None if prelude::is_trait(&name.text) || name.text.is_empty() => {}
            None if self.env.has_imports() => {}
            None => {
                let message = format!("no trait named `{}`", name.text);
                self.error(codes::UNKNOWN_TYPE, name.span, message);
            }
        }
        None
    }

    /// An impl of a trait written in cp must define every fn the trait
    /// leaves without a body, and no fn the trait does not have.
    fn impl_matches_trait(&mut self, trait_ty: &ast::Type, target: &ast::Type, body: &[Stmt]) {
        let Some(methods) = self.trait_exists(trait_ty) else {
            return;
        };
        let trait_text = self.quote(trait_ty.span);
        let defined: Vec<&ast::Name> = body
            .iter()
            .filter_map(|m| match &m.kind {
                StmtKind::Fn(f) => Some(&f.name),
                _ => None,
            })
            .collect();
        for name in &defined {
            if methods.iter().all(|m| m.name != name.text) {
                let known: Vec<String> = methods.iter().map(|m| m.name.clone()).collect();
                let suggestion = closest(&name.text, known.iter().map(String::as_str));
                let message = format!("`{trait_text}` has no fn `{}`", name.text);
                if let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) {
                    diag.help = Some(match suggestion {
                        Some(s) => format!("did you mean `{s}`?"),
                        None => format!("its fns are {}", super::expr::list_names(&known)),
                    });
                }
            }
        }
        let missing: Vec<String> = methods
            .iter()
            .filter(|m| m.required && defined.iter().all(|d| d.text != m.name))
            .map(|m| m.name.clone())
            .collect();
        if !missing.is_empty() {
            let message = format!(
                "`impl {trait_text} for {}` is missing {}",
                self.quote(target.span),
                super::expr::list_names(&missing)
            );
            let span = trait_ty.span.to(target.span);
            if let Some(diag) = self.error(codes::WRONG_ARG_COUNT, span, message) {
                diag.help = Some(format!(
                    "`{trait_text}` declares {} without a body, so each impl writes {}",
                    if missing.len() == 1 { "it" } else { "them" },
                    if missing.len() == 1 { "it" } else { "them" },
                ));
            }
        }
    }

    /// Reports a `@memo` that would not remember anything. Both back ends
    /// keep one table of answers per top-level or local fn, keyed by its
    /// arguments, and run the fn as written when they can't: the program
    /// would be right and, without a word, as slow as if `@memo` were not
    /// there.
    fn memo_fits(&mut self, f: &ast::FnDecl, self_ty: Option<&Ty>, generator: bool, unit: bool, memo: Span) {
        let name = &f.name.text;
        let changed = f.params.iter().find(|p| p.mutable && p.name.text != "self");
        let (message, help) = if let Some(owner) = self_ty {
            let owner = owner.to_string();
            (
                format!("`@memo` can't remember `{name}`: it is a fn of `{owner}`"),
                format!("only a top-level or local fn keeps a table of its answers: move `{name}` out of `{owner}` and pass it what it reads"),
            )
        } else if let Some(param) = changed {
            let param = &param.name.text;
            (
                format!("`@memo` can't remember `{name}`: it changes its argument `{param}`"),
                format!("a remembered answer is given back without running the fn, so `{param}` would be left as it was: take `{param}` without `mut`, or drop `@memo`"),
            )
        } else if generator {
            (
                format!("`@memo` can't remember `{name}`: it yields its values one by one"),
                "collect the values into a list and return that: `-> List<T>`".to_string(),
            )
        } else if !f.generics.is_empty() {
            (
                format!("`@memo` can't remember the generic fn `{name}`"),
                format!("there is one table of answers per fn, and `{name}` is a different fn for each type it is called with: write it for the type it is used with"),
            )
        } else if unit {
            (
                format!("`@memo` has nothing to remember: `{name}` gives no value"),
                "a fn that only acts has to run every time: drop `@memo`, or return what it computes".to_string(),
            )
        } else {
            return;
        };
        if let Some(diag) = self.error(codes::BAD_MEMO, memo, message) {
            diag.help = Some(help);
        }
    }

    /// `memo` is where the fn's `@memo` is written, when it has one.
    fn fn_decl(&mut self, f: &ast::FnDecl, self_ty: Option<&Ty>, local: bool, memo: Option<Span>) {
        let sig = self.fn_sig(f, self_ty);
        let Ty::Fn { params, ret } = &sig else { return };
        if local {
            self.declare(&f.name.text, f.name.span, sig.clone(), false, "fn");
        } else {
            self.hovers.push(HoverEntry {
                span: f.name.span,
                name: f.name.text.clone(),
                ty: sig.clone(),
                keyword: "fn",
                decl: Some(f.name.span),
            });
        }
        let outer = self.generics.len();
        self.generics
            .extend(f.generics.iter().map(|g| g.text.clone()));
        self.bounds_exist(&f.bounds);
        self.in_fn += 1;
        self.push_scope();
        // Inside a method the struct's fields can be named without `self.`.
        let receiver = f.params.iter().find(|p| p.name.text == "self");
        if let (Some(receiver), Some(Ty::Named { name, .. })) = (receiver, self_ty) {
            if let Some(Def::Struct { fields, .. }) = self.env.global(name).as_deref() {
                for field in fields {
                    self.scopes.last_mut().unwrap().insert(
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
                self.expect_fits(&default_ty, &ty, default, |c, got, want| {
                    format!(
                        "`{}` is {}, but its default `{}` is {}",
                        param.name.text,
                        want.with_article(),
                        c.quote(default.span),
                        got.with_article()
                    )
                });
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
        self.fns.push((
            f.name.text.clone(),
            (**ret).clone(),
            f.ret.is_some() && !generator,
        ));
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
                    let ty = c.block(block);
                    // The last expression is the fn's value.
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
                            c.returns(last, &ty);
                            c.no_escape(last, "returning it");
                        }
                    }
                }
                FnBody::Expr(expr) => {
                    let ty = c.infer(expr);
                    c.returns(expr, &ty);
                    c.no_escape(expr, "returning it");
                }
                FnBody::None => {}
            }
        });
        self.fns.pop();
        let gives = self.gives.pop().unwrap_or(false);
        if let Some(memo) = memo {
            // Written `-> T`, or read off what the body gives.
            let unit = match &f.ret {
                Some(_) => **ret == Ty::Unit,
                None => !gives && !matches!(f.body, FnBody::None),
            };
            self.memo_fits(f, self_ty, generator, unit, memo);
        }
        self.pop_scope();
        if receiver.is_some() && matches!(self_ty, Some(Ty::Named { .. })) {
            self.pop_scope();
        }
        self.in_fn -= 1;
        self.generics.truncate(outer);
    }
}

/// Where a fn's `@memo` is written.
fn memo_of(stmt: &Stmt) -> Option<Span> {
    let memo = stmt.decorators.iter().find(|d| d.name.text == "memo")?;
    Some(memo.span)
}

/// The span of each name a pattern binds, in the order `pattern_types`
/// visits them.
fn name_spans(pat: &Pattern, out: &mut Vec<Span>) {
    match &pat.kind {
        PatternKind::Name(_) => out.push(pat.span),
        PatternKind::Tuple(items) => {
            for item in items {
                match super::types::domain_binding(item) {
                    Some(_) => out.push(item.span),
                    None => name_spans(item, out),
                }
            }
        }
        _ => {}
    }
}
