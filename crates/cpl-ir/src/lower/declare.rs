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
        let reversed = !mutable && Self::reversed_list(&init);
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
