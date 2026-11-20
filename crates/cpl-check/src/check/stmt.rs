//! Statements, blocks, and declarations.

use super::types::pattern_types;
use super::*;
use cpl_syntax::ast::{
    self, AssignOp, Block, Expr, ExprKind, FnBody, Pattern, PatternKind, Stmt, StmtKind,
};

impl<'a> Checker<'a> {
    /// Checks the statements of one top-level item.
    pub(crate) fn item(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.stmt(stmt, true);
        }
    }

    /// Checks a block in its own scope. Its type is that of its last
    /// expression statement.
    pub(crate) fn block(&mut self, block: &Block) -> Ty {
        self.push_scope();
        let mut last = Ty::Unit;
        for stmt in &block.stmts {
            last = self.stmt(stmt, false);
        }
        self.pop_scope();
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

    fn stmt(&mut self, stmt: &Stmt, top_level: bool) -> Ty {
        match &stmt.kind {
            StmtKind::Let {
                mutable,
                pat,
                ty,
                init,
            } => {
                let keyword = if *mutable { "var" } else { "let" };
                let declared = ty.as_ref().map(|ty| self.resolve_type(ty));
                let init_ty = init.as_ref().map(|init| self.infer(init));
                // `var seen: [V]Bool = false` fills the array with the value.
                // A comprehension builds the array when one is declared.
                let fills = match (&declared, &init_ty) {
                    (Some(Ty::Array { elem, .. }), Some(Ty::Named { name, args }))
                        if name == "List" && args.len() == 1 =>
                    {
                        args[0].fits(elem)
                    }
                    (Some(Ty::Array { elem, .. }), Some(init_ty)) => init_ty.fits(elem),
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
                self.bind(pat, &ty, *mutable, keyword);
            }
            StmtKind::Const { name, ty, init } => {
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
                self.declare(&name.text, name.span, ty, false, "const");
            }
            StmtKind::Fn(f) => self.fn_decl(f, None, !top_level),
            StmtKind::Struct(s) => {
                let self_ty = Ty::Named {
                    name: s.name.text.clone(),
                    args: Vec::new(),
                };
                let outer = self.generics.len();
                self.generics
                    .extend(s.generics.iter().map(|g| g.text.clone()));
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
                self.generics.truncate(outer);
            }
            StmtKind::Enum(e) => {
                let self_ty = Ty::Named {
                    name: e.name.text.clone(),
                    args: Vec::new(),
                };
                let outer = self.generics.len();
                self.generics
                    .extend(e.generics.iter().map(|g| g.text.clone()));
                for variant in &e.variants {
                    for (_, ty) in &variant.fields {
                        self.resolve_type(ty);
                    }
                }
                self.type_hover(&e.name);
                self.methods(&e.methods, &self_ty);
                self.generics.truncate(outer);
            }
            StmtKind::Domain { name, def } => {
                self.infer(def);
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
                self.resolve_type(ty);
                self.generics.truncate(outer);
                self.type_hover(name);
            }
            StmtKind::Trait {
                name,
                generics,
                body,
            } => {
                let outer = self.generics.len();
                self.generics
                    .extend(generics.iter().map(|g| g.text.clone()));
                self.type_hover(name);
                self.methods(&body.stmts, &Ty::Unknown);
                self.generics.truncate(outer);
            }
            StmtKind::Impl {
                trait_name,
                target,
                body,
            } => {
                if let Some(trait_name) = trait_name {
                    self.resolve_type(trait_name);
                }
                let self_ty = self.resolve_type(target);
                self.methods(&body.stmts, &self_ty);
            }
            StmtKind::Use { .. } | StmtKind::Break(_) | StmtKind::Continue(_) | StmtKind::Error => {
            }
            StmtKind::Expr(expr) => return self.infer(expr),
            StmtKind::Assign { target, op, value } => {
                let target_ty = self.infer(target);
                let value_ty = self.infer(value);
                self.require_mutable(target, "assigned");
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
                self.cond(cond);
                self.block(body);
                self.pop_scope();
            }
            StmtKind::For {
                pat, iter, body, ..
            } => {
                let iter_ty = self.infer(iter);
                let elem = self.element_ty(&iter_ty);
                self.push_scope();
                self.bind(pat, &elem, false, "for");
                self.block(body);
                self.pop_scope();
            }
            StmtKind::Return(value) => {
                let value_ty = value.as_ref().map(|v| self.infer(v));
                if let (Some(value), Some(value_ty)) = (value, value_ty) {
                    self.returns(value, &value_ty);
                }
            }
            StmtKind::Yield { value, .. } => {
                self.infer(value);
            }
            StmtKind::Assert { cond, message } => {
                let ty = self.infer(cond);
                self.require_bool(&ty, cond, "`assert`");
                if let Some(message) = message {
                    self.infer(message);
                }
            }
            StmtKind::Unchecked(body) => return self.block(body),
        }
        Ty::Unit
    }

    /// Checks a returned value against the enclosing fn's declared type.
    fn returns(&mut self, value: &Expr, value_ty: &Ty) {
        let Some((name, ret, declared)) = self.fns.last().cloned() else {
            return;
        };
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

    fn methods(&mut self, methods: &[Stmt], self_ty: &Ty) {
        for method in methods {
            match &method.kind {
                StmtKind::Fn(f) => self.fn_decl(f, Some(self_ty), false),
                _ => {
                    self.stmt(method, false);
                }
            }
        }
    }

    fn fn_decl(&mut self, f: &ast::FnDecl, self_ty: Option<&Ty>, local: bool) {
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
        match &f.body {
            FnBody::Block(block) => {
                let ty = self.block(block);
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
                        self.returns(last, &ty);
                    }
                }
            }
            FnBody::Expr(expr) => {
                let ty = self.infer(expr);
                self.returns(expr, &ty);
            }
            FnBody::None => {}
        }
        self.fns.pop();
        self.pop_scope();
        if receiver.is_some() && matches!(self_ty, Some(Ty::Named { .. })) {
            self.pop_scope();
        }
        self.in_fn -= 1;
        self.generics.truncate(outer);
    }
}

/// The span of each name a pattern binds, in the order `pattern_types`
/// visits them.
fn name_spans(pat: &Pattern, out: &mut Vec<Span>) {
    match &pat.kind {
        PatternKind::Name(_) => out.push(pat.span),
        PatternKind::Tuple(items) => items.iter().for_each(|i| name_spans(i, out)),
        _ => {}
    }
}
