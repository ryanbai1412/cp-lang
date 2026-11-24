//! Std fns whose calls a built-in arm answers, run behind std's own leading
//! asserts, so a bad call traps with std's report on both engines: std's
//! message, the user's line and the values.

use super::*;

/// The std impl methods an arm answers whose bodies never lower, as
/// (struct, trait, method): a built-in view has no `lo`/`hi`/`src`, so the
/// arm rebuilds the assert on the view. Counted debt (`std_shadowed`), gone
/// once `Slice` is a std struct behind `Index<Range>`.
pub(super) const ARM_GUARDED: [(&str, &str, &str); 2] =
    [("Slice", "Positional", "nth"), ("Slice", "Indexed", "at")];

impl<'a> Lowerer<'a> {
    /// The first statement of std's `ARM_GUARDED[which]`, an assert,
    /// checking `cond`, with the values its condition shows taken from
    /// `values` by their text in std.
    fn arm_guard(&mut self, which: usize, cond: Expr, values: &[(&str, Expr)]) -> Option<Stmt> {
        let (target, trait_name, method) = ARM_GUARDED[which];
        let mut found = None;
        for imp in &self.impls {
            let ast::TypeKind::Named { name, .. } = &imp.target.kind else {
                continue;
            };
            if name.text != target || imp.trait_name.as_deref() != Some(trait_name) {
                continue;
            }
            for s in &imp.body.stmts {
                if let ast::StmtKind::Fn(f) = &s.kind {
                    if f.name.text == method && found.is_none() {
                        found = Some((imp.mod_id, imp.offset, f));
                    }
                }
            }
        }
        let (mod_id, offset, decl) = found?;
        let ast::FnBody::Block(body) = &decl.body else {
            return None;
        };
        let ast::StmtKind::Assert {
            cond: std_cond,
            message,
        } = &body.stmts.first()?.kind
        else {
            return None;
        };
        let span = cond.span;
        let kept = self.kept_assert();
        let saved = (
            std::mem::replace(&mut self.cur_mod, mod_id),
            std::mem::replace(&mut self.offset, offset),
        );
        let (text, line, note) = self.assert_place(std_cond.span);
        let message = message.as_ref().map(|m| self.expr_as(m, &Ty::Str));
        let shown = self.shown_values(std_cond, Some(values));
        (self.cur_mod, self.offset) = saved;
        let message = self.with_values(message, shown);
        Some(Stmt::Assert {
            cond,
            message,
            text,
            line,
            note,
            span,
            kept,
        })
    }

    /// Lowers the leading asserts of std fn `gid` with each parameter bound
    /// to the value given for it: an arm that answers the fn still checks
    /// what std's body checks first.
    pub(super) fn std_fn_asserts(
        &mut self,
        gid: GenFnId,
        binds: Vec<(&str, Expr)>,
        span: Span,
        out: &mut Vec<Stmt>,
    ) {
        let tpl = &self.gen_fns[gid];
        let (mod_id, offset, decl) = (tpl.mod_id, tpl.offset, tpl.decl);
        let ast::FnBody::Block(body) = &decl.body else {
            return;
        };
        if !matches!(
            body.stmts.first().map(|s| &s.kind),
            Some(ast::StmtKind::Assert { .. })
        ) {
            return;
        }
        let mut scope = HashMap::new();
        for (name, value) in binds {
            let slot = match value.kind {
                ExprKind::Local(slot) => slot,
                _ => {
                    let slot = self.temp(value.ty.clone());
                    out.push_let(slot, value);
                    slot
                }
            };
            scope.insert(name.to_string(), slot);
        }
        let _ = span;
        let saved = (
            std::mem::replace(&mut self.cur_mod, mod_id),
            std::mem::replace(&mut self.offset, offset),
        );
        self.cx.scopes.push(scope);
        for s in &body.stmts {
            if !matches!(s.kind, ast::StmtKind::Assert { .. }) {
                break;
            }
            self.stmt(s, out);
        }
        self.cx.scopes.pop();
        (self.cur_mod, self.offset) = saved;
    }

    /// The std fn `name` an arm answers, wherever the caller's module stands.
    pub(super) fn std_gen_fn(&self, name: &str) -> Option<GenFnId> {
        match self.std_sym(name)? {
            Sym::GenFn(gid) => Some(gid),
            _ => None,
        }
    }

    /// Whether an assert here is kept in every build (R09.4.1).
    fn kept_assert(&mut self) -> bool {
        self.context("checks", "asserts")
            .is_some_and(|v| matches!(&v.kind, ast::ExprKind::Name(n) if n == "on"))
    }

    fn held(&mut self, e: Expr, span: Span) -> Expr {
        if matches!(e.kind, ExprKind::Local(_) | ExprKind::Int(_)) {
            return e;
        }
        let slot = self.temp(e.ty.clone());
        self.cx.pending.push_let(slot, e);
        self.local(slot, span)
    }

    /// `view.nth(k)`, or `view.at(k)` when `at`, on a built-in view, behind
    /// std's `Slice` assert: then read unchecked where it proves `k` in
    /// range. A reversed view is std's `Rev`, whose `nth` has no assert.
    pub(super) fn guarded_read(
        &mut self,
        view: Expr,
        k: Expr,
        elem: Ty,
        at: bool,
        span: Span,
    ) -> Expr {
        let reversed = match &view.kind {
            ExprKind::Local(s) => self.cx.view_rev.contains(s),
            ExprKind::Builtin(Builtin::Reversed, _) => true,
            _ => false,
        };
        // A std fn's own reads stay std's: only user code reaches std's assert.
        if !at && (reversed || self.cur_mod != 0) {
            return self.mk_index(view, vec![k], elem, span);
        }
        if at && self.cur_mod != 0 && !reversed {
            let view = self.hold_view(view, span);
            return self.key_index(view, k, elem, span);
        }
        let view = self.hold_view(view, span);
        if reversed {
            return self.mk_index(view, vec![k], elem, span);
        }
        let k = self.held(k, span);
        let lo = self.key_lo(&view, span);
        let lo = if at { self.held(lo, span) } else { lo };
        let len = self.mk_builtin(Builtin::Len, [view.clone()], Ty::Int, span);
        let hi = self.mk_binary(BinOp::Add, lo.clone(), len.clone(), Ty::Int, span);
        let (low, top, pos) = if at {
            let pos = self.mk_binary(BinOp::Sub, k.clone(), lo.clone(), Ty::Int, span);
            (lo.clone(), hi.clone(), pos)
        } else {
            (self.int_lit(0, span), len, k.clone())
        };
        let low = self.mk_cmp(CmpOp::Le, low, k.clone(), Ty::Bool, span);
        let high = self.mk_cmp(CmpOp::Lt, k.clone(), top, Ty::Bool, span);
        let cond = self.mk_binary(BinOp::And, low, high, Ty::Bool, span);
        let values = [
            ("k", k.clone()),
            ("self.src.keys.ord(i)", k.clone()),
            ("self.hi", hi),
            ("self.lo", lo),
        ];
        match self.arm_guard(usize::from(at), cond, &values) {
            Some(guard) => {
                self.cx.pending.push(guard);
                let pos = self.mk(ExprKind::Cast(Cast::Raw, Box::new(pos)), Ty::Int, span);
                self.mk_index(view, vec![pos], elem, span)
            }
            None if at => self.key_index(view, k, elem, span),
            None => self.mk_index(view, vec![k], elem, span),
        }
    }
}
