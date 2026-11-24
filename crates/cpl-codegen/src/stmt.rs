//! Statements as C++.

use crate::emit::{pad, Emitter, Pushed, ADD, PRIMARY, REL};
use crate::expr::string_literal;
use crate::lend::Hold;
use crate::names::{dom_ident, fn_name};
use crate::uses::effects::{is_cheap_pure_key, writes_nothing, Touch};
use crate::walk;
use cpl_ir::{
    AssignOp, BinOp, Block, Builtin, Expr, ExprKind, Iter, LoopId, Mutation, Place, PlaceKind,
    SortBy, Spec, Stmt, Ty,
};

mod loops;

mod assign;

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
    /// A plain domain's position is an `int`: a checked build traps on one
    /// with more positions than that holds (a product or sum domain is
    /// widened instead).
    fn domain_fits(&mut self, out: &mut String, id: usize, d: &str) {
        let proven = (self.facts.domain_max_len(id)).is_some_and(|n| n <= i64::from(i32::MAX));
        if self.target.checks == crate::Checks::None
            || proven
            || crate::width::is_wide(self.program, id)
        {
            return;
        }
        self.need("fail");
        self.line(
            out,
            &format!("if ({d}_len > 2147483647) cpl::fail(\"a domain has more than 2^31 - 1 positions\");"),
        );
    }

    pub(crate) fn line(&self, out: &mut String, text: &str) {
        out.push_str(&pad(self.indent));
        out.push_str(text);
        out.push('\n');
    }

    pub fn block(&mut self, block: &Block, out: &mut String) {
        for k in 0..block.stmts.len() {
            self.tail = self.tail_at(&block.stmts, k);
            self.stmt_owning(block, k, out);
        }
        self.tail = None;
    }

    /// `let l = limbs` in a fn's body, where `limbs` is a parameter taken by
    /// value (see [`crate::scope::sunk_params`]) that nothing reads after.
    pub(crate) fn takes_sunk_param(&self, block: &Block, k: usize) -> bool {
        let (Stmt::Let(_, Some(init)), Some(id)) = (&block.stmts[k], self.fn_id) else {
            return false;
        };
        let ExprKind::Local(p) = init.kind else {
            return false;
        };
        std::ptr::eq(block, &self.program.fns[id].body)
            && self.sunk[id].contains(&p)
            && !init.ty.is_scalar()
            && !matches!(init.ty, Ty::Fn(..))
            && crate::uses::unread_after(self.program, block, k, p)
    }

    /// Writes the statements of a value block in line and gives back the
    /// expression that is its value.
    pub(crate) fn flatten<'a>(&mut self, mut e: &'a Expr, out: &mut String) -> &'a Expr {
        while crate::expr::map_remove_do(e).is_none() {
            let Some((stmts, value)) = flat(e) else { break };
            for k in 0..stmts.len() {
                // The value is read inside the statement the block is in,
                // whose other parts this cannot see: a `let` it reads owns.
                self.tail = match &stmts[k] {
                    Stmt::Let(slot, _) if crate::uses::mentions_any(self.program, value, *slot) => {
                        None
                    }
                    _ => self.tail_at(stmts, k),
                };
                self.moves_last = self.dies_here(&stmts[k]);
                self.stmt_presized(stmts, k, out);
                self.moves_last = false;
            }
            self.tail = None;
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
        let mark = self.pin_stored(e, out);
        self.flat_expr_stmt(e, out);
        self.pinned.truncate(mark);
    }

    fn flat_expr_stmt(&mut self, e: &Expr, out: &mut String) {
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
            ExprKind::Sort(place, sort_by, desc) => {
                self.sort_stmts(place, sort_by, desc.as_deref(), out)
            }
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
            ExprKind::Mutate(Mutation::Push, place, args) if self.fused_push(place, args, out) => {}
            // `s.remove(x)` or `s.insert(x)` for its effect alone: whether
            // `x` was there is not asked, so what answers it is not written.
            ExprKind::Mutate(Mutation::Remove | Mutation::Insert, _, _) => {
                let text = self.expr(e).text;
                let text = text
                    .strip_suffix(" != 0")
                    .or_else(|| text.strip_suffix(".second"))
                    .unwrap_or(&text);
                self.line(out, &format!("{text};"));
            }
            // A value named and dropped, as what is left of a flattened
            // block: it does nothing, and `(void)` says so to the compiler.
            ExprKind::Local(_) | ExprKind::Global(_) | ExprKind::Index(..) => {
                let text = self.expr(e).text;
                self.line(out, &format!("(void){text};"));
            }
            _ => {
                let text = self.expr(e).text;
                self.line(out, &format!("{text};"));
            }
        }
    }

    pub fn sort_stmts(
        &mut self,
        place: &Place,
        sort_by: &SortBy,
        desc: Option<&Expr>,
        out: &mut String,
    ) {
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
            SortBy::Natural => {
                self.need("sorted");
                let lt = match &desc {
                    None => String::new(),
                    Some(desc) => {
                        let (a, b) = (self.temp(), self.temp());
                        format!(", [&](const auto& {a}, const auto& {b}) {{ return {desc} ? {b} < {a} : {a} < {b}; }}")
                    }
                };
                self.line(
                    out,
                    &format!("cpl::sort_seq({seq}.begin(), {seq}.end(){lt});"),
                );
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
                let kf = self.temp();
                self.line(
                    out,
                    &format!("auto {kf} = [&]({param}) -> {key_ty} {{ return {key_text}; }};"),
                );
                if is_cheap_pure_key(key) {
                    self.need("sorted");
                    let (a, b) = (self.temp(), self.temp());
                    let cmp = turned(
                        format!("{kf}({a}) < {kf}({b})"),
                        format!("{kf}({b}) < {kf}({a})"),
                    );
                    self.line(
                        out,
                        &format!("cpl::stable_sort_lt({seq}.begin(), {seq}.end(), [&](const auto& {a}, const auto& {b}) {{ return {cmp}; }});"),
                    );
                } else {
                    self.need("sort_by");
                    let desc = desc.clone().unwrap_or_else(|| "false".to_string());
                    self.line(out, &format!("cpl::sort_by({seq}, {kf}, {desc});"));
                }
            }
            SortBy::Fn { func, is_order } => {
                // The sort only calls its key, so a closure written there lends
                // what it captures (see `crate::lend::only_called`).
                // (E0602 keeps the list being sorted out of the key.)
                let lent = match func.kind {
                    ExprKind::Closure(cid) => {
                        self.call_scoped.insert(cid);
                        self.borrow_closures.insert(cid).then_some(cid)
                    }
                    _ => None,
                };
                let f_text = self.expr(func).text;
                if let ExprKind::Closure(cid) = func.kind {
                    self.call_scoped.remove(&cid);
                }
                if let Some(cid) = lent {
                    self.borrow_closures.remove(&cid);
                }
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
        if let (Some(taken), true) = (crate::prune::known(cond), lead.trim().is_empty()) {
            let live = if taken { then } else { otherwise };
            if !live.stmts.is_empty() {
                out.push_str(&format!("{lead}{{\n"));
                self.indent += 1;
                self.block(live, out);
                self.indent -= 1;
                self.line(out, "}");
            }
            return;
        }
        let fused = found_entry(cond, then);
        // A profiled build counts the step's own line, so it keeps the branch.
        if fused.is_none()
            && otherwise.stmts.is_empty()
            && lead.trim().is_empty()
            && self.sites.is_none()
        {
            if let [Stmt::Assign(place, op, value)] = then.stmts.as_slice() {
                if let Some(sym) = self.counted_global(place, *op, value) {
                    // `if c: hits += 1` on a global: a conditional store
                    // keeps `hits` in memory and the branch in the loop;
                    // an unconditional one lets g++ keep it in a register
                    // and add the condition without a branch.
                    let (c, v, p) = (
                        self.expr(cond).text,
                        self.expr(value).text,
                        self.place(place),
                    );
                    out.push_str(&format!("{lead}{p} {sym}= ({c}) ? {v} : 0;\n"));
                    return;
                }
            }
        }
        if let Some((map, key)) = fused {
            let (m, k, it) = (self.expr(map).text, self.expr(key).text, self.temp());
            out.push_str(&format!(
                "{lead}if (auto {it} = {m}.find({k}); {it} != {m}.end()) {{\n"
            ));
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

    pub fn stmt(&mut self, stmt: &Stmt, out: &mut String) {
        self.note_site(stmt);
        if let (Some(sites), Some(span)) = (&mut self.sites, stmt.span()) {
            if self.counted.contains(&(stmt as *const Stmt)) {
                sites.push(span);
                let id = sites.len() - 1;
                if self.entries.contains(&(stmt as *const Stmt)) {
                    self.line(out, "cpl_prof::Frame cpl_prof_frame;");
                }
                self.line(out, &format!("cpl_prof::mark({id});"));
            }
        }
        let program = self.program;
        match stmt {
            Stmt::Let(slot, _)
                if self.closure_id.is_none() && self.devirt.dead_local(self.fn_id, *slot) => {}
            Stmt::Let(slot, Some(init))
                if self.closure_id.is_none() && self.devirt.unread_let(self.fn_id, *slot) =>
            {
                self.expr_stmt(init, out)
            }
            Stmt::Assign(place, op, value) if self.dropped_store(place, *op) => {
                self.dropped_value(value, out)
            }
            Stmt::Expr(e)
                if self.closure_id.is_none()
                    && self
                        .devirt
                        .skippable_push(e)
                        .is_some_and(|slot| self.devirt.dead_local(self.fn_id, slot)) => {}
            // A fn written as `cpl_each_f`: its list is the callback.
            Stmt::Let(slot, _) if self.pushing == Some(*slot) && self.closure_id.is_none() => {}
            Stmt::Expr(Expr {
                kind: ExprKind::Mutate(how, place, items),
                ..
            }) if self.pushing.is_some()
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
                    _ => self.line(
                        out,
                        &format!("if (int cpl_r = cpl_f({text})) return cpl_r;"),
                    ),
                }
            }
            Stmt::Return(Some(_)) if self.pushing.is_some() && self.closure_id.is_none() => {
                self.line(out, "return 0;")
            }
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
            Stmt::Break(to)
                if self
                    .bodies
                    .last()
                    .is_some_and(|b| to.map_or(b.depth == 0, |to| to == b.id)) =>
            {
                self.line(out, "return 1;")
            }
            Stmt::Continue(to)
                if self
                    .bodies
                    .last()
                    .is_some_and(|b| to.map_or(b.depth == 0, |to| to == b.id)) =>
            {
                self.line(out, "return 0;")
            }
            Stmt::Let(slot, Some(init))
                if matches!(self.locals()[*slot].ty, Ty::Fn(..))
                    && self.closure_id.is_none()
                    && !self.hoisted.contains(slot)
                    && matches!(init.kind, ExprKind::Closure(cid) if self.devirt.local_rec(*slot, cid))
                    && self.facts.stores(self.facts_local(*slot)) <= 1 =>
            {
                let ExprKind::Closure(cid) = init.kind else {
                    unreachable!()
                };
                self.borrow_if_named_closure(*slot, Some(init));
                self.need("rec");
                self.rec_slot = Some(*slot);
                let text = self.emit_closure(cid, false).text;
                let name = self.names[*slot].clone();
                self.line(out, &format!("auto {name} = cpl::rec({text});"));
            }
            Stmt::Let(slot, init) => {
                self.borrow_if_named_closure(*slot, init.as_ref());
                let tail = self.tail.take();
                let ty = self.locals()[*slot].ty.clone();
                let name = self.names[*slot].clone();
                let loc = cpl_ir::ElemLoc::Local(self.current_scope(), *slot);
                let fixed_len = self.fixed_array_len(loc);
                let elem_l = self.elem_layout(loc).cloned();
                let cpp = self.ty_for_loc(&ty, loc);
                let hoisted = self.hoisted.contains(slot);
                let Some(init) = init else {
                    if ty != Ty::Unit {
                        if hoisted {
                            self.line(out, &format!("{name} = {cpp}();"));
                        } else {
                            let z = if fixed_len.is_some() {
                                "{}"
                            } else {
                                self.zero_init(&ty)
                            };
                            self.line(out, &format!("{cpp} {name}{z};"));
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
                if reused && matches!(&init.kind, ExprKind::Tuple(items) if items.is_empty()) {
                    self.line(out, &format!("static {cpp} {name};"));
                    self.line(out, &format!("{name}.clear();"));
                    return;
                }
                if !hoisted && fixed_len.is_none() && elem_l.is_none() && !name.starts_with('_') {
                    let at = (cpp.as_str(), name.as_str());
                    if self.let_lending_branch(*slot, init, at, tail.as_ref(), out) {
                        return;
                    }
                    if self.let_lending_coalesce(*slot, init, at, tail.as_ref(), out) {
                        return;
                    }
                    if self.let_copying_lent(&name, init, out) {
                        return;
                    }
                    if let Some([map, key]) = self.lent_map_get(*slot, init, tail.as_ref()) {
                        // `if let l = m.get(k)`: point at the entry, never copy it.
                        if !self.locals()[*slot].name.is_empty() {
                            self.lent.insert(*slot);
                        }
                        self.need("map_lend");
                        let (m, k) = (self.expr(map).text, self.expr(key).text);
                        self.line(
                            out,
                            &format!("const auto {name} = cpl::map_lend({m}, {k});"),
                        );
                        return;
                    }
                }
                let whole = init;
                let init = self.flatten(init, out);
                match &init.kind {
                    ExprKind::Read(spec) => {
                        if let Spec::Array { dims, elem } = &**spec {
                            self.read_array_with_layout(
                                &name,
                                !hoisted,
                                dims,
                                elem,
                                elem_l.as_ref(),
                                out,
                            );
                            return;
                        }
                    }
                    ExprKind::Fill(dims, value) if !hoisted && dims.len() == 1 => {
                        if fixed_len.is_some() {
                            self.line(out, &format!("{cpp} {name}{{}};"));
                            if let Some(v) = value {
                                let val = self.expr(v).text;
                                self.line(out, &format!("{name}.fill({val});"));
                            }
                            return;
                        }
                        let args =
                            self.fill_args_with_layout(dims, value.as_deref(), elem_l.as_ref());
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
                let hold = if hoisted { Hold::Static } else { Hold::Block };
                let pinned = self.pinned_view(init, hold, out);
                let text = match &init.kind {
                    _ if pinned.is_some() => pinned.unwrap_or_default(),
                    ExprKind::Field(base, index) => match &base.kind {
                        ExprKind::Local(of) if self.spent.contains(of) => {
                            format!("std::get<{index}>(std::move({}))", self.names[*of])
                        }
                        _ => {
                            if fixed_len.is_some() {
                                self.expr_with_fixed_len(init, fixed_len).text
                            } else {
                                self.expr_with_elem_layout(init, elem_l.as_ref()).text
                            }
                        }
                    },
                    _ => {
                        if fixed_len.is_some() {
                            self.expr_with_fixed_len(init, fixed_len).text
                        } else {
                            self.expr_with_elem_layout(init, elem_l.as_ref()).text
                        }
                    }
                };
                let text = self.take_if_overwritten(init, text);
                // A kept comprehension's list stays in its buffer: name it.
                if let ExprKind::Local(built) = init.kind {
                    if reused && self.reused.contains(&built) && built_in(whole, built) {
                        self.line(out, &format!("auto& {name} = {};", self.names[built]));
                        return;
                    }
                }
                // A value its own block built (a comprehension) is not read again: it moves.
                let text = match init.kind {
                    ExprKind::Local(built)
                        if text == self.names[built]
                            && !self.ref_bound.contains(&built)
                            && !init.ty.is_scalar()
                            && !matches!(init.ty, Ty::Fn(..))
                            && built_in(whole, built) =>
                    {
                        format!("std::move({text})")
                    }
                    _ => text,
                };
                let stores = self.facts.stores(self.facts_local(*slot));
                let auto = matches!(ty, Ty::Fn(..))
                    && matches!(init.kind, ExprKind::Closure(cid) if self.devirt.let_auto(self.closure_id, *slot, cid, stores));
                // Slots are per fn: one another fn lent says nothing of this one.
                self.ref_bound.remove(slot);
                if hoisted {
                    self.line(out, &format!("{name} = {text};"));
                } else if name.starts_with('_') {
                    // `_t` is a name for a value the program may not use.
                    let cpp = if auto { "auto" } else { &cpp };
                    self.line(out, &format!("[[maybe_unused]] {cpp} {name} = {text};"));
                } else if self.lends(*slot, init, tail.as_ref()) {
                    // `let sp = sorted[b]`: a name for what is already
                    // there. A copy would cost the whole list.
                    self.line(out, &format!("const {cpp}& {name} = {text};"));
                    self.ref_bound.insert(*slot);
                } else if let Some(args) = sized_ctor(&cpp, &text) {
                    // `std::vector<ll> f((size_t)(n), 1);`: the type once.
                    self.line(out, &format!("{cpp} {name}({args});"));
                } else if auto {
                    self.line(out, &format!("auto {name} = {text};"));
                } else {
                    self.note_let_copy(*slot, init, &text, tail.as_ref());
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
            Stmt::For(id, slot, iter, body) => self.checked_for(*id, *slot, iter, body, out),
            Stmt::Return(None) if self.in_setup => self.line(out, "return false;"),
            Stmt::Return(None) => self.line(out, "return;"),
            Stmt::Return(Some(value)) | Stmt::Give(value) => {
                let value = self.flatten(value, out);
                if value.ty == Ty::Error {
                    self.expr_stmt(value, out);
                } else if value.ty == Ty::Unit {
                    self.expr_stmt(value, out);
                    self.line(out, "return;");
                } else {
                    let is_return = matches!(stmt, Stmt::Return(_));
                    self.last_use_call = is_return
                        && self.closure_id.is_none()
                        && self.fn_id.is_some()
                        && matches!(value.kind, ExprKind::Call(..));
                    let rekeyed = (is_return && self.closure_id.is_none())
                        .then(|| self.rekeyed_local(value))
                        .flatten();
                    let mut text = if is_return && self.closure_id.is_none() {
                        if let ExprKind::Tuple(items) = &value.kind {
                            if !items.is_empty()
                                && !matches!(
                                    value.ty,
                                    Ty::Unit | Ty::Error | Ty::Map(..) | Ty::HashMap(..)
                                )
                            {
                                let params =
                                    self.fn_id.map_or(0, |id| self.program.fns[id].params.len());
                                let ty = self.ty(&value.ty);
                                let struct_id = match value.ty {
                                    Ty::Struct(s) => Some(s),
                                    _ => None,
                                };
                                let dropped: Vec<bool> = (0..items.len())
                                    .map(|idx| {
                                        struct_id.is_some_and(|s| self.devirt.unread_field(s, idx))
                                    })
                                    .collect();
                                let rendered: Vec<String> = items
                                    .iter()
                                    .enumerate()
                                    .filter(|(idx, _)| !dropped[*idx])
                                    .map(|(idx, item)| {
                                        if struct_id.is_some_and(|s| {
                                            self.devirt.field_elidable(s, idx)
                                                && self.devirt.skippable_value(item)
                                        }) {
                                            return "false".to_string();
                                        }
                                        if let Some(sid) = struct_id {
                                            if let Some(&k) = self.rows.get(&(sid, idx)) {
                                                let cell = self
                                                    .ty(&item.ty)
                                                    .strip_prefix("std::vector<")
                                                    .and_then(|c| c.strip_suffix('>'))
                                                    .map(str::to_string);
                                                if let Some(cell) = cell {
                                                    if let Some(row_init) =
                                                        self.emit_row_init(item, sid, idx, &cell, k)
                                                    {
                                                        return row_init;
                                                    }
                                                }
                                            }
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
                                        if let Some(l) = struct_id
                                            .and_then(|s| {
                                                self.elem_layout(cpl_ir::ElemLoc::Field(s, idx))
                                            })
                                            .cloned()
                                        {
                                            return self.expr_with_elem_layout(item, Some(&l)).text;
                                        }
                                        let v = self.expr(item).text;
                                        match struct_id {
                                            Some(s) => self.narrow_field_store(s, idx, &v),
                                            None => v,
                                        }
                                    })
                                    .collect();
                                format!("{ty}{{{}}}", rendered.join(", "))
                            } else {
                                let ret_l = if is_return {
                                    self.fn_id
                                        .and_then(|id| self.elem_layout(cpl_ir::ElemLoc::FnRet(id)))
                                        .cloned()
                                } else {
                                    None
                                };
                                self.expr_with_elem_layout(value, ret_l.as_ref()).text
                            }
                        } else {
                            let ret_l = if is_return {
                                self.fn_id
                                    .and_then(|id| self.elem_layout(cpl_ir::ElemLoc::FnRet(id)))
                                    .cloned()
                            } else {
                                None
                            };
                            self.expr_with_elem_layout(value, ret_l.as_ref()).text
                        }
                    } else {
                        let ret_l = if is_return {
                            self.fn_id
                                .and_then(|id| self.elem_layout(cpl_ir::ElemLoc::FnRet(id)))
                                .cloned()
                        } else {
                            None
                        };
                        self.expr_with_elem_layout(value, ret_l.as_ref()).text
                    };
                    if let Some(local) = rekeyed {
                        let name = &self.names[local];
                        if let Some(rest) = text.strip_prefix(&format!("cpl::rekey_own({name}, ")) {
                            text = format!("cpl::rekey_own(std::move({name}), {rest}");
                        }
                    }
                    self.last_use_call = false;
                    if self.give_ref && matches!(stmt, Stmt::Give(_)) {
                        text = format!("({text})");
                    }
                    self.line(out, &format!("return {text};"));
                }
            }
            Stmt::Break(None) => self.line(out, "break;"),
            Stmt::Break(Some(id)) => self.line(out, &format!("goto brk_{id};")),
            Stmt::Continue(None) => self.line(out, "continue;"),
            Stmt::Continue(Some(id)) => self.line(out, &format!("goto cont_{id};")),
            Stmt::Assert { kept: false, .. } if self.target.checks == crate::Checks::None => {}
            Stmt::Assert {
                cond,
                message,
                text,
                line,
                note,
                ..
            } => self.assert(out, cond, message.as_ref(), text, *line, note.as_deref()),
            Stmt::DefDomain { id, lo, len } => {
                let d = dom_ident(program, *id);
                let lo_const = self.facts.const_domain_lo(*id).is_some();
                let lo = if lo_const {
                    String::new()
                } else {
                    self.expr(lo).text
                };
                let len = self.expr(len).text;
                if !lo_const {
                    self.line(out, &format!("{d}_lo = {lo};"));
                }
                self.line(out, &format!("{d}_len = std::max<ll>({len}, 0);"));
                self.domain_fits(out, *id, &d);
            }
            Stmt::DefDomainFromVal { id, val } => {
                let d = dom_ident(program, *id);
                let v = self.expr(val).text;
                let tmp = self.temp();
                self.line(out, &format!("auto {tmp} = {v};"));
                self.line(out, &format!("{d}_lo = {tmp}.lo;"));
                self.line(out, &format!("{d}_len = std::max<ll>({tmp}.len, 0);"));
                self.domain_fits(out, *id, &d);
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
        _ => {}
    });
    alias || Touch::of_expr(e).calls_code()
}

/// The variable a place reads from, looking through the unchecked unwrap
/// that follows an `if let` test. Its indices run no code that can write
/// anything, so reading them again gives the same cell.
pub(crate) fn place_root(e: &Expr) -> Option<(bool, usize)> {
    match &e.kind {
        ExprKind::Unwrap(base, _) | ExprKind::Field(base, _) | ExprKind::Payload(base, ..) => {
            place_root(base)
        }
        ExprKind::Index(base, indices) if indices.iter().all(writes_nothing) => place_root(base),
        _ => walk::path_root(e),
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
    let (mut reads, mut quiet) = (false, Touch::of_block(then).quiet());
    walk::deep_exprs(then, &mut |e| match &e.kind {
        ExprKind::Index(base, at)
            if base.kind == map.kind && at.len() == 1 && at[0].kind == key.kind =>
        {
            reads = true
        }
        _ => {}
    });
    walk::deep_stmts(then, &mut |stmt| {
        if !matches!(
            stmt,
            Stmt::Expr(_) | Stmt::Return(_) | Stmt::Give(_) | Stmt::If(..) | Stmt::Let(..)
        ) {
            quiet = false;
        }
    });
    (reads && quiet).then_some((map, key))
}

/// The arguments of `text` when it is exactly `cpp((size_t)(..), ..)`:
/// a container of the declared type built with a size. The size cast
/// first means the arguments can't be read as a declaration.
fn sized_ctor<'t>(cpp: &str, text: &'t str) -> Option<&'t str> {
    let args = text
        .strip_prefix(cpp)?
        .strip_prefix('(')?
        .strip_suffix(')')?;
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

/// The fields `e` names below a local: `self.w` is `(self, [w])`.
fn local_fields(e: &Expr) -> Option<(usize, Vec<usize>)> {
    match &e.kind {
        ExprKind::Local(slot) => Some((*slot, Vec::new())),
        ExprKind::Field(base, field) => {
            let (slot, mut path) = local_fields(base)?;
            path.push(*field);
            Some((slot, path))
        }
        _ => None,
    }
}

/// The root of a stored place and the fields above its first index:
/// a store to `self.w[k]` changes all of `self.w`.
fn stored_fields(place: &Place) -> (bool, usize, Vec<usize>, bool) {
    match &place.kind {
        PlaceKind::Local(slot) => (false, *slot, Vec::new(), false),
        PlaceKind::Global(id) => (true, *id, Vec::new(), false),
        PlaceKind::Field(base, field) => {
            let (global, root, mut path, cut) = stored_fields(base);
            if !cut {
                path.push(*field);
            }
            (global, root, path, cut)
        }
        PlaceKind::Index(base, _) | PlaceKind::Some(base) => {
            let (global, root, path, _) = stored_fields(base);
            (global, root, path, true)
        }
    }
}

/// True when `seq` is a field path into a local and every store the body
/// makes to that local is to a field beside it, never to it, inside it or
/// above it: `for x in w: n += popcount(x)` walks `w` in place.
fn fields_untouched(seq: &Expr, body: &Block) -> bool {
    let Some((slot, walked)) = local_fields(seq) else {
        return false;
    };
    if walked.is_empty() {
        return false;
    }
    let mut stores: Vec<&Place> = Vec::new();
    walk::deep_stmts(body, &mut |stmt| {
        if let Stmt::Assign(place, ..) = stmt {
            stores.push(place);
        }
    });
    walk::deep_exprs(body, &mut |e| match &e.kind {
        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
            stores.extend(args.iter().filter_map(|a| match a {
                cpl_ir::Arg::Mut(place) => Some(place),
                cpl_ir::Arg::Value(_) => None,
            }));
        }
        ExprKind::MutSlice(place, _, _)
        | ExprKind::Mutate(_, place, _)
        | ExprKind::Sort(place, _, _) => stores.push(place),
        _ => {}
    });
    stores.iter().all(|place| {
        let (global, root, path, _) = stored_fields(place);
        global || root != slot || (!path.starts_with(&walked) && !walked.starts_with(&path))
    })
}

/// Whether one of the value blocks around `e` declares `slot`.
fn built_in(mut e: &Expr, slot: usize) -> bool {
    while let Some((stmts, value)) = flat(e) {
        if stmts
            .iter()
            .any(|stmt| matches!(stmt, Stmt::Let(s, _) if *s == slot))
        {
            return true;
        }
        e = value;
    }
    false
}
