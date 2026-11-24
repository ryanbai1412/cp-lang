//! A generator fn written inside a block is lifted to a fn of its own, so it
//! runs as lazily as a top-level one: what it captures becomes trailing
//! parameters, a `var` a `mut` one, and each call passes them on.

use super::*;
use std::collections::BTreeSet;

/// A lifted block-local generator: what it captures, as (slot where it was
/// declared, by reference, name).
pub(super) struct Lifted {
    caps: Vec<(Slot, bool, String)>,
    /// How many fns enclose the declaration: a call at the same depth reads
    /// the captured slots directly; a deeper one captures them by name.
    depth: usize,
}

/// Whether a fn body has a `yield` of its own, outside any fn or lambda in it.
pub(super) fn body_yields(body: &ast::FnBody) -> bool {
    struct Finds(bool);
    impl cpl_syntax::visit::Visitor for Finds {
        fn stmt(&mut self, stmt: &ast::Stmt) {
            self.0 |= matches!(stmt.kind, ast::StmtKind::Yield { .. });
        }
    }
    let mut v = Finds(false);
    match body {
        ast::FnBody::Block(b) => cpl_syntax::visit::walk_block(b, &mut v),
        ast::FnBody::Expr(e) => cpl_syntax::visit::walk_expr(e, &mut v),
        ast::FnBody::None => {}
    }
    v.0
}

fn named(body: &ast::FnBody) -> BTreeSet<String> {
    struct Names(BTreeSet<String>);
    impl cpl_syntax::visit::Visitor for Names {
        fn expr(&mut self, e: &ast::Expr) {
            if let ast::ExprKind::Name(n) = &e.kind {
                self.0.insert(n.clone());
            }
        }
    }
    let mut v = Names(BTreeSet::new());
    match body {
        ast::FnBody::Block(b) => cpl_syntax::visit::walk_block(b, &mut v),
        ast::FnBody::Expr(e) => cpl_syntax::visit::walk_expr(e, &mut v),
        ast::FnBody::None => {}
    }
    v.0
}

impl<'a> Lowerer<'a> {
    /// Whether the local fn `decl` is a generator this lifts; plain
    /// parameters only, since its calls pass arguments by position.
    pub(super) fn liftable_gen(decl: &ast::FnDecl) -> bool {
        matches!(
            decl.ret.as_ref().map(|t| &t.kind),
            Some(ast::TypeKind::Yields(_))
        ) && decl.generics.is_empty()
            && decl
                .params
                .iter()
                .all(|p| !p.mutable && p.default.is_none())
    }

    pub(super) fn lift_local_gen(&mut self, decl: &ast::FnDecl) {
        let decl: &'a ast::FnDecl = Box::leak(Box::new(decl.clone()));
        let (param_tys, ret_ty, _) = self.local_fn_ty(decl);
        let elem_ty = match &ret_ty {
            Ty::List(e) => (**e).clone(),
            _ => Ty::Error,
        };
        let id = self.sigs.len();
        let name = format!("{}_local{id}", decl.name.text);
        self.sigs.push(FnSig {
            name: name.clone(),
            params: decl
                .params
                .iter()
                .zip(&param_tys)
                .map(|(p, ty)| SigParam {
                    label: p.label_text().to_string(),
                    name: p.name.text.clone(),
                    ty: ty.clone(),
                    mutable: false,
                    default: None,
                })
                .collect(),
            ret: Some(ret_ty.clone()),
            assumed_unit: false,
            owner: None,
            receiver: None,
            mod_id: self.cur_mod,
            env: self.gen_env.clone(),
            decl,
            offset: self.offset,
            is_generator: true,
            new_domain_slots: Vec::new(),
            returns_fresh_domain: false,
            used_fresh_call: false,
            state: State::InProgress,
            sig: SigState::Resolved,
            asked_at: self.asked_at.callee(self.cur_mod),
        });
        self.bodies.push(None);
        if let Some(scope) = self.cx.local_syms.last_mut() {
            scope.insert(decl.name.text.clone(), Sym::Fn(id));
        }
        let depth = self.outer_cxs.len();

        let mut fresh = FnCx {
            is_closure: true,
            ..FnCx::default()
        };
        fresh.scopes.push(HashMap::new());
        fresh.local_syms.push(HashMap::new());
        fresh.ret = Some(ret_ty.clone());
        let outer_cx = std::mem::replace(&mut self.cx, fresh);
        self.outer_cxs.push(outer_cx);
        let mut params = Vec::new();
        for (p, pty) in decl.params.iter().zip(&param_tys) {
            let slot = self.new_slot(&p.name.text, pty.clone());
            self.cx.scopes[0].insert(p.name.text.clone(), slot);
            let range = p.ty.as_ref().and_then(|t| self.range_info_from_ast(t));
            if let Some(rinfo) = range {
                self.cx.local_ranges.insert(slot, rinfo);
            }
            params.push(ParamDef {
                mutable: false,
                range: None,
            });
        }
        // What the body names from the fns around it, each a parameter
        // right after the declared ones.
        let mut caps = Vec::new();
        for n in named(&decl.body) {
            if decl.params.iter().any(|p| p.name.text == n) {
                continue;
            }
            let before = self.cx.locals.len();
            let Some(inner) = self.capture_from_outer(&n) else {
                continue;
            };
            if inner != before {
                // A capture that brings slots of its own (a sorted view)
                // can't be one parameter.
                self.report(
                    decl.name.span,
                    cpl_diag::msgs::ir::UNSUPPORTED_A_BLOCK_LOCAL_GENERATOR,
                    &[&n],
                );
                continue;
            }
            self.cx.locals.truncate(before + 1);
            let cap = self.cx.captures.iter().find(|c| c.inner == inner).cloned();
            let Some(cap) = cap else { continue };
            params.push(ParamDef {
                mutable: cap.by_ref,
                range: None,
            });
            caps.push((cap.outer, cap.by_ref, n));
        }
        self.lifted.insert(id, Lifted { caps, depth });
        // Captured slots are parameters now, not captures.
        self.cx.captures.clear();

        let mut out = Vec::new();
        let acc = self.new_slot("__yields", ret_ty.clone());
        self.cx.mut_slots.insert(acc);
        out.push(Stmt::Let(
            acc,
            Some(self.mk(ExprKind::Tuple(Vec::new()), ret_ty.clone(), decl.name.span)),
        ));
        self.cx.yield_slot = Some((acc, elem_ty));
        self.gen_body(&decl.body, &mut out);
        out.push(Stmt::Return(Some(self.local(acc, decl.name.span))));

        let inner_cx = std::mem::replace(&mut self.cx, self.outer_cxs.pop().unwrap_or_default());
        self.record_narrow_locals(ScopeId::Fn(id), &inner_cx.locals, &inner_cx.local_ranges);
        self.sigs[id].state = State::Done;
        self.bodies[id] = Some(Function {
            name,
            params,
            ret: ret_ty,
            locals: inner_cx.locals,
            body: Block { stmts: out },
        });
    }

    /// A generator's body, its `yield`s going to the accumulator already set:
    /// a block or an expression that yields, or `= e` yielding each of `e`.
    pub(super) fn gen_body(&mut self, body: &ast::FnBody, out: &mut Vec<Stmt>) {
        match body {
            ast::FnBody::Block(block) => {
                let body = self.block(block, Tail::Discard);
                out.extend(body.stmts);
            }
            ast::FnBody::Expr(e) if body_yields(body) => self.tail(e, Tail::Discard, out),
            ast::FnBody::Expr(e) => {
                let each = ast::Stmt {
                    kind: ast::StmtKind::Yield {
                        value: e.clone(),
                        from: true,
                    },
                    span: e.span,
                    decorators: Vec::new(),
                    public: false,
                };
                self.stmt(&each, out);
            }
            ast::FnBody::None => {}
        }
    }

    /// A call of a lifted generator: its arguments, as for any fn, then what
    /// it captured.
    pub(super) fn lifted_call(&mut self, id: FnId, args: &[ast::Arg], span: Span) -> Option<Expr> {
        let lifted = self.lifted.remove(&id)?;
        let mut call = self.user_call(id, None, args, span);
        let mut extra = Vec::new();
        for (outer, by_ref, name) in &lifted.caps {
            let slot = if self.outer_cxs.len() == lifted.depth {
                Some(*outer)
            } else {
                self.capture_from_outer(name)
            };
            let Some(slot) = slot else {
                self.lifted.insert(id, lifted);
                return Some(self.err_expr(span));
            };
            extra.push(if *by_ref {
                Arg::Mut(Place {
                    kind: PlaceKind::Local(slot),
                    ty: self.cx.locals[slot].ty.clone(),
                    span: self.span(span),
                })
            } else {
                Arg::Value(self.local(slot, span))
            });
        }
        self.lifted.insert(id, lifted);
        if let ExprKind::Call(_, lowered) = &mut call.kind {
            lowered.extend(extra);
        }
        Some(call)
    }
}
