//! `let` and `var` declarations: a global at the top level of the file, a
//! local anywhere else, and what the name keeps known about its view.

use super::*;

impl<'a> Lowerer<'a> {
    /// A local `Int` named as a type's value argument (`type Z = Mod(q)`,
    /// `for d in 0..=6: type F = Fixed<d>`) is mirrored into a global, so
    /// monomorphized methods read it at run time.
    pub(super) fn mirror_value_args(&mut self, ty: &ast::Type, out: &mut Vec<Stmt>) {
        let ast::TypeKind::Named { args, .. } = &ty.kind else {
            return;
        };
        for arg in args {
            match &arg.kind {
                ast::TypeKind::Value(e) => {
                    if let ast::ExprKind::Name(n) = &e.kind {
                        self.mirror_local_int(n, e.span, out);
                    }
                }
                ast::TypeKind::Named { name, args } if args.is_empty() => {
                    self.mirror_local_int(&name.text, name.span, out);
                }
                _ => self.mirror_value_args(arg, out),
            }
        }
    }

    pub(super) fn mirror_local_int(&mut self, name: &str, span: Span, out: &mut Vec<Stmt>) {
        let Some(slot) = self.lookup(name) else {
            return;
        };
        let value = self.local(slot, span);
        if !matches!(value.ty, Ty::Int) {
            return;
        }
        let gid = self.globals.len();
        self.globals.push(Global {
            name: format!("{name}_rt_{gid}"),
            ty: Ty::Int,
            span: Span::default(),
        });
        let place = Place {
            kind: PlaceKind::Global(gid),
            ty: Ty::Int,
            span: self.span(span),
        };
        out.push(Stmt::Assign(place, AssignOp::Set, value));
        if let Some(scope) = self.cx.local_syms.last_mut() {
            scope.insert(name.to_string(), Sym::Global(gid));
        }
    }

    /// Declares `name` with the value `init`: a global at the top level of
    /// the file, a local anywhere else.
    pub(super) fn declare(&mut self, name: &str, init: Expr, mutable: bool, out: &mut Vec<Stmt>) {
        let slice_lo = match &init.kind {
            ExprKind::Builtin(Builtin::ListSlice | Builtin::StrSlice, args) => {
                args.get(1).and_then(|e| match &e.kind {
                    ExprKind::Int(v) => Some(*v),
                    _ => None,
                })
            }
            _ => None,
        };
        let reversed = !mutable && self.is_reversed(&init);
        if self.cx.is_main && self.cx.scopes.len() == 1 {
            self.cx.recipes.remove(name);
            self.global_map_views
                .remove(&(self.cur_mod, name.to_string()));
            let id = self.globals.len();
            self.globals.push(Global {
                name: name.to_string(),
                ty: init.ty.clone(),
                span: init.span,
            });
            self.mod_syms[self.cur_mod].insert(name.to_string(), Sym::Global(id));
            if mutable {
                self.mut_globals.insert(id);
            }
            if let Some(lo) = slice_lo {
                self.global_view_lo.insert(id, lo);
            }
            if reversed {
                self.global_view_rev.insert(id);
            }
            let place = Place {
                kind: PlaceKind::Global(id),
                ty: init.ty.clone(),
                span: init.span,
            };
            out.push(Stmt::Assign(place, AssignOp::Set, init));
        } else {
            let slot = self.new_slot(name, init.ty.clone());
            if mutable {
                self.cx.mut_slots.insert(slot);
            }
            if let Some(lo) = slice_lo {
                self.cx.view_lo.insert(slot, lo);
            }
            if reversed {
                self.cx.view_rev.insert(slot);
            }
            self.cx
                .scopes
                .last_mut()
                .expect("a scope")
                .insert(name.to_string(), slot);
            out.push(Stmt::Let(slot, Some(init)));
        }
    }

    /// A `let` name bound to `a.rev`, whose `.keys` walk `a`'s backwards (R07.4.7).
    pub(super) fn named_reversed(&self, value: &Expr) -> bool {
        match &value.kind {
            ExprKind::Local(slot) => self.cx.view_rev.contains(slot),
            ExprKind::Global(id) => self.global_view_rev.contains(id),
            _ => false,
        }
    }
}

/// The env entry saying a generic fn's param `name` was given a `.rev` of a
/// list or view, so the instance's `name.keys` walk backwards (R07.4.7).
pub(super) fn rev_param_key(name: &str) -> String {
    format!("__rev_param_{name}")
}

impl<'a> Lowerer<'a> {
    /// `e` is `a.rev` over a list or view, or a name bound to one.
    fn reversed_arg(&mut self, e: &ast::Expr) -> bool {
        match &e.kind {
            ast::ExprKind::Field { base, name } if name.text == "rev" => matches!(
                self.probe_ty_silent(base),
                Some(Ty::List(_) | Ty::ListView(..) | Ty::MutView(..))
            ),
            ast::ExprKind::Name(n) => match self.lookup(n) {
                Some(slot) => self.cx.view_rev.contains(&slot),
                None => {
                    matches!(self.get_sym(n), Some(Sym::Global(id)) if self.global_view_rev.contains(&id))
                }
            },
            _ => false,
        }
    }

    /// The params of a generic fn given a reversed view, whose instance is
    /// its own: there the param's `.keys` walk backwards, as at the call.
    pub(super) fn reversed_params(
        &mut self,
        decl: &ast::FnDecl,
        params: &[&ast::Param],
        receiver: Option<&Arg>,
        args: &[ast::Arg],
        generics: &[String],
    ) -> Vec<String> {
        let generic = |p: &ast::Param| {
            matches!(p.ty.as_ref().map(|t| &t.kind),
                Some(ast::TypeKind::Named { name, args }) if args.is_empty() && generics.contains(&name.text))
        };
        let mut out = Vec::new();
        if let (Some(Arg::Value(e)), Some(p)) = (receiver, decl.params.first()) {
            if generic(p) && self.is_reversed(e) {
                out.push(p.name.text.clone());
            }
        }
        for (pos, p) in params.iter().enumerate() {
            let arg = args
                .iter()
                .find(|a| a.name.as_ref().is_some_and(|n| n.text == p.name.text))
                .or_else(|| args.get(pos).filter(|a| a.name.is_none()));
            if generic(p) && arg.is_some_and(|a| self.reversed_arg(&a.value)) {
                out.push(p.name.text.clone());
            }
            if is_opt_mut(p) && arg.is_some_and(|a| a.mutable) {
                out.push(format!("?mut {}", p.name.text));
            }
        }
        out
    }

    /// What a fn's params keep known in its body: the range of a ranged
    /// param, and that a param given a reversed view walks backwards.
    pub(super) fn bind_param_facts(&mut self, decl: &ast::FnDecl, params: &mut [ParamDef]) {
        for p in &decl.params {
            let Some(&slot) = self.cx.scopes[0].get(&p.name.text) else {
                continue;
            };
            if let Some(rinfo) = p.ty.as_ref().and_then(|t| self.range_info_from_ast(t)) {
                if let Some(param) = params.get_mut(slot) {
                    param.range = rinfo.spec();
                }
                self.cx.local_ranges.insert(slot, rinfo);
            }
            let declared_rev = matches!(p.ty.as_ref().map(|t| &t.kind),
                Some(ast::TypeKind::Named { name, args }) if name.text == "Rev" && args.len() == 1);
            if declared_rev || self.gen_env.contains_key(&rev_param_key(&p.name.text)) {
                self.cx.view_rev.insert(slot);
            }
        }
    }
}

pub(super) fn opt_mut_key(name: &str) -> String {
    format!("__opt_mut_param_{name}")
}

/// The env key of a call-shape marker from `reversed_params`: a reversed
/// view, or `?mut p` for a `?mut T` param the call passes `mut x` for.
pub(super) fn param_marker(name: &str) -> String {
    match name.strip_prefix("?mut ") {
        Some(n) => opt_mut_key(n),
        None => rev_param_key(name),
    }
}

/// `seen: ?mut T`: a param the call may leave out, and the fn changes in
/// place when it is given.
pub(super) fn is_opt_mut(p: &ast::Param) -> bool {
    matches!(&p.ty, Some(ast::Type { kind: ast::TypeKind::Optional(t), .. })
        if matches!(t.kind, ast::TypeKind::Mut(_)))
}

/// A fn's decl for one call shape of its `?mut T` params: one the call
/// passes `mut x` for is `mut p: T`, one it leaves out is `p: ?T = None`.
/// Each shape is its own instance, so neither engine needs a nullable
/// reference.
pub(super) fn opt_mut_instance<'d>(decl: &'d ast::FnDecl, shape: &[String]) -> &'d ast::FnDecl {
    if !decl.params.iter().any(is_opt_mut) {
        return decl;
    }
    let mut decl = decl.clone();
    for p in decl.params.iter_mut() {
        let inner = match p.ty.as_ref().map(|t| &t.kind) {
            Some(ast::TypeKind::Optional(t)) => match &t.kind {
                ast::TypeKind::Mut(inner) => (**inner).clone(),
                _ => continue,
            },
            _ => continue,
        };
        let given = shape
            .iter()
            .any(|s| s.strip_prefix("?mut ") == Some(p.name.text.as_str()));
        if given {
            p.mutable = true;
            p.ty = Some(inner);
            p.default = None;
        } else if let Some(ty) = &mut p.ty {
            ty.kind = ast::TypeKind::Optional(Box::new(inner));
        }
    }
    Box::leak(Box::new(decl))
}

impl<'a> Lowerer<'a> {
    /// `if let s = seen:` in an instance whose call passed `mut x` for the
    /// `?mut` param `seen`: `s` is `seen` itself, so its writes reach `x`.
    pub(super) fn opt_mut_alias(&mut self, pat: &ast::Pattern, value: &ast::Expr) -> Option<Expr> {
        let (ast::PatternKind::Name(s), ast::ExprKind::Name(n)) = (&pat.kind, &value.kind) else {
            return None;
        };
        if !self.gen_env.contains_key(&opt_mut_key(n)) {
            return None;
        }
        let slot = self.lookup(n)?;
        self.cx.scopes.last_mut()?.insert(s.clone(), slot);
        Some(self.mk(ExprKind::Bool(true), Ty::Bool, value.span))
    }
}

/// Depth of nested `List.filled(_, ...)` calls wrapping a bare `none` literal.
pub(super) fn filled_none_depth(e: &ast::Expr) -> Option<usize> {
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
