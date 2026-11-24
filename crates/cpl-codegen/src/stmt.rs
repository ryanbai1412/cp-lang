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

mod ret;
pub(crate) use ret::block_lets;

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
            if self.cold_suffix(block, k, out) {
                break;
            }
            self.tail = self.tail_at(&block.stmts, k);
            self.slot_written_next = self.slot_written_next(&block.stmts, k);
            self.stmt_owning(block, k, out);
            self.slot_written_next = false;
        }
        self.tail = None;
    }

    /// `let p = mut m[k]` right before `p += v` (or `-=`, `|=`, `^=`): that write puts an absent
    /// key at its zero anyway, so the `let` can put it and walk the map once. `v` must not read the
    /// map, which would see the key early.
    fn slot_written_next(&self, stmts: &[Stmt], k: usize) -> bool {
        let (Stmt::Let(slot, Some(_)), Some(Stmt::Assign(place, AssignOp::Binary(op), value))) =
            (&stmts[k], stmts.get(k + 1))
        else {
            return false;
        };
        let PlaceKind::Index(base, idx) = &place.kind else {
            return false;
        };
        self.locals()[*slot].binds.is_some()
            && matches!(op, BinOp::Add | BinOp::Sub | BinOp::BitOr | BinOp::BitXor)
            && matches!(idx[0].kind, ExprKind::Local(s) if s == *slot)
            && !expr_may_alias_root(value, walk::root(base))
    }

    /// `block.stmts[k..]` as a noinline lambda the block returns through,
    /// when it is cold (see [`crate::frame`]): its locals leave the
    /// recursive frame.
    fn cold_suffix(&mut self, block: &Block, k: usize, out: &mut String) -> bool {
        let Some(inputs) = self.cold.get(&(block as *const Block, k)).cloned() else {
            return false;
        };
        if self.pushing.is_some()
            || !self.bodies.is_empty()
            || self.closure_id.is_some()
            || self.sites.is_some()
        {
            return false;
        }
        let params: Vec<String> = inputs
            .iter()
            .map(|&s| format!("auto&& {}", self.names[s]))
            .collect();
        let args: Vec<String> = inputs.iter().map(|&s| self.names[s].clone()).collect();
        let head = format!(
            "return [&]({}) __attribute__((noinline)) -> {} {{",
            params.join(", "),
            self.cold_ret
        );
        self.line(out, &head);
        self.indent += 1;
        for j in k..block.stmts.len() {
            self.tail = self.tail_at(&block.stmts, j);
            self.stmt_owning(block, j, out);
        }
        self.tail = None;
        self.indent -= 1;
        self.line(out, &format!("}}({});", args.join(", ")));
        true
    }

    /// `let l = limbs` in a fn's body, where `limbs` is a parameter taken by
    /// value (see [`crate::scope::sunk_params`]) that nothing reads after.
    pub(crate) fn takes_sunk_param(&self, block: &Block, k: usize) -> bool {
        let (Stmt::Let(_, Some(init)), Some(id)) = (&block.stmts[k], self.fn_id) else {
            return false;
        };
        let src = crate::uses::copied(init).unwrap_or(crate::uses::in_place(init));
        let ExprKind::Local(p) = src.kind else {
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
            self.take_block_lets(stmts, value);
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
                if matches!(otherwise.kind, ExprKind::Unit) && self.fills_cache_call(then) {
                    self.line(out, &format!("if (__builtin_expect({cond}, 0)) {{"));
                } else {
                    self.line(out, &format!("if ({cond}) {{"));
                }
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
            ExprKind::Local(_)
            | ExprKind::Global(_)
            | ExprKind::Index(..)
            | ExprKind::Unary(..)
            | ExprKind::Binary(..)
            | ExprKind::Cast(..) => {
                let text = self.expr(e).text;
                self.line(out, &format!("(void){text};"));
            }
            _ => {
                let text = self.expr(e).text;
                self.line(out, &format!("{text};"));
            }
        }
    }

    /// `<true>` when the elements' order says two values are equal only
    /// when they are the same value, so `std::sort` gives the stable sort's
    /// output; a profile still counts the interpreter's comparisons.
    pub(crate) fn same_sort(&self, elem: Option<&Ty>) -> &'static str {
        let same = self.sites.is_none() && elem.is_some_and(|t| identity_order(self.program, t, 0));
        if same {
            "<true>"
        } else {
            ""
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
                let elem = match &place.ty {
                    Ty::List(t) | Ty::MutView(t, _) | Ty::Array { elem: t, .. } => Some(&**t),
                    _ => None,
                };
                let same = self.same_sort(elem);
                let lt = match &desc {
                    None => String::new(),
                    Some(desc) => {
                        let (a, b) = (self.temp(), self.temp());
                        format!(", [&](const auto& {a}, const auto& {b}) {{ return {desc} ? {b} < {a} : {a} < {b}; }}")
                    }
                };
                self.line(
                    out,
                    &format!("cpl::sort_seq{same}({seq}.begin(), {seq}.end(){lt});"),
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
                    // A key that ends in the element makes equal keys equal
                    // elements, so no order is told apart; a profile still
                    // counts the interpreter's comparisons.
                    let ends_in_elem =
                        |e: &Expr| matches!(e.kind, ExprKind::Local(s) if s == *slot);
                    let unique = elem.is_scalar()
                        && match &key.kind {
                            ExprKind::Tuple(items) => items.last().is_some_and(ends_in_elem),
                            _ => ends_in_elem(key),
                        };
                    let sort = if unique && self.sites.is_none() {
                        "std::sort"
                    } else {
                        "cpl::stable_sort_lt"
                    };
                    self.line(
                        out,
                        &format!("{sort}({seq}.begin(), {seq}.end(), [&](const auto& {a}, const auto& {b}) {{ return {cmp}; }});"),
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
            // A map named by a path, not a variable, is found through once.
            let cell = if matches!(map.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
                out.push_str(&format!(
                    "{lead}if (auto {it} = {m}.find({k}); {it} != {m}.end()) {{\n"
                ));
                format!("{it}->second")
            } else {
                self.need("map_slot");
                out.push_str(&format!(
                    "{lead}if (auto* {it} = cpl::map_slot({m}, {k})) {{\n"
                ));
                format!("(*{it})")
            };
            self.found.push((map.clone(), key.clone(), cell));
        } else if otherwise.stmts.is_empty() && self.fills_cache(then) {
            // A cache miss is the rare way in: g++ lays the fill out of line,
            // so the hit path of a query in a loop stays a compare and a load.
            let c = self.expr(cond).text;
            out.push_str(&format!("{lead}if (__builtin_expect({c}, 0)) {{\n"));
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

    /// `then` is one call to a unit fn that takes a read-only `self` to grow
    /// its `@cache` fields: all it can change is the cache.
    fn fills_cache(&self, then: &Block) -> bool {
        match then.stmts.as_slice() {
            [Stmt::Expr(e)] => self.fills_cache_call(e),
            _ => false,
        }
    }

    fn fills_cache_call(&self, e: &Expr) -> bool {
        let ExprKind::Call(id, _) = &e.kind else {
            return false;
        };
        let callee = &self.program.fns[*id];
        callee.ret == Ty::Unit && callee.params.first().is_some_and(|p| p.fills_cache)
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
                if !self.trap_free(init) {
                    self.expr_stmt(init, out)
                }
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
                self.line(out, "return {};")
            }
            Stmt::Let(slot, None)
                if self.closure_id.is_none() && self.memo_pairs.contains_key(slot) => {}
            Stmt::Let(slot, Some(init))
                if self.closure_id.is_none()
                    && self.memo_pairs.values().any(|(b, ..)| b == slot)
                    && matches!(init.kind, ExprKind::Closure(_)) =>
            {
                let ExprKind::Closure(cid) = init.kind else {
                    return;
                };
                self.borrow_if_named_closure(*slot, Some(init));
                let back = self
                    .memo_pairs
                    .iter()
                    .find(|(_, (b, rec, _))| b == slot && *rec);
                self.rec_slot = back.map(|(w, _)| *w);
                let later = back.is_some_and(|(_, (.., later))| *later);
                let text = self.emit_closure(cid, false).text;
                let name = self.names[*slot].clone();
                let name = if later { format!("{name}_body") } else { name };
                self.line(out, &format!("auto {name} = {text};"));
            }
            Stmt::Assign(place, AssignOp::Set, value)
                if self.closure_id.is_none()
                    && matches!(place.kind, PlaceKind::Local(w) if self.memo_pairs.contains_key(&w))
                    && matches!(value.kind, ExprKind::Closure(_)) =>
            {
                let (PlaceKind::Local(w), ExprKind::Closure(cid)) = (&place.kind, &value.kind)
                else {
                    return;
                };
                let (body, rec, later) = self.memo_pairs[w];
                let name = self.names[*w].clone();
                if !rec {
                    let text = self.emit_closure(*cid, false).text;
                    self.line(out, &format!("auto {name} = {text};"));
                    return;
                }
                self.need("memo");
                self.rec_slot = Some(body);
                let text = self.emit_closure(*cid, false).text;
                let body = self.names[body].clone();
                if later {
                    self.line(
                        out,
                        &format!("auto {name} = cpl::memo(std::move({body}_body), {text});"),
                    );
                    self.line(out, &format!("auto {body} = {name}.body();"));
                } else {
                    self.line(
                        out,
                        &format!("auto {name} = cpl::memo(std::move({body}), {text});"),
                    );
                }
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
            Stmt::Let(slot, Some(init))
                if self.locals()[*slot].binds.is_some()
                    && !self.hoisted.contains(slot)
                    && !self.plain_let =>
            {
                if let Some(map) = &self.locals()[*slot].binds {
                    self.let_map_slot(*slot, map, init, out);
                }
            }
            Stmt::Let(slot, init) => {
                self.borrow_if_named_closure(*slot, init.as_ref());
                let tail = self.tail.take();
                let ty = self.locals()[*slot].ty.clone();
                let name = self.names[*slot].clone();
                let loc = cpl_ir::ElemLoc::Local(self.current_scope(), *slot);
                let fixed_len = self.fixed_array_len(loc).or_else(|| self.ty_fixed_len(&ty));
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
                                let val = self.fill_value(v, elem_l.as_ref());
                                self.line(out, &format!("{name}.fill({val});"));
                            }
                            return;
                        }
                        let cell = cpp
                            .strip_prefix("std::vector<")
                            .and_then(|c| c.strip_suffix('>'))
                            .unwrap_or_default()
                            .to_string();
                        if let Some(zeros) = (!reused)
                            .then(|| self.zeros(&cell, dims, value.as_deref()))
                            .flatten()
                        {
                            self.line(out, &format!("{cpp} {name} = {};", zeros.text));
                            return;
                        }
                        let args =
                            self.fill_args_with_layout(dims, value.as_deref(), elem_l.as_ref());
                        if reused {
                            // One buffer, filled again each time round: it keeps its room,
                            // but cells that own storage are made anew (`clear` first), so no
                            // cell keeps more than a fresh one would.
                            let nested = matches!(&ty, Ty::List(elem) | Ty::Array { elem, .. }
                                if crate::lend::holds_heap(self.program, elem));
                            self.line(out, &format!("static {cpp} {name};"));
                            self.line(
                                out,
                                &match value {
                                    Some(_) if nested => {
                                        format!("{name}.clear(); {name}.assign({args});")
                                    }
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
                let text = match crate::uses::copied(init) {
                    Some(src) if self.moves_last && pinned.is_none() => self.expr(src).text,
                    _ => match &init.kind {
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
                    },
                };
                let text = self.take_if_overwritten(init, text);
                // A payload taken from an optional nothing reads again is owned:
                // a reference into the optional could not be moved on. So is a
                // named source taken here: the name owns it, so later stores move it on.
                let taken = text.starts_with("std::move(")
                    && match init.kind {
                        ExprKind::Unwrap(..) | ExprKind::Global(_) => true,
                        ExprKind::Local(src) => !self.locals()[src].name.is_empty(),
                        _ => false,
                    };
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
                    && match init.kind {
                        ExprKind::Closure(cid) => {
                            self.devirt.let_auto(self.closure_id, *slot, cid, stores)
                        }
                        ExprKind::Call(f, _) => {
                            !hoisted && self.devirt.let_auto_call(self.closure_id, *slot, f, stores)
                        }
                        _ => {
                            !hoisted
                                && self
                                    .devirt
                                    .let_auto_join(self.closure_id, *slot, init, stores)
                        }
                    };
                // A name for a fn parameter that is a template is that same functor.
                if matches!(ty, Ty::Fn(..))
                    && !hoisted
                    && stores <= 1
                    && self.closure_id.is_none()
                    && matches!(init.kind, ExprKind::Local(src)
                        if self.fn_id.is_some_and(|id| self.devirt.template_params(id).contains(&src)))
                {
                    self.ref_bound.remove(slot);
                    self.line(out, &format!("auto& {name} = {text};"));
                    return;
                }
                // Slots are per fn: one another fn lent says nothing of this one.
                self.ref_bound.remove(slot);
                self.mut_bound.remove(slot);
                if hoisted {
                    self.line(out, &format!("{name} = {text};"));
                } else if name.starts_with('_') {
                    // `_t` is a name for a value the program may not use.
                    let cpp = if auto { "auto" } else { &cpp };
                    self.line(out, &format!("[[maybe_unused]] {cpp} {name} = {text};"));
                } else if let Some(call) = (!taken)
                    .then(|| self.held_call(*slot, init, tail.as_ref(), &text))
                    .flatten()
                {
                    // `let s = t.fold(r)` of a fn that returns `cpl::Held<T>`: read in place.
                    self.line(out, &format!("const cpl::Held<{cpp}> {name}_h = {call};"));
                    self.line(out, &format!("const {cpp}& {name} = *{name}_h;"));
                    self.ref_bound.insert(*slot);
                } else if !taken && self.lends(*slot, init, tail.as_ref()) {
                    // `let sp = sorted[b]`: a name for what is already
                    // there. A copy would cost the whole list. One a return
                    // gives a part of, on a local the fn owns, may move it out.
                    let owner = crate::lend::bound_root(init).filter(|&src| {
                        self.closure_id.is_none()
                            && self
                                .fn_id
                                .is_some_and(|id| src >= self.program.fns[id].params.len())
                            && !self.ref_bound.contains(&src)
                    });
                    if owner.is_some() && self.given_parts.contains(slot) {
                        self.line(out, &format!("{cpp}& {name} = {text};"));
                        self.mut_bound.insert(*slot);
                    } else {
                        self.line(out, &format!("const {cpp}& {name} = {text};"));
                    }
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
            Stmt::Give(value)
                if self
                    .give_index
                    .contains_key(&(value as *const Expr as usize)) =>
            {
                let k = self.give_index[&(value as *const Expr as usize)];
                self.line(out, &format!("return {k};"));
            }
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
                    let moved_some = is_return.then(|| self.moved_some(value));
                    let handed = (is_return && self.closure_id.is_none())
                        || (!is_return && !self.give_ref && self.give_owned.is_some());
                    let mut text = if let Some(Some(text)) = moved_some {
                        text
                    } else if let Some(text) =
                        handed.then(|| self.handed_out(value, is_return)).flatten()
                    {
                        text
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
                    if is_return
                        && self.closure_id.is_none()
                        && matches!(value.kind, ExprKind::Field(..) | ExprKind::Index(..))
                        && self.returned_item_dies(std::slice::from_ref(value), 0)
                    {
                        text = self.moved_if(true, text);
                    }
                    if let Some(local) = rekeyed {
                        let name = &self.names[local];
                        if let Some(rest) = text.strip_prefix(&format!("cpl::rekey_own({name}, ")) {
                            text = format!("cpl::rekey_own(std::move({name}), {rest}");
                        }
                    }
                    self.last_use_call = false;
                    let held = (is_return && self.closure_id.is_none())
                        .then(|| self.holds_result.clone())
                        .flatten();
                    if let Some((root, by_ref)) = held {
                        let place = !text.starts_with("std::move(")
                            && self.lent_place_root(value, &by_ref) == Some(root);
                        if !place {
                            self.note_given_copy(value, &text, is_return);
                        }
                        let how = if place { "ref" } else { "own" };
                        text = format!("cpl::held_{how}({text})");
                    } else {
                        self.note_given_copy(value, &text, is_return);
                    }
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
                if !lo_const {
                    self.line(out, &format!("{d}_lo = {lo};"));
                }
                let len_text = self.expr(len).text;
                if self.facts.constexpr_domain_len(*id).is_none() {
                    self.line(out, &format!("{d}_len = std::max<ll>({len_text}, 0);"));
                    self.domain_fits(out, *id, &d);
                } else if reads_a_local(len) {
                    // The locals it was folded from are read, though its count is a constant.
                    self.line(out, &format!("(void)({len_text});"));
                }
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
    if !matches!(map.ty, Ty::Map(..) | Ty::HashMap(..)) || !steady(map) || !steady(key) {
        return None;
    }
    // Stores to the entry itself, at the top of the branch, keep it where it is.
    let entry = |stmt: &Stmt| {
        matches!(stmt, Stmt::Assign(place, _, value)
            if Touch::of_expr(value).quiet()
                && matches!(&place.kind, cpl_ir::PlaceKind::Index(base, at)
                    if at.len() == 1 && place_is(base, map) && same_path(&at[0], key)))
    };
    let rest = Block {
        stmts: then.stmts.iter().filter(|s| !entry(s)).cloned().collect(),
    };
    let (mut uses, mut quiet) = (
        rest.stmts.len() < then.stmts.len(),
        Touch::of_block(&rest).quiet(),
    );
    walk::deep_exprs(then, &mut |e| match &e.kind {
        ExprKind::Index(base, at)
            if at.len() == 1 && same_path(base, map) && same_path(&at[0], key) =>
        {
            uses = true
        }
        _ => {}
    });
    walk::deep_stmts(&rest, &mut |stmt| {
        if !matches!(
            stmt,
            Stmt::Expr(_) | Stmt::Return(_) | Stmt::Give(_) | Stmt::If(..) | Stmt::Let(..)
        ) {
            quiet = false;
        }
    });
    (uses && quiet).then_some((map, key))
}

/// A path that runs no code: a variable, or fields and non-map elements of one.
fn steady(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(_)
        | ExprKind::Bool(_)
        | ExprKind::Char(_)
        | ExprKind::DomainLen(_)
        | ExprKind::Local(_)
        | ExprKind::Global(_) => true,
        ExprKind::Field(base, _) | ExprKind::Cast(_, base) => steady(base),
        ExprKind::Binary(_, a, b) => steady(a) && steady(b),
        ExprKind::Index(base, at) => {
            !matches!(base.ty, Ty::Map(..) | Ty::HashMap(..))
                && steady(base)
                && at.iter().all(steady)
        }
        _ => false,
    }
}

/// Whether two [`steady`] paths name the same value, wherever they are written.
pub(crate) fn same_path(a: &Expr, b: &Expr) -> bool {
    match (&a.kind, &b.kind) {
        (ExprKind::Int(x), ExprKind::Int(y)) => x == y,
        (ExprKind::Bool(x), ExprKind::Bool(y)) => x == y,
        (ExprKind::Char(x), ExprKind::Char(y)) => x == y,
        (ExprKind::DomainLen(x), ExprKind::DomainLen(y)) => x == y,
        (ExprKind::Local(x), ExprKind::Local(y)) => x == y,
        (ExprKind::Global(x), ExprKind::Global(y)) => x == y,
        (ExprKind::Field(x, i), ExprKind::Field(y, j)) => i == j && same_path(x, y),
        (ExprKind::Cast(c, x), ExprKind::Cast(d, y)) => c == d && same_path(x, y),
        (ExprKind::Binary(o, x1, x2), ExprKind::Binary(p, y1, y2)) => {
            o == p && same_path(x1, y1) && same_path(x2, y2)
        }
        (ExprKind::Index(x, xs), ExprKind::Index(y, ys)) => {
            same_path(x, y)
                && xs.len() == ys.len()
                && xs.iter().zip(ys).all(|(x, y)| same_path(x, y))
        }
        _ => false,
    }
}

/// Whether the store `place` is the [`steady`] path `e`.
pub(crate) fn place_is(place: &Place, e: &Expr) -> bool {
    use cpl_ir::PlaceKind as P;
    match (&place.kind, &e.kind) {
        (P::Local(x), ExprKind::Local(y)) => x == y,
        (P::Global(x), ExprKind::Global(y)) => x == y,
        (P::Field(x, i), ExprKind::Field(y, j)) => i == j && place_is(x, y),
        (P::Index(x, xs), ExprKind::Index(y, ys)) => {
            place_is(x, y)
                && xs.len() == ys.len()
                && xs.iter().zip(ys).all(|(x, y)| same_path(x, y))
        }
        _ => false,
    }
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
        | ExprKind::MutLine(place, _, _)
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

impl<'p> Emitter<'p> {
    /// The entry an enclosing `if k in m` found, when `base[at]` is it.
    pub(crate) fn found_cell(&self, base: &Place, at: &[Expr]) -> Option<String> {
        let [key] = at else {
            return None;
        };
        let found = self.found.iter().rev();
        found
            .clone()
            .find(|(m, k, _)| place_is(base, m) && same_path(key, k))
            .map(|f| f.2.clone())
    }

    /// `let p = mut m[k]` keyed by `slot`: find the slot once; reads trap on an
    /// absent key and writes make room for it, as `m[k]` does.
    fn let_map_slot(&mut self, slot: usize, map: &'p Place, init: &Expr, out: &mut String) {
        let put = std::mem::take(&mut self.slot_written_next);
        let name = self.names[slot].clone();
        // A key that is an unchanging parameter needs no copy of its own.
        let param = match init.kind {
            ExprKind::Local(src) if self.closure_id.is_none() && !init.ty.is_scalar() => {
                self.fn_id.is_some_and(|id| {
                    self.program.fns[id]
                        .params
                        .get(src)
                        .is_some_and(|p| !p.mutable)
                })
            }
            _ => false,
        };
        if param {
            let (cpp, src) = (self.ty(&init.ty), self.expr(init).text);
            self.line(out, &format!("const {cpp}& {name} = {src};"));
        } else {
            self.plain_let = true;
            self.stmt(&Stmt::Let(slot, Some(init.clone())), out);
            self.plain_let = false;
        }
        let m = self.place(map);
        let at = self.temp();
        self.need("map_slot");
        let find = if put {
            format!("&{m}[{name}]")
        } else {
            format!("cpl::map_slot({m}, {name})")
        };
        self.line(out, &format!("auto* {at} = {find};"));
        self.map_slots.insert((self.closure_id, slot), at);
    }
}

fn reads_a_local(e: &Expr) -> bool {
    let mut found = matches!(e.kind, ExprKind::Local(_));
    crate::walk::children(e, &mut |child| found |= reads_a_local(child));
    found
}

/// True when `<` on `ty` is a total order whose ties are equal values: the
/// derived order of integers, `Bool`, `Char`, positions, `Str` and what is
/// made of them. `Float` (`-0.0` and `0.0`, NaN) and an `impl Ord` are not.
pub(crate) fn identity_order(program: &cpl_ir::Program, ty: &Ty, depth: usize) -> bool {
    let all = |tys: Vec<&Ty>| {
        depth < 8
            && tys
                .into_iter()
                .all(|t| identity_order(program, t, depth + 1))
    };
    match ty {
        Ty::Unit | Ty::Int | Ty::I128 | Ty::U64 | Ty::Bool | Ty::Char | Ty::Str => true,
        Ty::Pos(_) | Ty::Bound(_) => true,
        Ty::Opt(t) => all(vec![&**t]),
        Ty::Tuple(items) => all(items.iter().collect()),
        Ty::Record(fields) => all(fields.iter().map(|f| &f.1).collect()),
        Ty::Struct(id) => {
            let s = &program.structs[*id];
            s.ord.is_none()
                && s.eq_fn.is_none()
                && s.cached_fields.is_empty()
                && all(s.fields.iter().map(|f| &f.1).collect())
        }
        Ty::Enum(id) => all(program.enums[*id]
            .variants
            .iter()
            .flat_map(|v| &v.1)
            .collect()),
        _ => false,
    }
}
