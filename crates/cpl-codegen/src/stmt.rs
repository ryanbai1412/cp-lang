//! Statements as C++.

use crate::emit::{pad, Emitter, ADD, PRIMARY, REL, UNARY};
use crate::expr::string_literal;
use crate::names::ident;
use crate::walk::{self, Effects};
use cpl_ir::{
    AssignOp, BinOp, Block, Expr, ExprKind, Iter, LoopId, Place, PlaceKind, SortBy, Spec, Stmt, Ty,
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
        for stmt in &block.stmts {
            self.stmt(stmt, out);
        }
    }

    /// Writes the statements of a value block in line and gives back the
    /// expression that is its value.
    fn flatten<'a>(&mut self, mut e: &'a Expr, out: &mut String) -> &'a Expr {
        while let Some((stmts, value)) = flat(e) {
            for stmt in stmts {
                self.stmt(stmt, out);
            }
            e = value;
        }
        e
    }

    fn expr_stmt(&mut self, e: &Expr, out: &mut String) {
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
            ExprKind::Sort(place, sort_by) => self.sort_stmts(place, sort_by, out),
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
            _ => {
                let text = self.expr(e).text;
                self.line(out, &format!("{text};"));
            }
        }
    }

    pub fn sort_stmts(&mut self, place: &Place, sort_by: &SortBy, out: &mut String) {
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
        match sort_by {
            SortBy::Natural => {
                self.line(out, &format!("std::sort({seq}.begin(), {seq}.end());"));
            }
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
                let (kf, a, b) = (self.temp(), self.temp(), self.temp());
                self.line(
                    out,
                    &format!("auto {kf} = [&]({param}) -> {key_ty} {{ return {key_text}; }};"),
                );
                self.line(
                    out,
                    &format!("std::stable_sort({seq}.begin(), {seq}.end(), [&](const auto& {a}, const auto& {b}) {{ return {kf}({a}) < {kf}({b}); }});"),
                );
            }
            SortBy::Fn { func, is_order } => {
                let f_text = self.expr(func).text;
                let (kf, a, b) = (self.temp(), self.temp(), self.temp());
                self.line(out, &format!("auto {kf} = {f_text};"));
                if *is_order {
                    self.line(
                        out,
                        &format!("std::stable_sort({seq}.begin(), {seq}.end(), [&](const auto& {a}, const auto& {b}) {{ return {kf}({a}, {b}); }});"),
                    );
                } else {
                    self.line(
                        out,
                        &format!("std::stable_sort({seq}.begin(), {seq}.end(), [&](const auto& {a}, const auto& {b}) {{ return {kf}({a}) < {kf}({b}); }});"),
                    );
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
        self.indent -= 1;
        self.line(out, "}");
        if self.break_targets.contains(&id) {
            self.line(out, &format!("brk_{id}:;"));
        }
    }

    fn for_stmt(&mut self, id: LoopId, slot: usize, iter: &Iter, body: &Block, out: &mut String) {
        let ty = self.locals()[slot].ty.clone();
        let cpp = self.ty(&ty);
        let name = self.names[slot].clone();
        let stable = self.facts.stable((self.fn_id, false, slot));
        let hoisted = self.hoisted.contains(&slot);
        // The loop's own counter, when the body may change the variable.
        let var = if hoisted || !stable {
            self.temp()
        } else {
            name.clone()
        };
        let mut first = Vec::new();
        if hoisted {
            first.push(format!("{name} = {var};"));
        } else if !stable {
            first.push(format!("{cpp} {name} = {var};"));
        }
        match iter {
            Iter::Range { lo, hi, step, rev } => {
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
                        let advance = match &step_text {
                            Some(step) => format!("{var} += {step}"),
                            None => format!("{var}++"),
                        };
                        let start = fit(lo_code.text.clone(), lo_code.narrow);
                        self.line(
                            out,
                            &format!(
                                "for ({cpp} {var} = {start}{extra}; {var} < {bound}; {advance}) {{"
                            ),
                        );
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
                        let start = fit(format!("{} - 1", hi_code.at(ADD)), false);
                        self.line(
                            out,
                            &format!(
                                "for ({cpp} {var} = {start}{extra}; {var} >= {bound}; {var}--) {{"
                            ),
                        );
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
                        self.line(out, &format!("for (ll {k} = {b} > {a} ? ({b} - {a} - 1) / {step} : -1; {k} >= 0; {k}--) {{"));
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
        let whole = value;
        let value = self.flatten(value, out);
        if let (AssignOp::Set, PlaceKind::Local(_) | PlaceKind::Global(_), ExprKind::Read(spec)) =
            (op, &place.kind, &value.kind)
        {
            if let Spec::Array { dims, elem } = &**spec {
                let name = self.place(place);
                self.read_array(&name, false, dims, elem, out);
                return;
            }
        }
        let (global, index) = walk::root(place);
        let packed = self.is_packed(global, index);
        self.pack_init = packed && matches!(place.kind, PlaceKind::Local(_) | PlaceKind::Global(_));
        let mut v = self.expr(value);
        if packed && matches!(place.kind, PlaceKind::Index(..)) {
            v.text = self.pack(value, &v.text);
        }
        // A list built by the value's own block (a comprehension) is not
        // read again: it moves into the place.
        if let (AssignOp::Set, ExprKind::Local(built), Some((stmts, _))) = (op, &value.kind, flat(whole)) {
            let made_here = stmts.iter().any(|stmt| matches!(stmt, Stmt::Let(slot, _) if slot == built));
            if made_here && matches!(value.ty, Ty::List(_) | Ty::Str) {
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
                        self.line(out, &format!("{b}[{k}] {sym}= {};", v.text));
                        return;
                    }
                    _ => {}
                }
            }
        }
        let into_map = op == AssignOp::Set
            && matches!(&place.kind, PlaceKind::Index(base, _) if matches!(base.ty, Ty::Map(..) | Ty::HashMap(..)));
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

    pub fn stmt(&mut self, stmt: &Stmt, out: &mut String) {
        let program = self.program;
        match stmt {
            Stmt::Let(slot, init) => {
                let ty = self.locals()[*slot].ty.clone();
                let name = self.names[*slot].clone();
                let packed = self.is_packed(false, *slot);
                let cpp = match packed {
                    true => self.packed_ty(&ty),
                    false => self.ty(&ty),
                };
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
                if self.reused.contains(slot) && !hoisted && !packed && self.closure_id.is_none() {
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
                        let args = self.fill_args(dims, value.as_deref(), packed);
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
                    _ => {
                        self.pack_init = packed;
                        self.expr(init).text
                    }
                };
                if hoisted {
                    self.line(out, &format!("{name} = {text};"));
                } else if name.starts_with('_') {
                    // `_t` is a name for a value the program may not use.
                    self.line(out, &format!("[[maybe_unused]] {cpp} {name} = {text};"));
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
            Stmt::For(id, slot, iter, body) => self.for_stmt(*id, *slot, iter, body, out),
            Stmt::Return(None) if self.in_setup => self.line(out, "return false;"),
            Stmt::Return(None) => self.line(out, "return;"),
            Stmt::Return(Some(value)) | Stmt::Give(value) => {
                let value = self.flatten(value, out);
                if value.ty == Ty::Unit {
                    self.expr_stmt(value, out);
                    self.line(out, "return;");
                } else {
                    let text = self.expr(value).text;
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
