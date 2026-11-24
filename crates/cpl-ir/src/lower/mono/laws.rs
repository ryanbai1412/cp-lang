//! Lowering `laws_hold` into nested assertion loops over every law.

use super::*;
use crate::invariant;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// Lowers `laws_hold(obj, samples_or_kw...)` into nested assertion loops checking every `law`.
    pub(in crate::lower) fn lower_laws_hold(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        if args.is_empty() {
            self.report(span, msgs::ir::WRONG_ARG_COUNT_LAWS_HOLD_TAKES_AN, &[]);
            return self.err_expr(span);
        }
        let obj = self.expr(&args[0].value, None);
        if let Ty::Fn(param_tys, ret_ty) = &obj.ty {
            if param_tys.len() == 2 && param_tys[0] == param_tys[1] && **ret_ty == param_tys[0] {
                let elem_ty = param_tys[0].clone();
                let samples = if args.len() >= 2 {
                    self.expr(&args[1].value, None)
                } else {
                    return self.mk(ExprKind::Bool(true), Ty::Bool, span);
                };
                return self.do_expr(Some(&Ty::Bool), span, |lw, out| {
                    let fn_slot = lw.temp(obj.ty.clone());
                    out.push_let(fn_slot, obj);
                    let samp_slot = lw.temp(samples.ty.clone());
                    out.push_let(samp_slot, samples);
                    let ok_slot = lw.temp(Ty::Bool);
                    out.push_let(ok_slot, lw.mk(ExprKind::Bool(true), Ty::Bool, span));
                    let a_slot = lw.temp(elem_ty.clone());
                    let b_slot = lw.temp(elem_ty.clone());
                    let c_slot = lw.temp(elem_ty.clone());
                    let call2 = |lw: &Self, x: Expr, y: Expr| -> Expr {
                        lw.mk_call_expr(
                            lw.local(fn_slot, span),
                            vec![Arg::Value(x), Arg::Value(y)],
                            elem_ty.clone(),
                            span,
                        )
                    };
                    let ab = call2(lw, lw.local(a_slot, span), lw.local(b_slot, span));
                    let ab_c = call2(lw, ab, lw.local(c_slot, span));
                    let bc = call2(lw, lw.local(b_slot, span), lw.local(c_slot, span));
                    let a_bc = call2(lw, lw.local(a_slot, span), bc);
                    let ne = lw.mk_cmp(CmpOp::Ne, ab_c, a_bc, Ty::Bool, span);
                    let set_false = Stmt::Assign(
                        Place {
                            kind: PlaceKind::Local(ok_slot),
                            ty: Ty::Bool,
                            span: lw.span(span),
                        },
                        AssignOp::Set,
                        lw.mk(ExprKind::Bool(false), Ty::Bool, span),
                    );
                    let lid_c = lw.fresh_loop();
                    let lid_b = lw.fresh_loop();
                    let lid_a = lw.fresh_loop();
                    let loop_c = Stmt::For(
                        lid_c,
                        c_slot,
                        Iter::Seq(lw.local(samp_slot, span), false),
                        Block {
                            stmts: vec![Stmt::If(
                                ne,
                                Block {
                                    stmts: vec![set_false, Stmt::Break(None)],
                                },
                                Block::default(),
                            )],
                        },
                    );
                    let loop_b = Stmt::For(
                        lid_b,
                        b_slot,
                        Iter::Seq(lw.local(samp_slot, span), false),
                        Block {
                            stmts: vec![loop_c],
                        },
                    );
                    let loop_a = Stmt::For(
                        lid_a,
                        a_slot,
                        Iter::Seq(lw.local(samp_slot, span), false),
                        Block {
                            stmts: vec![loop_b],
                        },
                    );
                    out.push(loop_a);
                    out.push_give(lw.local(ok_slot, span));
                });
            }
        }
        let Ty::Struct(sid) = obj.ty else {
            self.report(
                args[0].value.span,
                msgs::ir::TYPE_MISMATCH_LAWS_HOLD_EXPECTS_A,
                &[],
            );
            return self.err_expr(span);
        };
        let laws = self.struct_laws(sid);
        let struct_env = self.structs[sid].env.clone();
        let struct_fields = self.structs[sid].def.fields.clone();

        // Lower positional default samples and named sample lists (e.g. `T: [...], F: [...]`).
        let mut default_samples: Option<Expr> = None;
        let mut named_samples: HashMap<String, Expr> = HashMap::new();
        for arg in &args[1..] {
            let v = self.expr(&arg.value, None);
            if let Some(n) = &arg.name {
                named_samples.insert(n.text.clone(), v);
            } else {
                default_samples = Some(v);
            }
        }

        self.do_expr(Some(&Ty::Bool), span, |lw, out| {
            let obj_slot = lw.temp(Ty::Struct(sid));
            out.push_let(obj_slot, obj);

            // Materialize sample lists in locals once.
            let def_slot = default_samples.map(|e| {
                let s = lw.temp(e.ty.clone());
                out.push_let(s, e);
                s
            });
            let mut kw_slots: HashMap<String, Slot> = HashMap::new();
            for (k, e) in named_samples {
                let s = lw.temp(e.ty.clone());
                out.push_let(s, e);
                kw_slots.insert(k, s);
            }

            // Bind fields of `obj` into locals so `op`, `id`, `inv`, `lt`, `compose`, `apply` work in laws.
            lw.cx.scopes.push(HashMap::new());
            for (fi, (fname, fty)) in struct_fields.iter().enumerate() {
                let fslot = lw.new_slot(fname, fty.clone());
                let fexpr = Expr {
                    kind: ExprKind::Field(Box::new(lw.local(obj_slot, span)), fi),
                    ty: fty.clone(),
                    span: lw.span(span),
                };
                out.push_let(fslot, fexpr);
                lw.cx.scopes.top_mut().insert(fname.clone(), fslot);
            }
            let saved_recv = lw.cx.receiver.replace((obj_slot, sid));

            for law in &laws {
                let saved_mod = std::mem::replace(&mut lw.cur_mod, law.mod_id);
                let saved_offset = std::mem::replace(&mut lw.offset, law.offset);
                let saved_env = std::mem::replace(&mut lw.gen_env, struct_env.clone());
                let saved_self = lw.self_struct.replace(sid);
                let saved_fns = std::mem::replace(&mut lw.law_fns, law.trait_fns.clone());

                // Resolve each law parameter's sample list slot.
                let mut pools: Vec<(String, Slot, Ty)> = Vec::new();
                for p in law.params {
                    let tn = match p.ty.as_ref().map(|t| &t.kind) {
                        Some(ast::TypeKind::Named { name, args }) if args.is_empty() => {
                            Some(name.text.as_str())
                        }
                        _ => None,
                    };
                    let slot = tn.and_then(|n| kw_slots.get(n).copied()).or(def_slot);
                    if let Some(s) = slot {
                        let elem_ty = match &lw.cx.locals[s].ty {
                            Ty::List(e) => (**e).clone(),
                            Ty::Array { elem, .. } => (**elem).clone(),
                            other => other.clone(),
                        };
                        pools.push((p.name.text.clone(), s, elem_ty));
                    }
                }
                if pools.len() == law.params.len() {
                    let stmts = lw.emit_law_loops(&pools, 0, law, span);
                    out.extend(stmts);
                }

                lw.cur_mod = saved_mod;
                lw.offset = saved_offset;
                lw.gen_env = saved_env;
                lw.self_struct = saved_self;
                lw.law_fns = saved_fns;
            }

            lw.cx.receiver = saved_recv;
            lw.cx.scopes.pop();
            let ok = lw.mk(ExprKind::Bool(true), Ty::Bool, span);
            out.push_give(ok);
        })
    }

    pub(in crate::lower) fn emit_law_loops(
        &mut self,
        pools: &[(String, Slot, Ty)],
        idx: usize,
        law: &LawInfo<'a>,
        span: Span,
    ) -> Vec<Stmt> {
        if idx == pools.len() {
            let cond = self.expr_as(law.body, &Ty::Bool);
            return vec![Stmt::Assert {
                cond,
                message: None,
                text: format!("law {}", law.name),
                line: self.line(law.body.span),
                note: None,
                span: self.span(span),
                kept: false,
            }];
        }
        let (pname, pool_slot, elem_ty) = &pools[idx];
        let var_slot = self.new_slot(pname, elem_ty.clone());
        let lid = self.open_loop(None);
        self.cx.scopes.push(HashMap::new());
        self.cx.scopes.top_mut().insert(pname.clone(), var_slot);
        let body = self.emit_law_loops(pools, idx + 1, law, span);
        self.cx.scopes.pop();
        self.cx.loops.pop();
        let seq = self.local(*pool_slot, span);
        vec![Stmt::For(
            lid,
            var_slot,
            Iter::Seq(seq, false),
            Block { stmts: body },
        )]
    }

    pub(in crate::lower) fn dummy_fn_decl(&self) -> &'a ast::FnDecl {
        // Only used for synthetic preamble init fns whose `state` is already `State::Done`.
        for m in &self.mods {
            for item in &m.items {
                for s in &item.syntax.parsed.stmts {
                    if let ast::StmtKind::Fn(f) = &s.kind {
                        return f;
                    }
                    if let ast::StmtKind::Struct(st) = &s.kind {
                        for m in &st.methods {
                            if let ast::StmtKind::Fn(f) = &m.kind {
                                return f;
                            }
                        }
                    }
                }
            }
        }
        invariant::broken_invariant("the loaded modules declare a fn")
    }
}
