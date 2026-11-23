//! Contexts are lexical (R09.1.3): a top-level fn that doesn't declare
//! `uses ctx` reads the value in force where it is written, and passes that
//! value on to the `uses ctx` fns it calls, whatever its caller has in force.

use super::*;

impl<'a> Lowerer<'a> {
    /// A read of the global `gid`: the lexical value when it is a context the
    /// fn being lowered doesn't declare it `uses`.
    pub(super) fn global_read(&mut self, gid: GlobalId, span: Span) -> Expr {
        match self.lexical_context(gid) {
            Some(name) => self.lexical_value(&name, span),
            None => self.mk(ExprKind::Global(gid), self.globals[gid].ty.clone(), span),
        }
    }

    fn lexical_context(&self, gid: GlobalId) -> Option<String> {
        let uses = self.cx.lexical_uses.as_ref()?;
        let (name, _) = self.context_defs.iter().find(|(_, (g, _))| *g == gid)?;
        (!uses.contains(name)).then(|| name.clone())
    }

    /// A call to a `uses ctx` fn from a fn that doesn't declare `uses ctx`:
    /// the callee sees the caller's lexical value, then the dynamic one is
    /// put back.
    pub(in crate::lower) fn lexical_call(
        &mut self,
        whole: &ast::Expr,
        callee: &ast::Expr,
        args: &[ast::Arg],
        hint: Option<&Ty>,
    ) -> Option<Expr> {
        let uses = self.cx.lexical_uses.as_ref()?;
        if self.lexical_call_at == Some(whole.span) {
            return None;
        }
        let decl = match self.sym(callee)? {
            Sym::Fn(id) => self.sigs[id].decl,
            Sym::GenFn(gfid) => self.gen_fns[gfid].decl,
            _ => return None,
        };
        let reset: Vec<String> = (decl.uses.iter())
            .map(|u| u.text.clone())
            .filter(|c| !uses.contains(c) && (c == "io" || self.context_defs.contains_key(c)))
            .collect();
        if reset.is_empty() {
            return None;
        }
        let span = whole.span;
        let mut stmts = Vec::new();
        let mut saved = Vec::new();
        for name in &reset {
            let gid = self.context_global(name);
            let ty = self.globals[gid].ty.clone();
            let save = self.temp(ty.clone());
            stmts.push(Stmt::Let(
                save,
                Some(self.mk(ExprKind::Global(gid), ty.clone(), span)),
            ));
            let value = self.lexical_value(name, span);
            let place = Place {
                kind: PlaceKind::Global(gid),
                ty,
                span: self.span(span),
            };
            stmts.push(Stmt::Assign(place.clone(), AssignOp::Set, value));
            saved.push((place, save));
        }
        let held = self.lexical_call_at.replace(whole.span);
        let call = self.call(whole, callee, args, hint);
        self.lexical_call_at = held;
        let ty = call.ty.clone();
        let result = self.temp(ty.clone());
        stmts.push(Stmt::Let(result, Some(call)));
        for (place, save) in saved {
            let back = self.local(save, span);
            stmts.push(Stmt::Assign(place, AssignOp::Set, back));
        }
        stmts.push(Stmt::Give(self.local(result, span)));
        Some(self.mk(ExprKind::Do(Block { stmts }), ty, span))
    }

    fn context_global(&mut self, name: &str) -> GlobalId {
        match self.context_defs.get(name) {
            Some((gid, _)) => *gid,
            None => self.ensure_io_global(),
        }
    }

    /// The value of context `name` where this code is written: each field's
    /// `with` value in scope, else its default.
    fn lexical_value(&mut self, name: &str, span: Span) -> Expr {
        let gid = self.context_global(name);
        let ty = self.globals[gid].ty.clone();
        let Ty::Record(fields) = &ty else {
            return self.mk(ExprKind::Global(gid), ty, span);
        };
        let defaults = self.context_defaults(name, gid);
        let mut values = Vec::new();
        for (i, (field, fty)) in fields.iter().enumerate() {
            let value = match self.context(name, field) {
                Some(e) => self.expect(&e, fty),
                None => match defaults.get(i).cloned().flatten() {
                    Some(DefaultOf::Ast(d)) => self.expect(&d, fty),
                    Some(DefaultOf::Ir(e)) => e,
                    None => self.mk(ExprKind::Int(0), fty.clone(), span),
                },
            };
            values.push(value);
        }
        self.mk(ExprKind::Tuple(values), ty, span)
    }

    /// Each field's default: from the `context` declaration, or for `io` the
    /// value its global starts with.
    fn context_defaults(&self, name: &str, gid: GlobalId) -> Vec<Option<DefaultOf>> {
        let declared = self.context_defs.contains_key(name);
        for m in (0..self.mods.len()).filter(|_| declared) {
            for (_, stmt) in self.mod_items(m) {
                if let ast::StmtKind::Context { name: n, fields } = &stmt.kind {
                    if n.text == name {
                        return fields
                            .iter()
                            .map(|f| f.default.clone().map(DefaultOf::Ast))
                            .collect();
                    }
                }
            }
        }
        self.preamble_stmts
            .iter()
            .find_map(|s| match s {
                Stmt::Assign(
                    Place {
                        kind: PlaceKind::Global(g),
                        ..
                    },
                    _,
                    init,
                ) if *g == gid => match &init.kind {
                    ExprKind::Tuple(fs) => {
                        Some(fs.iter().cloned().map(|e| Some(DefaultOf::Ir(e))).collect())
                    }
                    _ => None,
                },
                _ => None,
            })
            .unwrap_or_default()
    }
}

#[derive(Clone)]
enum DefaultOf {
    Ast(ast::Expr),
    Ir(Expr),
}
