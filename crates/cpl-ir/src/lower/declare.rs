//! `let` and `var` declarations: a global at the top level of the file, a
//! local anywhere else, and what the name keeps known about its view.

use super::*;

impl<'a> Lowerer<'a> {
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
                Some(Ty::List(_) | Ty::ListView(..) | Ty::MutView(_))
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
        }
        out
    }

    /// What a fn's params keep known in its body: the range of a ranged
    /// param, and that a param given a reversed view walks backwards.
    pub(super) fn bind_param_facts(&mut self, decl: &ast::FnDecl) {
        for p in &decl.params {
            let Some(&slot) = self.cx.scopes[0].get(&p.name.text) else {
                continue;
            };
            if let Some(rinfo) = p.ty.as_ref().and_then(|t| self.range_info_from_ast(t)) {
                self.cx.local_ranges.insert(slot, rinfo);
            }
            if self.gen_env.contains_key(&rev_param_key(&p.name.text)) {
                self.cx.view_rev.insert(slot);
            }
        }
    }
}
