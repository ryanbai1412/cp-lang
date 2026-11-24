//! Calls: a fn's arguments, fn values, and what moves into a call.

use super::*;

impl<'p> Emitter<'p> {
    /// The arguments of a call through a fn value: `codes` are the ones
    /// passed by value, in order, and a `mut` one is the place itself.
    pub(super) fn fn_value_args(&mut self, args: &[Arg], codes: Vec<Code>) -> Vec<String> {
        let mut codes = codes.into_iter();
        args.iter()
            .map(|arg| match arg {
                Arg::Value(v) => self.as_stored(v, codes.next().expect("one per value").text),
                Arg::Mut(place) => self.place(place),
            })
            .collect()
    }

    pub(super) fn call(&mut self, e: &Expr, id: usize, args: &[Arg]) -> Code {
        let last_use = std::mem::take(&mut self.last_use_call);
        if let Some(code) = self
            .try_fused_bitset_op(e, id, args)
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
                    if self.devirt.param_elidable(id, slot) && self.devirt.skippable_value(value) {
                        None
                    } else {
                        Some(value)
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
                }
            }
        }
        let site = self.site_store(id);
        let marked = self.mark_read_in_place(&operands, Some(id), &lent);
        let result = self.sequence(&operands, &e.ty, |this, codes| {
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
                        let code = codes.next().expect("one per value");
                        if slot < f.params.len() && f.locals[slot].ty == Ty::Unit {
                            if code.text != "(void)0" {
                                ran.push(code.text);
                            }
                            continue;
                        }
                        // A parameter taken by reference must not see the
                        // callee change what it refers to: the language
                        // passes values.
                        let by_ref = !value.ty.is_scalar()
                            && this.devirt.template_param_index(id, slot).is_none()
                            && this.facts.stores((Some(id), false, slot)) <= 2
                            && !this.sunk[id].contains(&slot);
                        if last_use && !by_ref && this.moves_in(id, slot, value, args, &code.text) {
                            texts.push(format!("std::move({})", code.text));
                            continue;
                        }
                        let shared = match crate::lend::given_root(value) {
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
            let call = format!("{home}{name}({})", texts.join(", "));
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
        let ExprKind::Local(local) = value.kind else {
            return false;
        };
        let Some(caller) = self.fn_id else {
            return false;
        };
        let cf = &self.program.fns[caller];
        let owned = local >= cf.params.len()
            || (!cf.params[local].mutable
                && (self.sunk[caller].contains(&local)
                    || self.facts.stores((Some(caller), false, local)) > 2));
        if !owned
            || value.ty.is_scalar()
            || matches!(value.ty, Ty::Fn(..))
            || self.devirt.param_known(id, slot)
            || text != self.names[local]
        {
            return false;
        }
        !args.iter().enumerate().any(|(j, arg)| {
            j != slot
                && match arg {
                    Arg::Mut(place) => walk::root(place) == (false, local),
                    Arg::Value(other) => {
                        let mut used = false;
                        walk::deep_expr(other, &mut |e| {
                            used |= matches!(e.kind, ExprKind::Local(s) if s == local);
                        });
                        used
                    }
                }
        })
    }
}
