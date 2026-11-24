//! A generic a call leaves open, bound by what the code after the `let` passes the value:
//! `var w = undoing(s, (mut d, (u, v)) => ..)` takes `T` from a later `w.push((V(0), V(1)))`.

use super::*;

/// Where the `let` of a generic call is (module, item offset, call span) and the method
/// calls the rest of its block makes on the name: method and positional arguments.
pub(super) type LaterCalls = (ModId, u32, Span, Vec<(String, Vec<ast::Expr>)>);

impl<'a> Lowerer<'a> {
    /// Before `let name = f(..)` with `f` generic: note what the rest of the block calls on `name`.
    pub(super) fn note_later_calls(&mut self, stmt: &ast::Stmt, rest: &[ast::Stmt]) {
        let ast::StmtKind::Let {
            pat,
            ty: None,
            init: Some(init),
            ..
        } = &stmt.kind
        else {
            return;
        };
        let (ast::PatternKind::Name(name), ast::ExprKind::Call { callee, .. }) =
            (&pat.kind, &init.kind)
        else {
            return;
        };
        if !matches!(self.sym(callee), Some(Sym::GenFn(_))) {
            return;
        }
        let mut find = Calls {
            name,
            calls: Vec::new(),
        };
        for s in rest {
            cpl_syntax::visit::walk_stmt(s, &mut find);
        }
        if !find.calls.is_empty() {
            self.later_calls = Some((self.cur_mod, self.offset, init.span, find.calls));
        }
    }

    /// The type of the first value the rest of the block pushes onto `var_name`.
    pub(super) fn peek_list_elem_type(&mut self, var_name: &str, rest: &[ast::Stmt]) -> Option<Ty> {
        let mut find = Calls {
            name: var_name,
            calls: Vec::new(),
        };
        for s in rest {
            cpl_syntax::visit::walk_stmt(s, &mut find);
        }
        let (_, arg) = find
            .calls
            .iter()
            .find(|(m, a)| m == "push" && a.len() == 1)?;
        self.probe_ty_silent(&arg[0])
    }

    /// Binds each generic of `decl` still open that its result passes to a struct, from the
    /// later calls on the value whose parameters take that struct's generic (R06.4.1).
    pub(super) fn bind_from_later_calls(
        &mut self,
        decl: &ast::FnDecl,
        span: Span,
        tpl_mod: ModId,
        gen_names: &[String],
        inferred: &mut GenEnv,
    ) {
        let here = (self.cur_mod, self.offset, span);
        if !matches!(&self.later_calls, Some((m, o, s, _)) if (*m, *o, *s) == here) {
            return;
        }
        let Some((.., calls)) = self.later_calls.take() else {
            return;
        };
        let Some(ast::Type {
            kind: ast::TypeKind::Named { name, args },
            ..
        }) = &decl.ret
        else {
            return;
        };
        let Some(&Sym::GenStruct(gid)) = self.mod_syms[tpl_mod].get(&name.text) else {
            return;
        };
        let (smod, sdecl) = (self.gen_structs[gid].mod_id, self.gen_structs[gid].decl);
        for (k, arg) in args.iter().enumerate() {
            let ast::TypeKind::Named {
                name: g,
                args: none,
            } = &arg.kind
            else {
                continue;
            };
            let Some(sg) = sdecl.generics.get(k) else {
                continue;
            };
            if !none.is_empty() || !gen_names.contains(&g.text) || inferred.contains_key(&g.text) {
                continue;
            }
            let sg = vec![sg.text.clone()];
            'calls: for (method, cargs) in &calls {
                let Some(m) = sdecl.methods.iter().find_map(|s| match &s.kind {
                    ast::StmtKind::Fn(f) if f.name.text == *method => Some(f),
                    _ => None,
                }) else {
                    continue;
                };
                let params = m.params.iter().filter(|p| p.name.text != "self");
                for (p, carg) in params.zip(cargs) {
                    let Some(pty) = &p.ty else { continue };
                    let Some(ty) = self.probe_ty_silent(carg) else {
                        continue;
                    };
                    let mut env = GenEnv::new();
                    self.unify_ast_ty(pty, &ty, smod, &sg, &mut env);
                    if let Some(bound) = env.remove(&sg[0]) {
                        inferred.insert(g.text.clone(), bound);
                        break 'calls;
                    }
                }
            }
        }
    }
}

struct Calls<'n> {
    name: &'n str,
    calls: Vec<(String, Vec<ast::Expr>)>,
}

impl cpl_syntax::visit::Visitor for Calls<'_> {
    fn expr(&mut self, expr: &ast::Expr) {
        let ast::ExprKind::Call { callee, args } = &expr.kind else {
            return;
        };
        let ast::ExprKind::Field { base, name } = &callee.kind else {
            return;
        };
        if matches!(&base.kind, ast::ExprKind::Name(n) if n == self.name) {
            let positional = args.iter().take_while(|a| a.name.is_none());
            self.calls.push((
                name.text.clone(),
                positional.map(|a| a.value.clone()).collect(),
            ));
        }
    }
}
