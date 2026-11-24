//! The methods `Rev<S>` declares (`a.rev.take(k)`), called on a list view
//! known to walk backwards: each runs its std body inline (an expression, or
//! a block giving its last value), with `self` the view and `self.src` the
//! sequence it reverses.

use super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn rev_method(
        &mut self,
        value: Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        span: Span,
    ) -> Result<Expr, Expr> {
        if !matches!(value.ty, Ty::ListView(..)) || !self.is_reversed(&value) {
            return Err(value);
        }
        let Some(gid) = self.view_struct("Rev") else {
            return Err(value);
        };
        let (tpl_mod, tpl_offset, decl) = {
            let t = &self.gen_structs[gid];
            (t.mod_id, t.offset, t.decl)
        };
        let found = decl.methods.iter().find_map(|stmt| match &stmt.kind {
            ast::StmtKind::Fn(f) if f.name.text == name.text => match &f.body {
                ast::FnBody::Expr(body) if body.leaves() => None,
                ast::FnBody::Expr(_) | ast::FnBody::Block(_) => Some((f, &f.body)),
                ast::FnBody::None => None,
            },
            _ => None,
        });
        let Some((f, body)) = found else {
            return Err(value);
        };
        let params: Vec<&ast::Param> = f.params.iter().skip(1).collect();
        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
        let tys: Vec<Option<Ty>> = params
            .iter()
            .map(|p| p.ty.as_ref().map(|t| self.resolve_ty(t)))
            .collect();
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        let mut given = Vec::with_capacity(params.len());
        for (pos, (p, ty)) in params.iter().zip(&tys).enumerate() {
            let arg = args
                .iter()
                .find(|a| a.name.as_ref().is_some_and(|n| n.text == p.label_text()))
                .or_else(|| args.get(pos).filter(|a| a.name.is_none()));
            given.push(arg.map(|a| match ty {
                Some(ty) => self.expect(&a.value, ty),
                None => self.expr(&a.value, None),
            }));
        }
        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
        let saved_env = std::mem::take(&mut self.gen_env);
        let saved_self = self.self_struct.take();
        self.cx.scopes.push(HashMap::new());
        let this = self.temp(value.ty.clone());
        self.cx.view_rev.insert(this);
        self.cx.pending.push(Stmt::Let(this, Some(value)));
        self.bind_local("self", this);
        for ((p, ty), arg) in params.iter().zip(&tys).zip(given) {
            let arg = match (arg, &p.default) {
                (Some(arg), _) => arg,
                (None, Some(default)) => match ty {
                    Some(ty) => self.expect(default, ty),
                    None => self.expr(default, None),
                },
                (None, None) => self.err_expr(span),
            };
            let slot = self.temp(arg.ty.clone());
            self.cx.pending.push(Stmt::Let(slot, Some(arg)));
            self.bind_local(&p.name.text, slot);
        }
        let out = match body {
            ast::FnBody::Block(block) => {
                let tail = if f.ret.is_some() {
                    Tail::Give
                } else {
                    Tail::Discard
                };
                self.do_expr(None, span, |lw, out| {
                    out.extend(lw.block(block, tail).stmts)
                })
            }
            ast::FnBody::Expr(body) => self.expr(body, None),
            ast::FnBody::None => self.err_expr(span),
        };
        self.cx.scopes.pop();
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;
        Ok(out)
    }

    /// The std struct `name` that types a built-in view, wherever the
    /// caller's module stands.
    pub(in crate::lower) fn view_struct(&self, name: &str) -> Option<GenStructId> {
        let found = |m: ModId| match self.mod_syms[m].get(name) {
            Some(Sym::GenStruct(gid)) => Some(*gid),
            _ => None,
        };
        self.std_mod.into_iter().find_map(found).or_else(|| {
            self.mods
                .iter()
                .flat_map(|a| a.modules())
                .filter(|module| module.name.starts_with("std."))
                .filter_map(|module| self.path_to_mod.get(&module.path).copied())
                .find_map(found)
        })
    }

    fn bind_local(&mut self, name: &str, slot: Slot) {
        self.cx
            .scopes
            .last_mut()
            .expect("a scope")
            .insert(name.to_string(), slot);
    }
}
