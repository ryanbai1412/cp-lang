//! Calls: a fn's arguments, fn values, and what moves into a call.

use super::*;

impl<'p> Emitter<'p> {
    /// The arguments of a call through a fn value: `codes` are the ones
    /// passed by value, in order, and a `mut` one is the place itself.
    pub(super) fn fn_value_args(&mut self, args: &[Arg], codes: Vec<Code>) -> Vec<String> {
        let mut codes = codes.into_iter();
        args.iter()
            .map(|arg| match arg {
                Arg::Value(v) => {
                    let text = match codes.next() {
                        Some(code) => code.text,
                        None => self.expr(v).text,
                    };
                    match (&v.ty, text.as_str()) {
                        // A fn value's `()` parameter is a `cpl::Unit`: the value runs, then one is given.
                        (Ty::Unit, "(void)0") => "cpl::Unit{}".to_string(),
                        (Ty::Unit, _) => format!("((void)({text}), cpl::Unit{{}})"),
                        _ => self.as_stored(v, text),
                    }
                }
                Arg::Mut(place) => self.place(place),
            })
            .collect()
    }

    /// The view signature of a `?fn` parameter held as a `cpl::FnRef`.
    fn opt_view(&self, id: usize, slot: usize) -> Option<String> {
        let optional =
            matches!(self.program.fns[id].locals.get(slot), Some(l) if matches!(l.ty, Ty::Opt(_)));
        self.fn_view(id, slot).filter(|_| optional)
    }

    /// The `cpl::FnRef` a `?fn` argument of [`view_shape`] is: null for
    /// `None`, a pointer to what `Some` holds, through either arm of an `if`.
    fn view_arg(&mut self, sig: &str, value: &Expr) -> String {
        match &value.kind {
            ExprKind::None => format!("cpl::FnRef<{sig}>()"),
            ExprKind::Some(inner) => format!("cpl::FnRef<{sig}>({})", self.expr(inner).text),
            ExprKind::If(cond, then, otherwise) => {
                let cond = self.expr(cond).at(COND);
                let (a, b) = (self.view_arg(sig, then), self.view_arg(sig, otherwise));
                format!("({cond} ? {a} : {b})")
            }
            _ => format!("cpl::FnRef<{sig}>({})", self.expr(value).text),
        }
    }

    /// The argument for a parameter held as a `cpl::Lent` (see
    /// [`crate::lentopt`]): a pointer to the value, or to a copy when the
    /// call also changes it through a `mut` argument.
    fn lend_arg(
        &mut self,
        id: usize,
        slot: usize,
        value: &Expr,
        text: &str,
        lent: &[(bool, usize)],
    ) -> String {
        self.need("lent");
        let shared = crate::lend::given_root(value, &self.lent_roots).is_some_and(|root| {
            lent.contains(&root) || (root.0 && self.uses.effects(id).writes.contains(&root.1))
        });
        match &value.kind {
            ExprKind::None => {
                let loc = cpl_ir::ElemLoc::Local(cpl_ir::ScopeId::Fn(id), slot);
                let ty = &self.program.fns[id].locals[slot].ty;
                format!("{}{{nullptr}}", self.ty_for_loc(ty, loc))
            }
            ExprKind::Some(inner) if shared => {
                self.note_shared_arg_copy(inner);
                format!("cpl::lend({}({text}))", self.ty(&inner.ty))
            }
            ExprKind::Some(_) => format!("cpl::lend({text})"),
            _ if shared => {
                self.note_shared_arg_copy(value);
                format!("cpl::lend_opt({}({text}))", self.ty(&value.ty))
            }
            _ => format!("cpl::lend_opt({text})"),
        }
    }

    pub(super) fn call(&mut self, e: &Expr, id: usize, args: &[Arg]) -> Code {
        let last_use = std::mem::take(&mut self.last_use_call);
        let into = self.into_arg.take();
        if let Some(code) = self
            .try_fused_bitset_op(e, id, args)
            .or_else(|| self.try_lap_fusion(e, id, args))
            .or_else(|| self.try_sink_call(id, args))
        {
            return code;
        }
        let program = self.program;
        let f = &program.fns[id];
        let operands: Vec<&Expr> = args
            .iter()
            .enumerate()
            .filter_map(|(slot, arg)| match arg {
                Arg::Value(value) => {
                    if (self.devirt.param_elidable(id, slot) && self.devirt.skippable_value(value))
                        || self.messages[id].contains(&slot)
                    {
                        None
                    } else {
                        match &value.kind {
                            _ if self.opt_view(id, slot).is_some() && view_shape(value) => None,
                            ExprKind::Some(inner) if self.lent_opts[id].contains(&slot) => {
                                Some(&**inner)
                            }
                            _ => Some(value),
                        }
                    }
                }
                Arg::Mut(_) => None,
            })
            .collect();
        let lent: Vec<(bool, usize)> = args
            .iter()
            .filter_map(|arg| match arg {
                Arg::Mut(place) => Some(walk::root(place)),
                Arg::Value(_) => None,
            })
            .collect();
        let mut borrowed = Vec::new();
        for (slot, arg) in args.iter().enumerate() {
            if let Arg::Value(value) = arg {
                if let ExprKind::Closure(cid) = value.kind {
                    if crate::lend::fn_param_only_called(program, id, slot)
                        && self.borrow_closures.insert(cid)
                    {
                        borrowed.push(cid);
                    }
                } else if crate::lend::opt_fn_param_only_called(program, id, slot) {
                    for cid in crate::lend::optional_closures(value) {
                        if self.borrow_closures.insert(cid) {
                            borrowed.push(cid);
                        }
                    }
                }
            }
        }
        let layouts: Vec<_> = (args.iter().enumerate())
            .filter(|(_, arg)| matches!(arg, Arg::Value(v) if operands.iter().any(|o| std::ptr::eq(*o, v))))
            .map(|(slot, arg)| match arg {
                Arg::Value(v) if crate::scope::narrow::is_fresh_list(v) => self
                    .elem_layout(cpl_ir::ElemLoc::Local(cpl_ir::ScopeId::Fn(id), slot))
                    .cloned(),
                _ => None,
            })
            .collect();
        let site = self.site_store(id);
        let marked = self.mark_read_in_place(&operands, Some(id), &lent);
        let result = self.sequence_laid(&operands, &layouts, &e.ty, |this, codes| {
            let mut codes = codes.into_iter();
            let mut texts = Vec::with_capacity(args.len());
            // Arguments for parameters that hold no value: they are run
            // for what they do, ahead of the call, and not passed.
            let mut ran = Vec::new();
            for (slot, arg) in args.iter().enumerate() {
                match arg {
                    Arg::Mut(place) => texts.push(this.place(place)),
                    Arg::Value(value) => {
                        if this.devirt.param_elidable(id, slot)
                            && this.devirt.skippable_value(value)
                        {
                            texts.push("false".to_string());
                            continue;
                        }
                        if let Some(sig) = this.opt_view(id, slot) {
                            if view_shape(value) {
                                let text = this.view_arg(&sig, value);
                                texts.push(text);
                                continue;
                            }
                        }
                        if this.messages[id].contains(&slot) {
                            texts.push(this.message_text(value));
                            continue;
                        }
                        let code = match codes.next() {
                            Some(code) => code,
                            None => this.expr(value),
                        };
                        if let Some(sig) = this.fn_view(id, slot) {
                            texts.push(match this.opt_view(id, slot) {
                                Some(_) => format!("cpl::FnRef<{sig}>({})", code.text),
                                None => code.text,
                            });
                            continue;
                        }
                        if slot < f.params.len() && f.locals[slot].ty == Ty::Unit {
                            if code.text != "(void)0" {
                                ran.push(code.text);
                            }
                            continue;
                        }
                        if this.lent_opts[id].contains(&slot) {
                            texts.push(this.lend_arg(id, slot, value, &code.text, &lent));
                            continue;
                        }
                        // A parameter taken by reference must not see the
                        // callee change what it refers to: the language
                        // passes values.
                        let by_ref = !value.ty.is_scalar()
                            && this.devirt.template_param_index(id, slot).is_none()
                            && this.facts.stores((Some(id), false, slot)) <= 2
                            && !this.sunk[id].contains(&slot);
                        let dies = last_use || this.called_moves.contains(&(value as *const Expr));
                        if dies && !by_ref && this.moves_in(id, slot, value, args, &code.text) {
                            texts.push(format!("std::move({})", code.text));
                            continue;
                        }
                        let shared = match crate::lend::given_root(value, &this.lent_roots) {
                            Some(root) => {
                                lent.contains(&root)
                                    || (root.0 && this.uses.effects(id).writes.contains(&root.1))
                            }
                            None => false,
                        };
                        if by_ref && shared {
                            let ty = if let Some(k) = this.expr_fixed_len(value) {
                                this.fixed_array_cpp(&value.ty, k)
                            } else {
                                this.ty_with_elem_layout(&value.ty, this.expr_elem_layout(value))
                            };
                            if crate::lend::holds_heap(this.program, &value.ty) {
                                this.note_copy(
                                    value.span,
                                    "the callee writes the value it came from, so it gets its own",
                                    None,
                                );
                            }
                            texts.push(format!("{ty}({})", code.text));
                        } else {
                            texts.push(code.text);
                        }
                    }
                }
            }
            // An argument of a class type makes C++ look in that class's
            // namespace too, where `std::apply` or `std::swap` would join
            // a fn of that name. The qualified name looks only here.
            // No name of the standard library has a capital letter, so a
            // fn with one (`conv_Int_115`, a copy made for its types) can't
            // meet one.
            let name = fn_name(&f.name);
            let scalars = args
                .iter()
                .all(|arg| matches!(arg, Arg::Value(value) if value.ty.is_scalar()));
            let home = if scalars || name.bytes().any(|b| b.is_ascii_uppercase()) {
                ""
            } else {
                "::prog::"
            };
            texts.extend(into.clone());
            let mut call = format!("{home}{name}({})", texts.join(", "));
            if this.held_roots.contains_key(&id) {
                call.push_str(".take()");
            }
            ran.extend(site);
            if ran.is_empty() {
                Code::primary(call)
            } else {
                Code::primary(format!("({}, {call})", ran.join(", ")))
            }
        });
        for cid in borrowed {
            self.borrow_closures.remove(&cid);
        }
        marked.iter().for_each(|m| _ = self.read_in_place.remove(m));
        result
    }

    /// A constant message as a `const char*` (see [`crate::uses::message`]).
    pub(crate) fn message_text(&mut self, e: &Expr) -> String {
        match &e.kind {
            ExprKind::Str(bytes) => string_literal(bytes),
            ExprKind::If(c, a, b) => {
                let c = self.expr(c).text;
                format!(
                    "({c} ? {} : {})",
                    self.message_text(a),
                    self.message_text(b)
                )
            }
            _ => self.expr(e).text,
        }
    }

    /// True when `value`, an argument of the call a `return` makes, is a
    /// local the caller owns and names nowhere else in the call, passed to a
    /// parameter taken by value: moving it in skips a copy.
    pub(super) fn moves_in(
        &self,
        id: usize,
        slot: usize,
        value: &Expr,
        args: &[Arg],
        text: &str,
    ) -> bool {
        // A global moves only from the top level, which `dies_in` checks.
        let (global, local) = match value.kind {
            ExprKind::Local(local) => (false, local),
            ExprKind::Global(g) if self.fn_id.is_none() => (true, g),
            _ => return false,
        };
        // The program's top level has no parameters: every local is its own.
        let owned = global
            || self.fn_id.is_none_or(|caller| {
                let cf = &self.program.fns[caller];
                local >= cf.params.len()
                    || (!cf.params[local].mutable
                        && (self.sunk[caller].contains(&local)
                            || self.facts.stores((Some(caller), false, local)) > 2))
            });
        if !owned
            || value.ty.is_scalar()
            || matches!(value.ty, Ty::Fn(..))
            || self.devirt.param_known(id, slot)
            || (!global && text != self.names[local])
            || (global && text != crate::names::global_ident(self.program, local))
        {
            return false;
        }
        !args.iter().enumerate().any(|(j, arg)| {
            j != slot
                && match arg {
                    Arg::Mut(place) => walk::root(place) == (global, local),
                    Arg::Value(other) => {
                        let mut used = false;
                        crate::uses::places::each_part(other, &mut |e| {
                            used |= match e.kind {
                                ExprKind::Local(s) => !global && s == local,
                                ExprKind::Global(g) => global && g == local,
                                _ => false,
                            };
                        });
                        used
                    }
                }
        })
    }
}

/// A `?fn` argument built in place as a `cpl::FnRef`: `None`, `Some` of a
/// closure or name, or an `if` on a name between two of them.
fn view_shape(value: &Expr) -> bool {
    match &value.kind {
        ExprKind::None => true,
        ExprKind::Some(inner) => matches!(inner.kind, ExprKind::Closure(_) | ExprKind::Local(_)),
        ExprKind::If(cond, then, otherwise) => {
            matches!(cond.kind, ExprKind::Local(_) | ExprKind::Bool(_))
                && view_shape(then)
                && view_shape(otherwise)
        }
        _ => false,
    }
}
