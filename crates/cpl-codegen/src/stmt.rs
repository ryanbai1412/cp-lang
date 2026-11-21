//! Statements as C++.

use crate::emit::{pad, Emitter, Pushed, ADD, PRIMARY, REL, UNARY};
use crate::expr::string_literal;
use crate::names::{fn_name, ident};
use crate::walk::{self, Effects};
use cpl_ir::{
    AssignOp, BinOp, Block, Expr, ExprKind, Iter, LoopId, Mutation, Place, PlaceKind, SortBy, Spec, Stmt, Ty,
};

/// A block with a value that is statements and then the value, with no
/// other way out: it can be written in line.
fn flat(e: &Expr) -> Option<(&[Stmt], &Expr)> {
    let ExprKind::Do(block) = &e.kind else {
        return None;
    };
    match block.stmts.split_last() {
        Some((Stmt::Give(value), rest)) if !walk::has_give(rest) => Some((rest, value)),
        _ => None,
    }
}

impl<'p> Emitter<'p> {
    /// Sizes each `Subset<D>` of the domain `base`, which was just set.
    fn size_subsets(&mut self, base: usize, out: &mut String) {
        let program = self.program;
        for &(subset, of) in &program.subsets {
            if of == base {
                self.need("fail");
                let (s, b) = (
                    ident(&program.domains[subset]),
                    ident(&program.domains[base]),
                );
                let limit = cpl_ir::MAX_SUBSET_BITS;
                self.line(
                    out,
                    &format!("if ({b}_len > {limit}) cpl::fail(\"a `Subset` needs a domain of at most {limit} values\");"),
                );
                self.line(out, &format!("{s}_lo = 0;"));
                self.line(out, &format!("{s}_len = 1LL << {b}_len;"));
            }
        }
    }

    fn line(&self, out: &mut String, text: &str) {
        out.push_str(&pad(self.indent));
        out.push_str(text);
        out.push('\n');
    }

    pub fn block(&mut self, block: &Block, out: &mut String) {
        for (k, stmt) in block.stmts.iter().enumerate() {
            self.tail = match stmt {
                Stmt::Let(_, Some(init)) if walk::path_root(init).is_some() => {
                    Some(Tail::of(&block.stmts[k + 1..]))
                }
                _ => None,
            };
            self.stmt(stmt, out);
        }
        self.tail = None;
    }

    /// Writes the statements of a value block in line and gives back the
    /// expression that is its value.
    fn flatten<'a>(&mut self, mut e: &'a Expr, out: &mut String) -> &'a Expr {
        while crate::expr::map_remove_do(e).is_none() {
            let Some((stmts, value)) = flat(e) else { break };
            for stmt in stmts {
                self.stmt(stmt, out);
            }
            e = value;
        }
        e
    }

    fn expr_stmt(&mut self, e: &Expr, out: &mut String) {
        if let Some((prefix, place, key_val)) = crate::expr::map_remove_do(e) {
            for stmt in prefix {
                self.stmt(stmt, out);
            }
            let p = self.place(place);
            let k = self.expr(key_val).text;
            self.line(out, &format!("{p}.erase({k});"));
            return;
        }
        let e = self.flatten(e, out);
        match &e.kind {
            ExprKind::Unit | ExprKind::Error => {}
            // Nothing of a `debug(...)` runs. A variable it alone reads
            // is still named, unevaluated, so g++ does not call it unused.
            ExprKind::Debug(_, args) => {
                for arg in args {
                    if matches!(arg.value.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
                        let name = self.expr(&arg.value).text;
                        self.line(out, &format!("(void)sizeof({name});"));
                    }
                }
            }
            ExprKind::Print(args, end) => self.print_stmts(args, end.as_deref(), out),
            ExprKind::Sort(place, sort_by, desc) => self.sort_stmts(place, sort_by, desc.as_deref(), out),
            ExprKind::If(cond, then, otherwise) if e.ty == Ty::Unit => {
                let cond = self.expr(cond).text;
                self.line(out, &format!("if ({cond}) {{"));
                self.indent += 1;
                self.expr_stmt(then, out);
                self.indent -= 1;
                if !matches!(otherwise.kind, ExprKind::Unit) {
                    self.line(out, "} else {");
                    self.indent += 1;
                    self.expr_stmt(otherwise, out);
                    self.indent -= 1;
                }
                self.line(out, "}");
            }
            // `s.remove(x)` or `s.insert(x)` for its effect alone: whether
            // `x` was there is not asked, so what answers it is not written.
            ExprKind::Mutate(Mutation::Remove | Mutation::Insert, _, _) => {
                let text = self.expr(e).text;
                let text = text.strip_suffix(" != 0").or_else(|| text.strip_suffix(".second")).unwrap_or(&text);
                self.line(out, &format!("{text};"));
            }
            // A value named and dropped, as what is left of a flattened
            // block: it does nothing, and `(void)` says so to the compiler.
            ExprKind::Local(_) | ExprKind::Global(_) => {
                let text = self.expr(e).text;
                self.line(out, &format!("(void){text};"));
            }
            _ => {
                let text = self.expr(e).text;
                self.line(out, &format!("{text};"));
            }
        }
    }

    pub fn sort_stmts(&mut self, place: &Place, sort_by: &SortBy, desc: Option<&Expr>, out: &mut String) {
        // Evaluate `desc` before binding `auto& r = p` so a side-effecting `desc`
        // cannot rehash/reallocate the outer container holding `place`.
        let desc = desc.map(|desc| {
            let d = self.expr(desc).text;
            let v = self.temp();
            self.line(out, &format!("bool {v} = {d};"));
            v
        });
        let mut p = self.place(place);
        if !self.place_is_plain(place) {
            let r = self.temp();
            self.line(out, &format!("auto& {r} = {p};"));
            p = r;
        }
        let seq = if matches!(&place.ty, Ty::Array { dims, .. } if dims.len() > 1) {
            format!("{p}.d")
        } else {
            p
        };
        // `desc:` asks the comparison the other way round, which leaves
        // equal elements in the order they had.
        let turned = |less: String, more: String| match &desc {
            None => less,
            Some(desc) => format!("{desc} ? {more} : {less}"),
        };
        match sort_by {
            SortBy::Natural => match &desc {
                None => self.line(out, &format!("std::sort({seq}.begin(), {seq}.end());")),
                Some(desc) => {
                    let (a, b) = (self.temp(), self.temp());
                    self.line(
                        out,
                        &format!("std::stable_sort({seq}.begin(), {seq}.end(), [&](const auto& {a}, const auto& {b}) {{ return {desc} ? {b} < {a} : {a} < {b}; }});"),
                    );
                }
            },
            SortBy::Key(slot, key) => {
                let elem = self.locals()[*slot].ty.clone();
                let cpp = self.ty(&elem);
                let param = if elem.is_scalar() {
                    format!("{cpp} {}", self.names[*slot])
                } else {
                    format!("const {cpp}& {}", self.names[*slot])
                };
                let key_ty = self.ty(&key.ty);
                let key_text = self.expr(key).text;
                let kf = self.temp();
                self.line(
                    out,
                    &format!("auto {kf} = [&]({param}) -> {key_ty} {{ return {key_text}; }};"),
                );
                if is_cheap_pure_key(key) {
                    let (a, b) = (self.temp(), self.temp());
                    let cmp = turned(format!("{kf}({a}) < {kf}({b})"), format!("{kf}({b}) < {kf}({a})"));
                    self.line(
                        out,
                        &format!("std::stable_sort({seq}.begin(), {seq}.end(), [&](const auto& {a}, const auto& {b}) {{ return {cmp}; }});"),
                    );
                } else {
                    self.need("sort_by");
                    let desc = desc.clone().unwrap_or_else(|| "false".to_string());
                    self.line(out, &format!("cpl::sort_by({seq}, {kf}, {desc});"));
                }
            }
            SortBy::Fn { func, is_order } => {
                let f_text = self.expr(func).text;
                let kf = self.temp();
                self.line(out, &format!("auto {kf} = {f_text};"));
                let desc = desc.clone().unwrap_or_else(|| "false".to_string());
                if *is_order {
                    self.need("sort_cmp");
                    self.line(out, &format!("cpl::sort_cmp({seq}, {kf}, {desc});"));
                } else {
                    self.need("sort_by");
                    self.line(out, &format!("cpl::sort_by({seq}, {kf}, {desc});"));
                }
            }
        }
    }

    fn if_stmt(
        &mut self,
        cond: &Expr,
        then: &Block,
        otherwise: &Block,
        out: &mut String,
        lead: &str,
    ) {
        let fused = found_entry(cond, then);
        if let Some((map, key)) = fused {
            let (m, k, it) = (self.expr(map).text, self.expr(key).text, self.temp());
            out.push_str(&format!("{lead}if (auto {it} = {m}.find({k}); {it} != {m}.end()) {{\n"));
            self.found.push((map.clone(), key.clone(), it));
        } else {
            let c = self.expr(cond).text;
            out.push_str(&format!("{lead}if ({c}) {{\n"));
        }
        self.indent += 1;
        self.block(then, out);
        self.indent -= 1;
        if fused.is_some() {
            self.found.pop();
        }
        match otherwise.stmts.as_slice() {
            [] => self.line(out, "}"),
            [Stmt::If(cond, then, otherwise)] => {
                let lead = format!("{}}} else ", pad(self.indent));
                self.if_stmt(cond, then, otherwise, out, &lead);
            }
            _ => {
                self.line(out, "} else {");
                self.indent += 1;
                self.block(otherwise, out);
                self.indent -= 1;
                self.line(out, "}");
            }
        }
    }

    /// The body of a loop, its `}` and the labels a `break` or
    /// `continue` from an inner loop jumps to. `first` are statements
    /// that open the body.
    fn loop_body(&mut self, id: LoopId, first: &[String], body: &Block, out: &mut String) {
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
    fn pushed_loop(&mut self, id: LoopId, decl: &str, first: &[String], seq: &Expr, body: &Block, out: &mut String) -> bool {
        let ExprKind::Call(callee, _) = &seq.kind else {
            return false;
        };
        if self.pushing.is_some() || self.closure_id.is_some() {
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
        let returns = cpl_ir::pushed::returns(body);
        let call = self.expr(seq).text;
        let name = fn_name(&self.program.fns[*callee].name);
        let Some(call) = call.strip_prefix(&format!("{name}(")).and_then(|rest| rest.strip_suffix(')')) else {
            return false;
        };
        let sep = if call.is_empty() { "" } else { ", " };
        let ret = match self.fn_id {
            Some(f) if self.program.fns[f].ret != Ty::Unit => Some(self.ty(&self.program.fns[f].ret)),
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
        self.line(out, &format!("{open}cpl_each_{name}({call}{sep}[&]({decl}) -> int {{"));
        self.indent += 1;
        for line in first {
            self.line(out, line);
        }
        self.bodies.push(Pushed { id, depth: 0, result: result.clone() });
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

    fn for_stmt(&mut self, id: LoopId, slot: usize, iter: &Iter, body: &Block, out: &mut String) {
        let ty = self.locals()[slot].ty.clone();
        let cpp = self.ty(&ty);
        let name = self.names[slot].clone();
        let stable = self.facts.stable((self.fn_id, false, slot));
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
                            None if cpp == "int" || hi_code.narrow => {
                                format!("for ({cpp} {var} = {start}{extra}; {var} <= {bound}; {var}++) {{")
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
                } else {
                    self.line(
                        out,
                        &format!("for ({cpp} {var} = {held}.lo; {var} < {held}.hi; {var}++) {{"),
                    );
                }
                self.loop_body(id, &first, body, out);
                self.indent -= 1;
                self.line(out, "}");
            }
            Iter::Subsets(mask_expr) => {
                let m_code = self.expr(mask_expr);
                let held = self.temp();
                let done = self.temp();
                self.line(out, "{");
                self.indent += 1;
                self.line(out, &format!("{cpp} {held} = {};", m_code.text));
                self.line(
                    out,
                    &format!(
                        "for ({cpp} {var} = {held}, {done} = 0; !{done}; {done} = !{var}, {var} = ({var} - 1) & {held}) {{"
                    ),
                );
                self.loop_body(id, &first, body, out);
                self.indent -= 1;
                self.line(out, "}");
            }
            Iter::Members(mask_expr) => {
                let mask = self.expr(mask_expr);
                let left = self.temp();
                self.line(
                    out,
                    &format!(
                        "for (unsigned long long {left} = (unsigned long long){}; {left}; {left} &= {left} - 1) {{",
                        mask.at(UNARY)
                    ),
                );
                let mut first = first;
                first.insert(0, format!("{cpp} {var} = ({cpp})__builtin_ctzll({left});"));
                self.loop_body(id, &first, body, out);
            }
            Iter::Seq(seq, rev) => {
                let effects = Effects::of_block(body);
                // The loop walks the value as it was when the loop began,
                // so a container the body changes is walked as a copy.
                let untouched = match walk::path_root(seq) {
                    Some((false, local)) => !effects.locals.contains(&local),
                    Some((true, global)) => {
                        !effects.globals.contains(&global)
                            && !effects
                                .calls
                                .iter()
                                .any(|f| self.summaries[*f].writes.contains(&global))
                    }
                    None => false,
                };
                let is_map = matches!(seq.ty, Ty::Map(..) | Ty::HashMap(..));
                let is_grid = matches!(&seq.ty, Ty::Array { dims, .. } if dims.len() > 1);
                if !rev && matches!(seq.kind, ExprKind::Call(..)) {
                    let decl = if ty.is_scalar() || var != name {
                        format!("{cpp} {var}")
                    } else {
                        format!("const {cpp}& {var}")
                    };
                    if self.pushed_loop(id, &decl, &first, seq, body, out) {
                        return;
                    }
                }
                let seq_code = self.expr(seq);
                let opened = !(untouched && !rev && !is_map);
                let holder = if opened {
                    self.line(out, "{");
                    self.indent += 1;
                    let held = self.temp();
                    let holder_ty = self.ty(&seq.ty);
                    if untouched {
                        self.line(
                            out,
                            &format!("const {holder_ty}& {held} = {};", seq_code.text),
                        );
                    } else {
                        self.line(out, &format!("{holder_ty} {held} = {};", seq_code.text));
                    }
                    held
                } else {
                    seq_code.at(PRIMARY)
                };
                let items = if is_grid {
                    format!("{holder}.d")
                } else {
                    holder
                };
                let decl = if ty.is_scalar() || var != name {
                    format!("{cpp} {var}")
                } else {
                    format!("const {cpp}& {var}")
                };
                if is_map {
                    let it = self.temp();
                    let mut opening = Vec::new();
                    if *rev {
                        self.line(out, &format!("for (auto {it} = {items}.rbegin(); {it} != {items}.rend(); ++{it}) {{"));
                        opening.push(format!("{cpp} {var}({it}->first, {it}->second);"));
                    } else {
                        self.line(out, &format!("for (const auto& {it} : {items}) {{"));
                        opening.push(format!("{cpp} {var}({it}.first, {it}.second);"));
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
                    self.line(out, &format!("for ({decl} : {items}) {{"));
                    self.loop_body(id, &first, body, out);
                }
                if opened {
                    self.indent -= 1;
                    self.line(out, "}");
                }
            }
        }
    }

    fn assign(&mut self, place: &Place, op: AssignOp, value: &Expr, out: &mut String) {
        if let (AssignOp::Set, PlaceKind::Global(id)) = (op, &place.kind) {
            if self.const_globals.contains(id) {
                return;
            }
        }
        let whole = value;
        let value = self.flatten(value, out);
        if let (AssignOp::Set, PlaceKind::Local(_) | PlaceKind::Global(_)) = (op, &place.kind) {
            if matches!((&value.kind, &value.ty), (ExprKind::Tuple(items), Ty::List(_)) if items.is_empty())
                || matches!(&value.kind, ExprKind::Str(bytes) if bytes.is_empty())
            {
                let name = self.place(place);
                self.line(out, &format!("{name}.clear();"));
                return;
            }
        }
        if let (AssignOp::Set, PlaceKind::Local(_) | PlaceKind::Global(_), ExprKind::Read(spec)) =
            (op, &place.kind, &value.kind)
        {
            match &**spec {
                Spec::Array { dims, elem } => {
                    let name = self.place(place);
                    self.read_array(&name, false, dims, elem, out);
                    return;
                }
                Spec::Plain(Ty::Str) => {
                    self.need("read_str");
                    let name = self.place(place);
                    self.line(out, &format!("cpl::read_str_into({name});"));
                    return;
                }
                _ => {}
            }
        }
        let mut v = self.expr(value);
        // A value built by its own block (a comprehension) is not
        // read again: it moves into the place.
        if let (AssignOp::Set, ExprKind::Local(built), Some((stmts, _))) = (op, &value.kind, flat(whole)) {
            let made_here = stmts.iter().any(|stmt| matches!(stmt, Stmt::Let(slot, _) if slot == built));
            if made_here && !value.ty.is_scalar() && !matches!(value.ty, Ty::Fn(..)) {
                v.text = format!("std::move({})", v.text);
            }
        }
        // Handle map/hashmap fold-on-missing-key (`max=`, `min=`, `*=`, `+=`, `-=`, `|=`, `^=`, `&=`).
        if let PlaceKind::Index(base, idx) = &place.kind {
            if matches!(base.ty, Ty::Map(..) | Ty::HashMap(..)) && op != AssignOp::Set {
                let b = self.place(base);
                let k = self.expr(&idx[0]).text;
                match op {
                    AssignOp::Max => {
                        self.need("map_fold");
                        self.line(out, &format!("cpl::map_max({b}, {k}, {});", v.text));
                        return;
                    }
                    AssignOp::Min => {
                        self.need("map_fold");
                        self.line(out, &format!("cpl::map_min({b}, {k}, {});", v.text));
                        return;
                    }
                    AssignOp::Binary(BinOp::Mul) => {
                        self.need("map_fold");
                        self.line(out, &format!("cpl::map_mul({b}, {k}, {});", v.text));
                        return;
                    }
                    AssignOp::Binary(BinOp::BitAnd) => {
                        self.need("map_fold");
                        self.line(out, &format!("cpl::map_and({b}, {k}, {});", v.text));
                        return;
                    }
                    AssignOp::Binary(BinOp::Add | BinOp::Sub | BinOp::BitOr | BinOp::BitXor) => {
                        let sym = match op {
                            AssignOp::Binary(BinOp::Sub) => "-",
                            AssignOp::Binary(BinOp::BitOr) => "|",
                            AssignOp::Binary(BinOp::BitXor) => "^",
                            _ => "+",
                        };
                        if matches!(base.ty, Ty::HashMap(..))
                            && expr_may_alias_root(value, walk::root(base))
                        {
                            let tmp = self.temp();
                            self.line(out, &format!("auto {tmp} = {};", v.text));
                            self.line(out, &format!("{b}[{k}] {sym}= {tmp};"));
                        } else {
                            self.line(out, &format!("{b}[{k}] {sym}= {};", v.text));
                        }
                        return;
                    }
                    _ => {}
                }
            }
        }
        let into_map = op == AssignOp::Set
            && matches!(&place.kind, PlaceKind::Index(base, _) if matches!(base.ty, Ty::Map(..) | Ty::HashMap(..)));
        let hash_alias = op == AssignOp::Set
            && matches!(
                &place.kind,
                PlaceKind::Index(base, _)
                    if matches!(base.ty, Ty::HashMap(..))
                        && expr_may_alias_root(value, walk::root(base))
            );
        let p = self.place_at(place, into_map);
        let ty = self.ty(&place.ty);
        // `p = f(p, v)` names the place twice.
        let twice = |this: &mut Self, out: &mut String, make: &dyn Fn(&str, &str) -> String| {
            if this.place_is_plain(place) {
                this.line(out, &make(&p, &v.text));
            } else {
                let (value_name, r) = (this.temp(), this.temp());
                this.line(out, "{");
                this.indent += 1;
                this.line(out, &format!("{ty} {value_name} = {};", v.text));
                this.line(out, &format!("auto& {r} = {p};"));
                this.line(out, &make(&r, &value_name));
                this.indent -= 1;
                this.line(out, "}");
            }
        };
        match op {
            AssignOp::Set if hash_alias => {
                let tmp = self.temp();
                self.line(out, &format!("auto {tmp} = {};", v.text));
                self.line(out, &format!("{p} = std::move({tmp});"));
            }
            AssignOp::Set => self.line(out, &format!("{p} = {};", v.text)),
            AssignOp::Max => twice(self, out, &|p, v| {
                format!("{p} = std::max<{ty}>({p}, {v});")
            }),
            AssignOp::Min => twice(self, out, &|p, v| {
                format!("{p} = std::min<{ty}>({p}, {v});")
            }),
            AssignOp::Binary(bop) => match (&place.ty, bop) {
                (Ty::Int, BinOp::Div | BinOp::Rem) => {
                    let positive =
                        self.facts.sign(self.fn_id, value) == crate::ranges::Sign::Positive;
                    let helper = match (positive, bop == BinOp::Div) {
                        (true, true) => "divp",
                        (true, false) => "modp",
                        (false, true) => "ediv",
                        (false, false) => "emod",
                    };
                    self.need(helper);
                    twice(self, out, &|p, v| format!("{p} = cpl::{helper}({p}, {v});"));
                }
                (Ty::Int, BinOp::Pow) => {
                    self.need("ipow");
                    twice(self, out, &|p, v| format!("{p} = cpl::ipow({p}, {v});"));
                }
                (Ty::I128, BinOp::Div | BinOp::Rem) => {
                    let helper = if bop == BinOp::Div { "ediv128" } else { "emod128" };
                    self.need(helper);
                    twice(self, out, &|p, v| format!("{p} = cpl::{helper}({p}, {v});"));
                }
                (Ty::I128 | Ty::U64, BinOp::Pow) => {
                    self.need("wpow");
                    twice(self, out, &|p, v| format!("{p} = cpl::wpow({p}, {v});"));
                }
                (Ty::Float, BinOp::Pow) => {
                    twice(self, out, &|p, v| format!("{p} = std::pow({p}, {v});"))
                }
                _ => {
                    let sym = match bop {
                        BinOp::Sub => "-",
                        BinOp::Mul => "*",
                        BinOp::Div => "/",
                        BinOp::Rem => "%",
                        BinOp::BitAnd => "&",
                        BinOp::BitOr => "|",
                        BinOp::BitXor => "^",
                        BinOp::Shl => "<<",
                        BinOp::Shr => ">>",
                        _ => "+",
                    };
                    self.line(out, &format!("{p} {sym}= {};", v.text));
                }
            },
        }
    }

    /// Whether the `let` of `slot` can be a reference to what `init`
    /// names instead of a copy of it. It can when nothing changes either
    /// for as long as the name lives: the name is set once and never
    /// changed, `init` is a path into a global or a parameter, and the
    /// statements after the `let` in its block (`tail`) neither store to
    /// that variable nor call anything that could. Without a `tail` the
    /// whole fn is held to that.
    fn reads_in_place(&self, slot: usize, init: &Expr, tail: Option<&Tail>) -> bool {
        if self.closure_id.is_some() {
            return false;
        }
        if init.ty.is_scalar() || matches!(init.ty, Ty::Fn(..)) || self.spent.contains(&slot) {
            return false;
        }
        if !matches!(init.kind, ExprKind::Field(..) | ExprKind::Index(..)) {
            return false;
        }
        let Some((global, root)) = walk::path_root(init) else {
            return false;
        };
        let params = self.fn_id.map_or(0, |id| self.program.fns[id].params.len());
        if !global && root >= params {
            return false;
        }
        if self.facts.stores((self.fn_id, false, slot)) > 1 {
            return false;
        }
        let Some(tail) = tail else {
            let Some(id) = self.fn_id else {
                return false;
            };
            let body = &self.program.fns[id].body;
            let own = crate::ranges::Facts::of_fn(id, body);
            let stored = match global {
                true => own.stores((None, true, root)),
                false => own.stores((Some(id), false, root)),
            };
            return stored == 0 && !Tail::of(&body.stmts).calls;
        };
        if tail.opaque {
            return false;
        }
        let effects = &tail.effects;
        match global {
            // A parameter may be another name for the global.
            true => {
                !effects.globals.contains(&root)
                    && effects.locals.iter().all(|local| *local >= params)
                    && !effects.calls.iter().any(|f| self.summaries[*f].writes.contains(&root))
            }
            false => !effects.locals.contains(&root) && !tail.calls,
        }
    }

    pub fn stmt(&mut self, stmt: &Stmt, out: &mut String) {
        if let (Some(sites), Some(span)) = (&mut self.sites, stmt.span()) {
            if self.counted.contains(&(stmt as *const Stmt)) {
                sites.push(span);
                let id = sites.len() - 1;
                self.line(out, &format!("cpl_prof::mark({id});"));
            }
        }
        let program = self.program;
        match stmt {
            // A fn written as `cpl_each_f`: its list is the callback.
            Stmt::Let(slot, _) if self.pushing == Some(*slot) && self.closure_id.is_none() => {}
            Stmt::Expr(Expr { kind: ExprKind::Mutate(how, place, items), .. })
                if self.pushing.is_some()
                    && self.closure_id.is_none()
                    && matches!(place.kind, PlaceKind::Local(slot) if Some(slot) == self.pushing) =>
            {
                let item = self.flatten(&items[0], out);
                let text = self.expr(item).text;
                match how {
                    Mutation::Extend => {
                        let x = self.temp();
                        self.line(out, &format!("for (auto&& {x} : {text}) if (int cpl_r = cpl_f({x})) return cpl_r;"));
                    }
                    _ => self.line(out, &format!("if (int cpl_r = cpl_f({text})) return cpl_r;")),
                }
            }
            Stmt::Return(Some(_)) if self.pushing.is_some() && self.closure_id.is_none() => self.line(out, "return 0;"),
            // In the body of a loop over a `yields` fn, which is a callback.
            Stmt::Return(value) if !self.bodies.is_empty() => {
                if let Some(value) = value {
                    let value = self.flatten(value, out);
                    match self.bodies[0].result.clone() {
                        Some(held) if value.ty != Ty::Unit => {
                            let text = self.expr(value).text;
                            self.line(out, &format!("{held} = {text};"));
                        }
                        _ => self.expr_stmt(value, out),
                    }
                }
                self.line(out, "return 2;");
            }
            Stmt::Break(to) if self.bodies.last().is_some_and(|b| to.map_or(b.depth == 0, |to| to == b.id)) => {
                self.line(out, "return 1;")
            }
            Stmt::Continue(to) if self.bodies.last().is_some_and(|b| to.map_or(b.depth == 0, |to| to == b.id)) => {
                self.line(out, "return 0;")
            }
            Stmt::Let(slot, init) => {
                let tail = self.tail.take();
                let ty = self.locals()[*slot].ty.clone();
                let name = self.names[*slot].clone();
                let cpp = self.ty(&ty);
                let hoisted = self.hoisted.contains(slot);
                let Some(init) = init else {
                    if ty != Ty::Unit {
                        if hoisted {
                            self.line(out, &format!("{name} = {cpp}();"));
                        } else {
                            self.line(out, &format!("{cpp} {name}{};", self.zero_init(&ty)));
                        }
                    }
                    return;
                };
                if ty == Ty::Unit {
                    self.expr_stmt(init, out);
                    return;
                }
                let reused = self.reused.contains(slot)
                    && !hoisted
                    && self.closure_id.is_none()
                    && cpp.starts_with("std::vector<");
                if reused && !matches!(init.kind, ExprKind::Fill(..)) {
                    self.line(out, &format!("static {cpp} {name};"));
                    self.line(out, &format!("{name}.clear();"));
                    return;
                }
                let init = self.flatten(init, out);
                match &init.kind {
                    ExprKind::Read(spec) => {
                        if let Spec::Array { dims, elem } = &**spec {
                            self.read_array(&name, !hoisted, dims, elem, out);
                            return;
                        }
                    }
                    ExprKind::Fill(dims, value) if !hoisted && dims.len() == 1 => {
                        let args = self.fill_args(dims, value.as_deref());
                        if reused {
                            // One buffer, filled again each time round.
                            self.line(out, &format!("static {cpp} {name};"));
                            self.line(
                                out,
                                &match value {
                                    Some(_) => format!("{name}.assign({args});"),
                                    None => format!("{name}.clear(); {name}.resize({args});"),
                                },
                            );
                            return;
                        }
                        self.line(out, &format!("{cpp} {name}({args});"));
                        return;
                    }
                    _ => {}
                }
                let text = match &init.kind {
                    ExprKind::Field(base, index) => match &base.kind {
                        ExprKind::Local(of) if self.spent.contains(of) => {
                            format!("std::get<{index}>(std::move({}))", self.names[*of])
                        }
                        _ => self.expr(init).text,
                    },
                    _ => self.expr(init).text,
                };
                if hoisted {
                    self.line(out, &format!("{name} = {text};"));
                } else if name.starts_with('_') {
                    // `_t` is a name for a value the program may not use.
                    self.line(out, &format!("[[maybe_unused]] {cpp} {name} = {text};"));
                } else if self.reads_in_place(*slot, init, tail.as_ref()) {
                    // `let sp = sorted[b]`: a name for what is already
                    // there. A copy would cost the whole list.
                    self.line(out, &format!("const {cpp}& {name} = {text};"));
                } else if let Some(args) = sized_ctor(&cpp, &text) {
                    // `std::vector<ll> f((size_t)(n), 1);`: the type once.
                    self.line(out, &format!("{cpp} {name}({args});"));
                } else if matches!(ty, Ty::Fn(..))
                    && self.closure_id.is_none()
                    && matches!(init.kind, ExprKind::Closure(cid) if self.devirt.local_auto(*slot, cid))
                    && self.facts.stores((self.fn_id, false, *slot)) <= 1
                {
                    self.line(out, &format!("auto {name} = {text};"));
                } else {
                    self.line(out, &format!("{cpp} {name} = {text};"));
                }
            }
            Stmt::Assign(place, op, value) => self.assign(place, *op, value, out),
            Stmt::Expr(e) => self.expr_stmt(e, out),
            Stmt::If(cond, then, otherwise) => {
                let lead = pad(self.indent);
                self.if_stmt(cond, then, otherwise, out, &lead);
            }
            Stmt::While(id, cond, body) => {
                let cond = self.expr(cond).text;
                self.line(out, &format!("while ({cond}) {{"));
                self.loop_body(*id, &[], body, out);
            }
            Stmt::For(id, slot, iter, body) => {
                let inside = walk::inside_list(*slot, iter, body, self.locals());
                self.inside.extend(inside);
                self.for_stmt(*id, *slot, iter, body, out);
                self.inside.truncate(self.inside.len() - usize::from(inside.is_some()));
            }
            Stmt::Return(None) if self.in_setup => self.line(out, "return false;"),
            Stmt::Return(None) => self.line(out, "return;"),
            Stmt::Return(Some(value)) | Stmt::Give(value) => {
                let value = self.flatten(value, out);
                if value.ty == Ty::Unit {
                    self.expr_stmt(value, out);
                    self.line(out, "return;");
                } else {
                    let is_return = matches!(stmt, Stmt::Return(_));
                    self.last_use_call = is_return
                        && self.closure_id.is_none()
                        && self.fn_id.is_some()
                        && matches!(value.kind, ExprKind::Call(..));
                    let text = if is_return && self.closure_id.is_none() {
                        if let ExprKind::Tuple(items) = &value.kind {
                            if !items.is_empty() && !matches!(value.ty, Ty::Unit | Ty::Error | Ty::Map(..) | Ty::HashMap(..)) {
                                let params = self.fn_id.map_or(0, |id| self.program.fns[id].params.len());
                                let ty = self.ty(&value.ty);
                                let struct_id = match value.ty {
                                    Ty::Struct(s) => Some(s),
                                    _ => None,
                                };
                                let rendered: Vec<String> = items
                                    .iter()
                                    .enumerate()
                                    .map(|(idx, item)| {
                                        if struct_id.is_some_and(|s| {
                                            self.devirt.field_elidable(s, idx)
                                                && self.devirt.skippable_value(item)
                                        }) {
                                            return "false".to_string();
                                        }
                                        if let ExprKind::Local(slot) = item.kind {
                                            if (slot >= params
                                                || self.fn_id.is_some_and(|id| self.sunk[id].contains(&slot)))
                                                && !item.ty.is_scalar()
                                                && !matches!(item.ty, Ty::Fn(..))
                                                && !items.iter().enumerate().any(|(j, other)| {
                                                    j != idx && {
                                                        let mut used = false;
                                                        crate::walk::deep_expr(other, &mut |e| {
                                                            if matches!(e.kind, ExprKind::Local(s) if s == slot) {
                                                                used = true;
                                                            }
                                                        });
                                                        used
                                                    }
                                                })
                                            {
                                                return format!("std::move({})", self.names[slot]);
                                            }
                                        }
                                        let v = self.expr(item).text;
                                        match struct_id
                                            .and_then(|s| self.narrow_ints.get(&(s, idx)))
                                        {
                                            Some(cpl_ir::IntWidth::U32) => format!("uint32_t({v})"),
                                            Some(cpl_ir::IntWidth::I32) => format!("int32_t({v})"),
                                            None => v,
                                        }
                                    })
                                    .collect();
                                format!("{ty}{{{}}}", rendered.join(", "))
                            } else {
                                self.expr(value).text
                            }
                        } else {
                            self.expr(value).text
                        }
                    } else {
                        self.expr(value).text
                    };
                    self.last_use_call = false;
                    self.line(out, &format!("return {text};"));
                }
            }
            Stmt::Break(None) => self.line(out, "break;"),
            Stmt::Break(Some(id)) => self.line(out, &format!("goto brk_{id};")),
            Stmt::Continue(None) => self.line(out, "continue;"),
            Stmt::Continue(Some(id)) => self.line(out, &format!("goto cont_{id};")),
            Stmt::Assert {
                cond,
                message,
                text,
                line,
                ..
            } => {
                self.need("fail");
                let cond = self.expr(cond).text;
                let report = format!("assertion failed: {text} (line {line})");
                match message {
                    None => self.line(
                        out,
                        &format!(
                            "if (!({cond})) cpl::fail({});",
                            string_literal(report.as_bytes())
                        ),
                    ),
                    Some(message) => {
                        let message = self.expr(message).text;
                        let report = string_literal(format!("{report}: ").as_bytes());
                        self.line(out, &format!("if (!({cond})) cpl::fail((std::string({report}) + {message}).c_str());"));
                    }
                }
            }
            Stmt::DefDomain { id, lo, len } => {
                let d = ident(&program.domains[*id]);
                let lo = self.expr(lo).text;
                let len = self.expr(len).text;
                self.line(out, &format!("{d}_lo = {lo};"));
                self.line(out, &format!("{d}_len = std::max<ll>({len}, 0);"));
                self.size_subsets(*id, out);
            }
            Stmt::DefDomainFromVal { id, val } => {
                let d = ident(&program.domains[*id]);
                let v = self.expr(val).text;
                let tmp = self.temp();
                self.line(out, &format!("auto {tmp} = {v};"));
                self.line(out, &format!("{d}_lo = {tmp}.lo;"));
                self.line(out, &format!("{d}_len = std::max<ll>({tmp}.len, 0);"));
                self.size_subsets(*id, out);
            }
            Stmt::Trap(message, _) => {
                self.need("fail");
                self.line(
                    out,
                    &format!("cpl::fail({});", string_literal(message.as_bytes())),
                );
            }
        }
    }
}

/// True when `e` may evaluate to a reference into the container at `root`
/// (which an open-addressing `HashMap` insert/rehash on the LHS could invalidate).
fn expr_may_alias_root(e: &Expr, root: (bool, usize)) -> bool {
    let mut alias = false;
    walk::deep_expr(e, &mut |sub| match &sub.kind {
        ExprKind::Local(id) if !root.0 && *id == root.1 => alias = true,
        ExprKind::Global(id) if root.0 && *id == root.1 => alias = true,
        ExprKind::Call(..) | ExprKind::CallExpr(..) => alias = true,
        _ => {}
    });
    alias
}

/// For `if k in m:` whose body reads `m[k]` and can't change `m` or
/// `k`: the map and the key. The C++ then finds the entry once.
fn found_entry<'p>(cond: &'p Expr, then: &Block) -> Option<(&'p Expr, &'p Expr)> {
    let ExprKind::Builtin(cpl_ir::Builtin::Contains, args) = &cond.kind else {
        return None;
    };
    let [map, key] = args.as_slice() else {
        return None;
    };
    let plain = |e: &Expr| matches!(e.kind, ExprKind::Local(_) | ExprKind::Global(_));
    if !matches!(map.ty, Ty::Map(..) | Ty::HashMap(..)) || !plain(map) || !plain(key) {
        return None;
    }
    let (mut reads, mut quiet) = (false, true);
    walk::deep_exprs(then, &mut |e| match &e.kind {
        ExprKind::Index(base, at) if base.kind == map.kind && at.len() == 1 && at[0].kind == key.kind => reads = true,
        ExprKind::Mutate(..) | ExprKind::Call(..) | ExprKind::CallExpr(..) | ExprKind::Closure(_) => quiet = false,
        _ => {}
    });
    walk::deep_stmts(then, &mut |stmt| {
        if !matches!(stmt, Stmt::Expr(_) | Stmt::Return(_) | Stmt::Give(_) | Stmt::If(..) | Stmt::Let(..)) {
            quiet = false;
        }
    });
    (reads && quiet).then_some((map, key))
}

/// What the statements after a `let` in its block can do, for
/// [`Emitter::reads_in_place`].
pub(crate) struct Tail {
    effects: Effects,
    /// A call of a fn of the program.
    calls: bool,
    /// A call through a fn value, or a closure: code whose stores are not
    /// in `effects`.
    opaque: bool,
}

impl Tail {
    fn of(stmts: &[Stmt]) -> Tail {
        let block = Block { stmts: stmts.to_vec() };
        let (mut calls, mut opaque) = (false, false);
        walk::deep_exprs(&block, &mut |e| match e.kind {
            ExprKind::Call(..) => calls = true,
            ExprKind::CallExpr(..) | ExprKind::Closure(_) => {
                calls = true;
                opaque = true;
            }
            _ => {}
        });
        Tail { effects: Effects::of_block(&block), calls, opaque }
    }
}

/// The arguments of `text` when it is exactly `cpp((size_t)(..), ..)`:
/// a container of the declared type built with a size. The size cast
/// first means the arguments can't be read as a declaration.
fn sized_ctor<'t>(cpp: &str, text: &'t str) -> Option<&'t str> {
    let args = text.strip_prefix(cpp)?.strip_prefix('(')?.strip_suffix(')')?;
    if !args.starts_with("(size_t)(") {
        return None;
    }
    // The parenthesis taken off the end is the one that opened first.
    let mut depth = 0i32;
    for c in args.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return None;
        }
    }
    (depth == 0 && !args.contains('"') && !args.contains('\'')).then_some(args)
}

/// True when a `sort(by:)` key expression is pure, trap-free, and O(1), so comparing
/// `kf(a) < kf(b)` in-place is safe and avoids allocating `keys`/`order`/`sorted` buffers.
fn is_cheap_pure_key(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Unit
        | ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Char(_)
        | ExprKind::None
        | ExprKind::Local(_)
        | ExprKind::Global(_) => true,
        ExprKind::Some(inner) | ExprKind::Field(inner, _) => is_cheap_pure_key(inner),
        ExprKind::Unary(cpl_ir::UnOp::Not | cpl_ir::UnOp::BitNot, inner) => is_cheap_pure_key(inner),
        ExprKind::Tuple(items) | ExprKind::Variant(_, items) => {
            items.iter().all(is_cheap_pure_key)
        }
        _ => false,
    }
}
