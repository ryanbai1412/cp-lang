//! Inlining `lends` / `lends mut` projections and structural struct
//! projection coercion.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// Inlines a `lends` / `lends mut` projection for a **read** expression: `base.proj(args)`.
    pub(in crate::lower) fn lower_projection_read(
        &mut self,
        proj: ProjectionInfo<'a>,
        base: &ast::Expr,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let recv_place = self.try_place(base);
        let recv_val = if recv_place.is_none() {
            Some(self.expr(base, None))
        } else {
            None
        };
        let (pre_stmts, lend_place_ast, post_stmts) = split_projection_body(proj.decl);
        let Some(lend_place_ast) = lend_place_ast else {
            self.report(span, msgs::ir::UNSUPPORTED_PROJECTION_HAS_NO_LEND, &[]);
            return self.err_expr(span);
        };

        self.do_expr(None, span, |lw, out| {
            let p_place = match recv_place {
                Some(p) => p,
                None => {
                    let v = recv_val.unwrap();
                    let slot = lw.temp(v.ty.clone());
                    let sp = v.span;
                    out.push_let(slot, v);
                    Place {
                        kind: PlaceKind::Local(slot),
                        ty: Ty::Struct(proj.owner),
                        span: sp,
                    }
                }
            };
            lw.with_projection_scope(&proj, p_place, args, out, |lw, out| {
                for s in pre_stmts {
                    lw.stmt(s, out);
                }
                let val = lw.expr(lend_place_ast, None);
                let result_slot = lw.temp(val.ty.clone());
                let res_ty = val.ty.clone();
                out.push_let(result_slot, val);
                for s in post_stmts {
                    lw.stmt(s, out);
                }
                if let Some(frame) = lw.cx.dos.last_mut() {
                    frame.ty = Some(res_ty);
                }
                let res = lw.local(result_slot, span);
                out.push_give(res);
            });
        })
    }

    /// Inlines a `lends mut` projection for an **assignment** (`base.proj(args) op= rhs`).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::lower) fn lower_projection_assign(
        &mut self,
        proj: ProjectionInfo<'a>,
        base: &ast::Expr,
        args: &[ast::Arg],
        op: ast::AssignOp,
        rhs: &ast::Expr,
        span: Span,
        out: &mut Vec<Stmt>,
    ) {
        let rhs_val = self.expr(rhs, None);
        let rhs_slot = self.temp(rhs_val.ty.clone());
        let rhs_ty = rhs_val.ty.clone();
        out.push_let(rhs_slot, rhs_val);

        let recv_place = self.place(base);
        let (pre_stmts, lend_place_ast, post_stmts) = split_projection_body(proj.decl);
        let Some(lend_place_ast) = lend_place_ast else {
            self.report(span, msgs::ir::UNSUPPORTED_PROJECTION_HAS_NO_LEND, &[]);
            return;
        };

        self.with_projection_scope(&proj, recv_place, args, out, |lw, out| {
            for s in pre_stmts {
                lw.stmt(s, out);
            }
            let target_place = lw.place(lend_place_ast);
            let val_expr = lw.local(rhs_slot, rhs.span);
            let coerced = lw.coerce(val_expr, &target_place.ty, rhs.span);
            lw.emit_assign_place(target_place, op, coerced, span, out);
            for s in post_stmts {
                lw.stmt(s, out);
            }
        });
        let _ = rhs_ty;
    }

    /// Inlines a `lends mut` projection passed as a `mut` call argument (`f(mut base.proj(args))`).
    pub(in crate::lower) fn lower_projection_mut_call(
        &mut self,
        proj: ProjectionInfo<'a>,
        base: &ast::Expr,
        args: &[ast::Arg],
        span: Span,
        out: &mut Vec<Stmt>,
        post_out: &mut Vec<Stmt>,
    ) -> Place {
        let recv_place = self.place(base);
        let (pre_stmts, lend_place_ast, post_stmts) = split_projection_body(proj.decl);
        let Some(lend_place_ast) = lend_place_ast else {
            self.report(span, msgs::ir::UNSUPPORTED_PROJECTION_HAS_NO_LEND, &[]);
            return Place {
                kind: PlaceKind::Local(0),
                ty: Ty::Error,
                span: self.span(span),
            };
        };
        let mut lent_place = None;
        self.with_projection_scope(&proj, recv_place, args, out, |lw, out| {
            for s in pre_stmts {
                lw.stmt(s, out);
            }
            lent_place = Some(lw.place(lend_place_ast));
            for s in post_stmts {
                lw.stmt(s, post_out);
            }
        });
        lent_place.unwrap()
    }

    pub(in crate::lower) fn with_projection_scope(
        &mut self,
        proj: &ProjectionInfo<'a>,
        recv_place: Place,
        call_args: &[ast::Arg],
        out: &mut Vec<Stmt>,
        f: impl FnOnce(&mut Self, &mut Vec<Stmt>),
    ) {
        // Bind arguments in the caller's module/env first.
        let ast_params: Vec<&ast::Param> = proj
            .decl
            .params
            .iter()
            .enumerate()
            .filter(|(i, p)| !(*i == 0 && p.name.text == "self"))
            .map(|(_, p)| p)
            .collect();

        // Resolve parameter types in the projection's module/env.
        let saved_mod = std::mem::replace(&mut self.cur_mod, proj.mod_id);
        let saved_offset = std::mem::replace(&mut self.offset, proj.offset);
        let saved_env = std::mem::replace(&mut self.gen_env, proj.env.clone());
        let saved_self = self.self_struct.replace(proj.owner);
        let param_tys: Vec<(String, Ty)> = ast_params
            .iter()
            .map(|p| {
                let ty = p.ty.as_ref().map(|t| self.resolve_ty(t)).unwrap_or(Ty::Int);
                (p.name.text.clone(), ty)
            })
            .collect();
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;

        // Evaluate arguments in caller context.
        let mut arg_slots = Vec::new();
        let mut arg_globals = Vec::new();
        for ((pname, pty), arg) in param_tys.iter().zip(call_args) {
            let v = self.expect(&arg.value, pty);
            // A read-only variable is lent, not copied: the checker holds it
            // while the result lives.
            if !arg.mutable && v.ty == *pty {
                match v.kind {
                    ExprKind::Local(src) => {
                        arg_slots.push((pname.clone(), src));
                        continue;
                    }
                    ExprKind::Global(g) => {
                        let place = Place {
                            kind: PlaceKind::Global(g),
                            ty: v.ty,
                            span: v.span,
                        };
                        arg_globals.push((pname.clone(), place));
                        continue;
                    }
                    _ => {}
                }
            }
            let slot = self.new_slot(pname, pty.clone());
            out.push_let(slot, v);
            arg_slots.push((pname.clone(), slot));
        }

        // Now switch into projection's module/env with `inline_self` active.
        let saved_mod = std::mem::replace(&mut self.cur_mod, proj.mod_id);
        let saved_offset = std::mem::replace(&mut self.offset, proj.offset);
        let saved_env = std::mem::replace(&mut self.gen_env, proj.env.clone());
        let saved_self = self.self_struct.replace(proj.owner);
        let saved_inline = self.cx.inline_self.replace((recv_place, proj.owner));
        let saved_aliases = self.cx.mut_aliases.clone();
        for (pname, _) in &param_tys {
            self.cx.mut_aliases.remove(pname);
        }
        self.cx.mut_aliases.extend(arg_globals);
        self.cx.scopes.push(HashMap::new());
        for (pname, slot) in arg_slots {
            self.cx.scopes.last_mut().unwrap().insert(pname, slot);
        }

        f(self, out);

        self.cx.scopes.pop();
        self.cx.mut_aliases = saved_aliases;
        self.cx.inline_self = saved_inline;
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;
    }

    /// Checks whether a struct `have_id` structurally contains every field of `want_id` (e.g., `Group<T>` -> `Monoid<T>`).
    pub(in crate::lower) fn coerce_struct_projection(
        &mut self,
        e: Expr,
        have_id: StructId,
        want_id: StructId,
    ) -> Option<Expr> {
        let want_fields = self.structs[want_id].def.fields.clone();
        let have_fields = &self.structs[have_id].def.fields;
        let mut mapping = Vec::with_capacity(want_fields.len());
        for (wname, wty) in &want_fields {
            let pos = have_fields
                .iter()
                .position(|(hname, hty)| hname == wname && hty == wty)?;
            mapping.push((pos, wty.clone()));
        }
        let span = e.span;
        let tmp = self.temp(Ty::Struct(have_id));
        let mut stmts = vec![Stmt::Let(tmp, Some(e))];
        let mut vals = Vec::with_capacity(mapping.len());
        for (pos, fty) in mapping {
            vals.push(Expr {
                kind: ExprKind::Field(
                    Box::new(Expr {
                        kind: ExprKind::Local(tmp),
                        ty: Ty::Struct(have_id),
                        span,
                    }),
                    pos,
                ),
                ty: fty,
                span,
            });
        }
        stmts.push_give(Expr {
            kind: ExprKind::Tuple(vals),
            ty: Ty::Struct(want_id),
            span,
        });
        Some(Expr {
            kind: ExprKind::Do(Block { stmts }),
            ty: Ty::Struct(want_id),
            span,
        })
    }
}
