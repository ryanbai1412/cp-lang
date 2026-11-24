//! Loops: for statements, their bodies, and loops that push.

use super::*;

impl<'p> Emitter<'p> {
    /// The body of a loop, its `}` and the labels a `break` or
    /// `continue` from an inner loop jumps to. `first` are statements
    /// that open the body.
    pub(crate) fn loop_body(
        &mut self,
        id: LoopId,
        first: &[String],
        body: &Block,
        out: &mut String,
    ) {
        self.indent += 1;
        for line in first {
            self.line(out, line);
        }
        if let Some(pushed) = self.bodies.last_mut() {
            pushed.depth += 1;
        }
        if self.continue_targets.contains(&id) {
            // In a block of its own, so the jump passes no declaration.
            self.line(out, "{");
            self.indent += 1;
            self.block(body, out);
            self.indent -= 1;
            self.line(out, "}");
            self.line(out, &format!("cont_{id}:;"));
        } else {
            self.block(body, out);
        }
        if let Some(pushed) = self.bodies.last_mut() {
            pushed.depth -= 1;
        }
        self.indent -= 1;
        self.line(out, "}");
        if self.break_targets.contains(&id) {
            self.line(out, &format!("brk_{id}:;"));
        }
    }

    /// `for x in f(..)` over a `yields` fn as `cpl_each_f(.., body)`: the
    /// body runs on each value as `f` makes it and no list is built.
    /// False when the loop has to walk the list instead.
    pub(super) fn pushed_loop(
        &mut self,
        id: LoopId,
        decl: &str,
        first: &[String],
        seq: &Expr,
        body: &Block,
        out: &mut String,
    ) -> bool {
        let ExprKind::Call(callee, _) = &seq.kind else {
            return false;
        };
        let returns = cpl_ir::pushed::returns(body);
        // A closure's `return` is not the enclosing fn's, so only a body that
        // never returns is fed from inside one.
        if self.pushing.is_some() || (self.closure_id.is_some() && returns) {
            return false;
        }
        let feeds = cpl_ir::pushed::Feeds {
            summaries: std::mem::take(&mut self.summaries),
            pushers: std::mem::take(&mut self.pushers),
        };
        let fed = feeds.feeds(id, seq, body);
        self.summaries = feeds.summaries;
        self.pushers = feeds.pushers;
        if fed.is_none() {
            return false;
        }
        let call = self.expr(seq).text;
        let name = fn_name(&self.program.fns[*callee].name);
        let Some(call) = (call.strip_prefix("::prog::").unwrap_or(&call))
            .strip_prefix(&format!("{name}("))
            .and_then(|rest| rest.strip_suffix(')'))
        else {
            return false;
        };
        let sep = if call.is_empty() { "" } else { ", " };
        let ret = match self.fn_id {
            Some(f) if self.program.fns[f].ret != Ty::Unit => {
                Some(self.ty(&self.program.fns[f].ret))
            }
            _ => None,
        };
        let outermost = self.bodies.is_empty();
        self.line(out, "{");
        self.indent += 1;
        let result = match (outermost, returns, ret) {
            (false, ..) => self.bodies[0].result.clone(),
            (true, true, Some(ret)) => {
                let held = self.temp();
                self.line(out, &format!("std::optional<{ret}> {held};"));
                Some(held)
            }
            _ => None,
        };
        let open = if returns { "if (" } else { "" };
        self.line(
            out,
            &format!("{open}cpl_each_{name}({call}{sep}[&]({decl}) -> int {{"),
        );
        self.indent += 1;
        for line in first {
            self.line(out, line);
        }
        self.bodies.push(Pushed {
            id,
            depth: 0,
            result: result.clone(),
        });
        self.block(body, out);
        self.bodies.pop();
        self.line(out, "return 0;");
        self.indent -= 1;
        let close = match (returns, outermost, &result) {
            (false, ..) => "});".to_string(),
            (true, false, _) => "}) == 2) return 2;".to_string(),
            (true, true, Some(held)) => format!("}}) == 2) return std::move(*{held});"),
            (true, true, None) if self.in_setup => "}) == 2) return false;".to_string(),
            (true, true, None) => "}) == 2) return;".to_string(),
        };
        self.line(out, &close);
        self.indent -= 1;
        self.line(out, "}");
        true
    }

    pub(crate) fn for_stmt(
        &mut self,
        id: LoopId,
        slot: usize,
        iter: &Iter,
        body: &Block,
        out: &mut String,
    ) {
        let ty = self.locals()[slot].ty.clone();
        let cpp = self.ty_with_elem_layout(
            &ty,
            self.elem_layout(cpl_ir::ElemLoc::Local(self.current_scope(), slot)),
        );
        let name = self.names[slot].clone();
        let stable = self.facts.stable(self.facts_local(slot));
        let hoisted = self.hoisted.contains(&slot);
        let enum_range = matches!(ty, Ty::Enum(_)) && matches!(iter, Iter::Range { .. });
        // The loop's own counter, when the body may change the variable.
        let var = if hoisted || !stable || enum_range {
            self.temp()
        } else {
            name.clone()
        };
        let mut first = Vec::new();
        if enum_range {
            if hoisted {
                first.push(format!("{name} = static_cast<{cpp}>({var});"));
            } else {
                first.push(format!("{cpp} {name} = static_cast<{cpp}>({var});"));
            }
        } else if hoisted {
            first.push(format!("{name} = {var};"));
        } else if !stable {
            first.push(format!("{cpp} {name} = {var};"));
        }
        let cpp = if enum_range { "ll".to_string() } else { cpp };
        match iter {
            Iter::Walk { .. } => unreachable!("Iter::Walk is expanded before codegen"),
            Iter::Range {
                lo,
                hi,
                step,
                rev,
                closed,
            } => {
                let closed = *closed;
                let fit = |text: String, narrow: bool| {
                    if cpp == "int" && !narrow {
                        format!("(int)({text})")
                    } else {
                        text
                    }
                };
                let lo_code = self.expr(lo);
                let hi_code = self.expr(hi);
                let mut extra = String::new();
                let step_text = match step {
                    None => None,
                    Some(step) if self.is_fixed(step) => Some(self.expr(step).at(ADD + 1)),
                    Some(step) => {
                        let name = self.unique(&format!("{var}_step"));
                        let text = self.expr(step).text;
                        extra.push_str(&format!(", {name} = {}", fit(text, false)));
                        Some(name)
                    }
                };
                match (rev, &step_text) {
                    (false, _) => {
                        let bound = if self.is_fixed(hi) {
                            hi_code.at(REL + 1)
                        } else {
                            let end = self.unique(&format!("{var}_end"));
                            extra.push_str(&format!(
                                ", {end} = {}",
                                fit(hi_code.text.clone(), hi_code.narrow)
                            ));
                            end
                        };
                        let start = fit(lo_code.text.clone(), lo_code.narrow);
                        let head = match &step_text {
                            // By steps: the number of rounds is counted
                            // first (a step of 0 or less stops the program,
                            // as in `cpl run`), and the last step past the
                            // end wraps instead of overflowing.
                            Some(step) => {
                                self.need("steps");
                                let left = self.unique(&format!("{var}_left"));
                                let wide = if cpp == "int" {
                                    "unsigned"
                                } else {
                                    "unsigned long long"
                                };
                                let to = if closed { ", true" } else { "" };
                                format!(
                                    "for ({cpp} {var} = {start}{extra}, {left} = ({cpp})cpl::steps({var}, {bound}, {step}{to}); \
                                     {left} > 0; {left}--, {var} = ({cpp})(({wide}){var} + ({wide})({step}))) {{"
                                )
                            }
                            None if !closed => format!(
                                "for ({cpp} {var} = {start}{extra}; {var} < {bound}; {var}++) {{"
                            ),
                            // Up to a last value that is not the largest
                            // of its type: `<=` ends the loop.
                            None if cpp == "int"
                                || hi_code.narrow
                                || self.facts.below_max(self.facts_home(), hi) =>
                            {
                                format!(
                                    "for ({cpp} {var} = {start}{extra}; {var} <= {bound}; {var}++) {{"
                                )
                            }
                            // The last value may be `Int.MAX`, which has
                            // no next one: the loop stops on reaching it.
                            None => {
                                let go = self.unique(&format!("{var}_go"));
                                format!(
                                    "for ({cpp} {var} = {start}{extra}, {go} = {var} <= {bound}; {go}; \
                                     {go} = {var} < {bound} && (++{var}, true)) {{"
                                )
                            }
                        };
                        self.line(out, &head);
                        self.loop_body(id, &first, body, out);
                    }
                    (true, None) => {
                        let bound = if self.is_fixed(lo) {
                            lo_code.at(REL + 1)
                        } else {
                            let end = self.unique(&format!("{var}_end"));
                            extra.push_str(&format!(
                                ", {end} = {}",
                                fit(lo_code.text.clone(), lo_code.narrow)
                            ));
                            end
                        };
                        let start = if closed {
                            fit(hi_code.text.clone(), hi_code.narrow)
                        } else {
                            fit(format!("{} - 1", hi_code.at(ADD)), false)
                        };
                        // A first value that may be `Int.MIN` has none
                        // before it: the loop stops on reaching it.
                        let head = if cpp == "int" || lo_code.narrow {
                            format!(
                                "for ({cpp} {var} = {start}{extra}; {var} >= {bound}; {var}--) {{"
                            )
                        } else {
                            let go = self.unique(&format!("{var}_go"));
                            format!(
                                "for ({cpp} {var} = {start}{extra}, {go} = {var} >= {bound}; {go}; \
                                 {go} = {var} > {bound} && (--{var}, true)) {{"
                            )
                        };
                        self.line(out, &head);
                        self.loop_body(id, &first, body, out);
                    }
                    (true, Some(step)) => {
                        // Backward by steps: count them, then walk down.
                        let (a, b, k) = (self.temp(), self.temp(), self.temp());
                        self.line(out, "{");
                        self.indent += 1;
                        self.line(
                            out,
                            &format!(
                                "ll {a} = {}, {b} = {}{};",
                                lo_code.text,
                                hi_code.text,
                                extra.replace(", ", "; ll ").replacen("; ll ", ", ", 1)
                            ),
                        );
                        self.need("steps");
                        self.line(out, &format!("for (ll {k} = cpl::steps({a}, {b}, {step}{}) - 1; {k} >= 0; {k}--) {{", if closed { ", true" } else { "" }));
                        let mut opening = vec![format!(
                            "{cpp} {var} = {};",
                            fit(format!("{a} + {k} * {step}"), false)
                        )];
                        opening.extend(first);
                        self.loop_body(id, &opening, body, out);
                        self.indent -= 1;
                        self.line(out, "}");
                    }
                }
            }
            Iter::RangeVal(r_expr, rev) => {
                let r_code = self.expr(r_expr);
                let held = self.temp();
                self.line(out, "{");
                self.indent += 1;
                self.line(out, &format!("auto {held} = {};", r_code.text));
                if *rev {
                    self.line(
                        out,
                        &format!("for ({cpp} {var} = {held}.hi; {var}-- > {held}.lo; ) {{"),
                    );
                    self.loop_body(id, &first, body, out);
                } else {
                    let plain = var == name && first.is_empty();
                    let guard = plain.then(|| crate::scope::start_guard(slot, body));
                    let (lo, body) = match guard.flatten() {
                        Some((e, then)) => {
                            let e = self.expr(e).text;
                            let lo =
                                format!("std::max<ll>({held}.lo, std::min<ll>({e}, {held}.hi))");
                            (format!("({cpp})({lo})"), then)
                        }
                        None => (format!("{held}.lo"), body),
                    };
                    let head = format!("for ({cpp} {var} = {lo}; {var} < {held}.hi; {var}++) {{");
                    self.line(out, &head);
                    self.loop_body(id, &first, body, out);
                }
                self.indent -= 1;
                self.line(out, "}");
            }
            Iter::TreeRange {
                container,
                lo,
                hi,
                rev,
            } => {
                let effects = Touch::of_block(body);
                let untouched = match walk::path_root(container) {
                    Some((false, local)) => {
                        !effects.writes.contains(&(false, local))
                            || fields_untouched(container, body)
                    }
                    Some((true, global)) => {
                        !effects.writes.contains(&(true, global))
                            && !effects
                                .calls
                                .iter()
                                .any(|f| self.uses.effects(*f).writes.contains(&global))
                    }
                    None => false,
                };
                let is_map = matches!(container.ty, Ty::Map(..));
                let c_expr = self.expr(container);
                let c_name = self.temp();
                self.line(out, "{");
                self.indent += 1;
                if untouched {
                    self.line(out, &format!("const auto& {c_name} = {};", c_expr.text));
                } else {
                    self.line(out, &format!("auto {c_name} = {};", c_expr.text));
                }
                let lo_var = lo.as_ref().map(|(e, inc)| {
                    let v = self.temp();
                    let code = self.expr(e);
                    self.line(out, &format!("auto {v} = {};", code.text));
                    (v, *inc)
                });
                let hi_var = hi.as_ref().map(|(e, inc)| {
                    let v = self.temp();
                    let code = self.expr(e);
                    self.line(out, &format!("auto {v} = {};", code.text));
                    (v, *inc)
                });
                let it = self.temp();
                let key_expr = if is_map {
                    format!("{it}->first")
                } else {
                    format!("*{it}")
                };
                let bind_line = if is_map {
                    format!("{cpp} {var} = {cpp}{{{it}->first, {it}->second}};")
                } else {
                    format!("{cpp} {var} = *{it};")
                };
                let mut opening = vec![bind_line];
                opening.extend(first);
                if !rev {
                    let start = match &lo_var {
                        Some((v, true)) => format!("{c_name}.lower_bound({v})"),
                        Some((v, false)) => format!("{c_name}.upper_bound({v})"),
                        None => format!("{c_name}.begin()"),
                    };
                    let cond = match &hi_var {
                        Some((v, true)) => format!("{it} != {c_name}.end() && !({v} < {key_expr})"),
                        Some((v, false)) => format!("{it} != {c_name}.end() && ({key_expr} < {v})"),
                        None => format!("{it} != {c_name}.end()"),
                    };
                    self.line(
                        out,
                        &format!("for (auto {it} = {start}; {cond}; ++{it}) {{"),
                    );
                    self.loop_body(id, &opening, body, out);
                } else {
                    let start = match &hi_var {
                        Some((v, true)) => format!("{c_name}.upper_bound({v})"),
                        Some((v, false)) => format!("{c_name}.lower_bound({v})"),
                        None => format!("{c_name}.end()"),
                    };
                    self.line(
                        out,
                        &format!("for (auto {it} = {start}; {it} != {c_name}.begin(); ) {{"),
                    );
                    let mut rev_opening = vec![format!("--{it};")];
                    if let Some((v, inc)) = &lo_var {
                        let stop = if *inc {
                            format!("if ({key_expr} < {v}) break;")
                        } else {
                            format!("if (!({v} < {key_expr})) break;")
                        };
                        rev_opening.push(stop);
                    }
                    rev_opening.extend(opening);
                    self.loop_body(id, &rev_opening, body, out);
                }
                self.indent -= 1;
                self.line(out, "}");
            }
            Iter::Seq(seq, rev) => {
                let inlined = self.inline_list_call(seq);
                let seq = inlined.as_ref().unwrap_or(seq);
                let effects = Touch::of_block(body);
                // The loop walks the value as it was when the loop began,
                // so a container the body changes is walked as a copy.
                let untouched_by_body =
                    |seq: &Expr| match walk::path_root(crate::uses::in_place(seq)) {
                        Some((false, local)) => {
                            !effects.writes.contains(&(false, local))
                                || fields_untouched(crate::uses::in_place(seq), body)
                        }
                        Some((true, global)) => {
                            !effects.writes.contains(&(true, global))
                                && !effects
                                    .calls
                                    .iter()
                                    .any(|f| self.uses.effects(*f).writes.contains(&global))
                        }
                        None => false,
                    };
                let untouched = untouched_by_body(seq);
                let lent = self.lent_coalesce(seq).filter(|(from, _)| match from {
                    crate::lend::Lender::Get(m, _) => untouched_by_body(m),
                    crate::lend::Lender::Named(_) => true,
                });
                let is_map = matches!(seq.ty, Ty::Map(..) | Ty::HashMap(..));
                let is_grid = matches!(&seq.ty, Ty::Array { dims, .. } if dims.len() > 1);
                if !rev && self.sink_loop(id, seq, &var, &cpp, &first, body, out) {
                    return;
                }
                if !rev && self.try_fuse_producer(id, slot, &var, &cpp, &first, seq, body, out) {
                    return;
                }
                if !rev && matches!(seq.kind, ExprKind::Call(..)) {
                    let (decl, pushed_first): (String, &[String]) =
                        if matches!(ty, Ty::Struct(_)) && !hoisted {
                            (format!("auto&& {name}"), &[])
                        } else if ty.is_scalar() || var != name {
                            (format!("{cpp} {var}"), &first)
                        } else {
                            (format!("const {cpp}& {var}"), &first)
                        };
                    if self.pushed_loop(id, &decl, pushed_first, seq, body, out) {
                        return;
                    }
                }
                let lit_arr = match (&seq.kind, &seq.ty) {
                    (ExprKind::Tuple(items), Ty::List(elem)) if items.len() <= 64 => {
                        if items.is_empty() {
                            return;
                        }
                        Some((items, elem))
                    }
                    _ => None,
                };
                let opened = lit_arr.is_some() || !(untouched && !rev && !is_map);
                // A short literal walked forward is unrolled: each round then reads constants.
                let unroll = match lit_arr {
                    Some((items, _)) if items.len() <= 16 && !rev => {
                        format!("_Pragma(\"GCC unroll {}\") ", items.len())
                    }
                    _ => String::new(),
                };
                let holder = if let Some((items, elem)) = lit_arr {
                    let elem_cpp = if matches!(elem.as_ref(), Ty::Bool) {
                        "bool".to_string()
                    } else {
                        self.ty(elem)
                    };
                    let vals: Vec<String> = items.iter().map(|it| self.expr(it).text).collect();
                    let held = self.temp();
                    self.line(out, "{");
                    self.indent += 1;
                    self.line(
                        out,
                        &format!(
                            "std::array<{elem_cpp}, {}> {held} = {{{}}};",
                            items.len(),
                            vals.join(", ")
                        ),
                    );
                    held
                } else if let Some((from, default)) = lent {
                    self.loop_over_lent(from, default, out)
                } else if opened {
                    self.line(out, "{");
                    self.indent += 1;
                    let seq_text = self
                        .pinned_view(seq, Hold::Block, out)
                        .unwrap_or_else(|| self.expr(seq).text);
                    let held = self.temp();
                    let holder_ty = if let Some(k) = self.expr_fixed_len(seq) {
                        self.fixed_array_cpp(&seq.ty, k)
                    } else {
                        self.ty_with_elem_layout(&seq.ty, self.expr_elem_layout(seq))
                    };
                    if untouched {
                        self.line(out, &format!("const {holder_ty}& {held} = {seq_text};"));
                    } else {
                        self.line(out, &format!("{holder_ty} {held} = {seq_text};"));
                    }
                    held
                } else {
                    let text = self.expr(seq).at(PRIMARY);
                    match text.strip_suffix(".vec()") {
                        Some(row) if self.is_row(seq) => row.to_string(),
                        _ => text,
                    }
                };
                let items = if is_grid {
                    format!("{holder}.d")
                } else {
                    holder
                };
                let narrow_tuple_iter = matches!(
                    self.expr_elem_layout(seq),
                    Some(cpl_ir::NarrowLayout::Tuple(_))
                );
                let decl = if ty.is_scalar() || var != name || narrow_tuple_iter {
                    format!("{cpp} {var}")
                } else {
                    format!("const {cpp}& {var}")
                };
                if is_map {
                    let it = self.temp();
                    let mut opening = Vec::new();
                    let entry = |e: &str| crate::lend::entry_binding(&ty, &cpp, &var, e);
                    if *rev {
                        self.line(out, &format!("for (auto {it} = {items}.rbegin(); {it} != {items}.rend(); ++{it}) {{"));
                        opening.push(entry(&format!("{it}->")));
                    } else {
                        self.line(out, &format!("for (const auto& {it} : {items}) {{"));
                        opening.push(entry(&format!("{it}.")));
                    }
                    opening.extend(first);
                    self.loop_body(id, &opening, body, out);
                } else if *rev {
                    let it = self.temp();
                    self.line(
                        out,
                        &format!(
                            "for (auto {it} = {items}.rbegin(); {it} != {items}.rend(); ++{it}) {{"
                        ),
                    );
                    let mut opening = vec![format!("{decl} = *{it};")];
                    opening.extend(first);
                    self.loop_body(id, &opening, body, out);
                } else {
                    self.line(out, &format!("{unroll}for ({decl} : {items}) {{"));
                    self.loop_body(id, &first, body, out);
                }
                if opened {
                    self.indent -= 1;
                    self.line(out, "}");
                }
            }
        }
    }
}
