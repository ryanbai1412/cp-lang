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
        // Inside a `cpl_each_f` form a body that yields on feeds the same
        // callback; one that may stop early is not fed there.
        let nested = self.pushing.is_some();
        let escapes = cpl_ir::pushed::escapes(body);
        // A closure's `return` is not the enclosing fn's, so only a body that
        // never returns is fed from inside one.
        if (nested && escapes) || (self.closure_id.is_some() && returns) {
            return false;
        }
        let feeds = cpl_ir::pushed::Feeds {
            summaries: std::mem::take(&mut self.summaries),
            pushers: std::mem::take(&mut self.pushers),
            eager: std::mem::take(&mut self.eager),
            scope: std::mem::take(&mut self.feed_scope),
        };
        let fed = feeds.feeds(id, seq, body);
        self.summaries = feeds.summaries;
        self.pushers = feeds.pushers;
        self.eager = feeds.eager;
        self.feed_scope = feeds.scope;
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
        // A body that never stops says so in its type, so the form's
        // `if (int cpl_r = cpl_f(..))` folds away even where it isn't inlined.
        // Inside a pusher, a yield forwards the outer callback's stop.
        let go = if self.pushing.is_some() || cpl_ir::pushed::stops(body) {
            "int"
        } else {
            "std::integral_constant<int, 0>"
        };
        if nested {
            // A pusher calling itself, through others too, hands on a callback
            // of one type: a fresh lambda type per level never ends in C++.
            let (wrap, unwrap) =
                if self.devirt.recursive_fns.contains(callee) && !decl.contains("auto") {
                    self.need("fn_ref");
                    ("cpl::fn_ref(", ")")
                } else {
                    ("", "")
                };
            // What the callback stops with stops this form too.
            self.line(
                out,
                &format!("if (int cpl_r = cpl_each_{name}({call}{sep}{wrap}[&]({decl}) -> {go} {{"),
            );
            self.indent += 1;
            for line in first {
                self.line(out, line);
            }
            self.block(body, out);
            self.line(out, "return {};");
            self.indent -= 1;
            self.line(out, &format!("}}{unwrap})) return cpl_r;"));
            return true;
        }
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
            &format!("{open}cpl_each_{name}({call}{sep}[&]({decl}) -> {go} {{"),
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
        self.line(out, "return {};");
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

    /// `for l in lo..hi: if let w = row[l]: acc[l] max= Some(base + w)` over two `?Int` rows,
    /// the max-plus product's inner loop: on an AVX2 judge, one call to the runtime's
    /// branch-free pass, which g++ vectorises. `None` when the loop is any other.
    fn keep_max_plus(&mut self, slot: usize, iter: &Iter, body: &Block) -> Option<String> {
        let Iter::Range {
            lo,
            hi,
            step: None,
            rev: false,
            closed: false,
        } = iter
        else {
            return None;
        };
        // Without AVX2 and O3 g++ won't vectorise it, and the branchy loop is faster.
        if !self.target.has("avx2") || self.hoisted.contains(&slot) {
            return None;
        }
        let [Stmt::Let(held, Some(read)), Stmt::If(test, then, other)] = body.stmts.as_slice()
        else {
            return None;
        };
        let local = |e: &Expr, want: usize| matches!(e.kind, ExprKind::Local(s) if s == want);
        let ExprKind::Index(row, at) = &read.kind else {
            return None;
        };
        let (ExprKind::Local(row_slot), [i]) = (&row.kind, at.as_slice()) else {
            return None;
        };
        let ExprKind::IsSome(tested) = &test.kind else {
            return None;
        };
        let [Stmt::Let(w, Some(unwrap)), Stmt::Assign(place, AssignOp::Max, value)] =
            then.stmts.as_slice()
        else {
            return None;
        };
        let ExprKind::Unwrap(opened, _) = &unwrap.kind else {
            return None;
        };
        let PlaceKind::Index(acc, at) = &place.kind else {
            return None;
        };
        let (PlaceKind::Local(acc_slot), [j]) = (&acc.kind, at.as_slice()) else {
            return None;
        };
        let ExprKind::Some(sum) = &value.kind else {
            return None;
        };
        let ExprKind::Binary(BinOp::Add, a, b) = &sum.kind else {
            return None;
        };
        let base = match (local(a, *w), local(b, *w)) {
            (true, false) => b,
            (false, true) => a,
            _ => return None,
        };
        let ExprKind::Local(base_slot) = base.kind else {
            return None;
        };
        let shaped = other.stmts.is_empty()
            && local(i, slot)
            && local(j, slot)
            && local(tested, *held)
            && local(opened, *held)
            && acc_slot != row_slot
            && base.ty == Ty::Int
            && ![slot, *held, *w, *acc_slot, *row_slot].contains(&base_slot);
        if !shaped {
            return None;
        }
        for s in [*row_slot, *acc_slot] {
            let ty = self.locals()[s].ty.clone();
            let layout = self.elem_layout(cpl_ir::ElemLoc::Local(self.current_scope(), s));
            if self.ty_with_elem_layout(&ty, layout) != "std::vector<cpl::OptInt>" {
                return None;
            }
        }
        self.need("keep_max_plus");
        let acc = self.place(acc);
        let (row, base) = (self.expr(row).text, self.expr(base).text);
        let (lo, hi) = (self.expr(lo).text, self.expr(hi).text);
        Some(format!(
            "cpl::keep_max_plus({acc}.data(), {row}.data(), {base}, {lo}, {hi});"
        ))
    }

    pub(crate) fn for_stmt(
        &mut self,
        id: LoopId,
        slot: usize,
        iter: &Iter,
        body: &Block,
        out: &mut String,
    ) {
        if let Some(call) = self.keep_max_plus(slot, iter, body) {
            self.line(out, &call);
            return;
        }
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
                            None if !closed => {
                                // A few rounds known before the program runs (a fixed domain's
                                // positions) are unrolled, as a hand-written fixed array would be.
                                let known = |e: &Expr| match &e.kind {
                                    ExprKind::Int(n) => Some(*n),
                                    ExprKind::DomainLen(d) => self.facts.constexpr_domain_len(*d),
                                    _ => None,
                                };
                                let unroll = match (known(lo), known(hi)) {
                                    (Some(lo), Some(hi)) if (1..=16).contains(&(hi - lo)) => {
                                        format!("_Pragma(\"GCC unroll {}\") ", hi - lo)
                                    }
                                    _ => String::new(),
                                };
                                format!(
                                    "{unroll}for ({cpp} {var} = {start}{extra}; {var} < {bound}; {var}++) {{"
                                )
                            }
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
                            // `!=` rather than `<` lets the C++ compiler count
                            // the rounds, as it does for a plain loop.
                            None => {
                                let go = self.unique(&format!("{var}_go"));
                                format!(
                                    "for ({cpp} {var} = {start}{extra}, {go} = {var} <= {bound}; {go}; \
                                     {go} = {var} != {bound} && (++{var}, true)) {{"
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
                                 {go} = {var} != {bound} && (--{var}, true)) {{"
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
                let untouched_by_body = |seq: &Expr| match held_root(crate::uses::in_place(seq)) {
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
                // A fn that returns `cpl::Held<T>` may lend a part of its argument: the walk
                // reads it in place while the body leaves that argument be.
                let held_from = match &seq.kind {
                    ExprKind::Call(f, args) => match self.held_roots.get(f).map(|&r| &args[r]) {
                        Some(cpl_ir::Arg::Value(arg)) => untouched_by_body(arg),
                        _ => false,
                    },
                    _ => false,
                };
                let lent = self.lent_coalesce(seq).filter(|(from, _)| match from {
                    crate::lend::Lender::Get(m, _) => untouched_by_body(m),
                    crate::lend::Lender::Named(_) => true,
                });
                let is_map = matches!(seq.ty, Ty::Map(..) | Ty::HashMap(..));
                let is_grid = matches!(&seq.ty, Ty::Array { dims, .. } if dims.len() > 1);
                if !rev && self.read_streamed(id, seq, &format!("{cpp} {var}"), &first, body, out) {
                    return;
                }
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
                    let mut read = !pushed_first.is_empty();
                    crate::walk::deep_exprs(body, &mut |e| {
                        read |= matches!(e.kind, ExprKind::Local(s) if s == slot)
                    });
                    let decl = if read {
                        decl
                    } else {
                        format!("[[maybe_unused]] {decl}")
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
                    } else if let Some(call) =
                        seq_text.strip_suffix(".take()").filter(|_| held_from)
                    {
                        self.line(
                            out,
                            &format!("const cpl::Held<{holder_ty}> {held}_h = {call};"),
                        );
                        self.line(out, &format!("const {holder_ty}& {held} = *{held}_h;"));
                    } else {
                        self.line(out, &format!("{holder_ty} {held} = {seq_text};"));
                        if held_root(crate::uses::in_place(seq)).is_some()
                            && crate::lend::holds_heap(self.program, &seq.ty)
                        {
                            self.note_walked_copy(seq);
                        }
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

    /// `for x in read<[D]T>()` reads each `x` as its round begins, with no
    /// array, when the body can neither trap, stop early, read nor print:
    /// nothing can tell that from reading every value first.
    fn read_streamed(
        &mut self,
        id: LoopId,
        seq: &Expr,
        decl: &str,
        first: &[String],
        body: &Block,
        out: &mut String,
    ) -> bool {
        let ExprKind::Read(spec) = &seq.kind else {
            return false;
        };
        let cpl_ir::Spec::Array { dims, elem } = spec.as_ref() else {
            return false;
        };
        let scalar = match elem.as_ref() {
            cpl_ir::Spec::Int { .. } => true,
            cpl_ir::Spec::Plain(ty) => ty.is_scalar() && *ty != Ty::Char,
            _ => false,
        };
        if dims.len() != 1 || !scalar {
            return false;
        }
        let pushed = self.fused_pushes.last().map_or(usize::MAX, |(vec, _)| *vec);
        let traps = self.target.checks != crate::Checks::None;
        let call_ok = |e: &Expr| {
            crate::uses::effects::pure_call(
                self.program,
                &self.devirt,
                &self.sink_pure,
                self.fn_id,
                traps,
                e,
            )
        };
        if !crate::uses::effects::trap_free_stmts(&body.stmts, pushed, traps, &call_ok) {
            return false;
        }
        let len = self.dim(&dims[0]);
        let value = self.read(elem).text;
        let (at, end) = (self.temp(), self.temp());
        self.line(
            out,
            &format!("for (ll {at} = 0, {end} = {len}; {at} < {end}; {at}++) {{"),
        );
        let mut opening = vec![format!("{decl} = {value};")];
        opening.extend(first.iter().cloned());
        self.loop_body(id, &opening, body, out);
        true
    }
}

/// The variable a loop source is a place in, whatever its indices: they
/// run once, before the loop binds the place, so only the variable must
/// stay unchanged (`for d in divisors[a[p]]`).
fn held_root(e: &Expr) -> Option<(bool, usize)> {
    match &e.kind {
        ExprKind::Local(slot) => Some((false, *slot)),
        ExprKind::Global(id) => Some((true, *id)),
        ExprKind::Field(base, _) | ExprKind::Index(base, _) => held_root(base),
        _ => None,
    }
}
