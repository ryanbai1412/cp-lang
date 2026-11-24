//! Producer-consumer loop fusion: `for x in (do { let v = []; for .. { v.push(e) }; give v }) { body }`
//! runs `body` at each `push`, so the temporary list is never built. It
//! majorizes: taken only when the two loops' effects cannot be told apart
//! from running them one after the other.

use crate::emit::Emitter;
use crate::uses::effects::{trap_free_block, writes_own};
use crate::uses::loops::stmt_sites;
use crate::uses::places::each_part;
use crate::walk;
use cpl_ir::{Block, Expr, ExprKind, Iter, LoopId, Mutation, Place, PlaceKind, Stmt, Ty};

/// The consumer loop fused onto a producer's `push`.
#[derive(Clone)]
pub(crate) struct FusedPush {
    pub var: String,
    pub cpp: String,
    pub first: Vec<String>,
    /// Shared, not copied, by each `push` it runs at: a profile counts the
    /// body's statements by address.
    pub body: std::rc::Rc<Block>,
    /// The consumer loop's variable, a new value each `push`.
    pub slot: Option<usize>,
    /// Hands each element to this callback instead (`crate::sink`).
    pub sink: Option<String>,
}

impl<'p> Emitter<'p> {
    /// `vec.push(elem)` in a producer whose consumer loop was fused onto it:
    /// runs the consumer body on `elem` in place.
    pub(crate) fn fused_push(&mut self, place: &Place, args: &[Expr], out: &mut String) -> bool {
        let PlaceKind::Local(v) = place.kind else {
            return false;
        };
        let Some(consumer) = self
            .fused_pushes
            .iter()
            .rev()
            .find(|(s, _)| *s == v)
            .map(|(_, c)| c.clone())
        else {
            return false;
        };
        let [elem] = args else {
            return false;
        };
        if let Some(body) = self.parted(&consumer, elem) {
            self.line(out, "{");
            self.indent += 1;
            let key: *const Block = &body;
            let held = consumer.slot.and_then(|v| self.loop_vars.insert(key, v));
            self.block(&body, out);
            match held {
                Some(v) => self.loop_vars.insert(key, v),
                None => self.loop_vars.remove(&key),
            };
            self.indent -= 1;
            self.line(out, "}");
            return true;
        }
        let elem = self.expr(elem).text;
        if let Some(put) = &consumer.sink {
            self.line(out, &format!("{put}({elem});"));
            return true;
        }
        self.line(out, "{");
        self.indent += 1;
        self.line(out, &format!("{} {} = {elem};", consumer.cpp, consumer.var));
        for line in &consumer.first {
            self.line(out, line);
        }
        // The body is a copy of the loop's, so its variable is told apart
        // by this copy's address while it is written.
        let key = std::rc::Rc::as_ptr(&consumer.body);
        let held = consumer.slot.and_then(|v| self.loop_vars.insert(key, v));
        self.block(&consumer.body, out);
        match held {
            Some(v) => self.loop_vars.insert(key, v),
            None => self.loop_vars.remove(&key),
        };
        self.indent -= 1;
        self.line(out, "}");
        true
    }

    /// The consumer body with its pattern bound part by part, when `elem`
    /// is a tuple the body only takes apart: `for (x, y) in` over a push of
    /// `(a, b)` binds `x = a` and `y = b`, and the tuple is never built.
    /// The parts run in the tuple's order. A part read from a variable
    /// keeps the tuple, whose copy it is anyway. A profile counts the
    /// body's own statements, so it keeps them.
    fn parted(&self, consumer: &FusedPush, elem: &Expr) -> Option<Block> {
        let (Some(t), ExprKind::Tuple(items), None, true) = (
            consumer.slot,
            &elem.kind,
            &self.sites,
            consumer.first.is_empty() && consumer.sink.is_none(),
        ) else {
            return None;
        };
        let n = items.len();
        let head = consumer.body.stmts.get(..n)?;
        let mut stmts = Vec::with_capacity(consumer.body.stmts.len());
        for (i, (item, stmt)) in items.iter().zip(head).enumerate() {
            let Stmt::Let(
                part,
                Some(Expr {
                    kind: ExprKind::Field(base, j),
                    ..
                }),
            ) = stmt
            else {
                return None;
            };
            let named_in_items = |slot: usize| {
                crate::uses::reads_local(items, &|s| self.names.get(s) == self.names.get(slot))
            };
            if *j != i
                || !matches!(base.kind, ExprKind::Local(s) if s == t)
                || (!item.ty.is_scalar() && crate::stmt::place_root(item).is_some())
                || named_in_items(*part)
            {
                return None;
            }
            stmts.push(Stmt::Let(*part, Some(item.clone())));
        }
        let rest = &consumer.body.stmts[n..];
        if n == 0 || crate::uses::mentions_in(self.program, rest, t) {
            return None;
        }
        stmts.extend(rest.iter().cloned());
        Some(Block { stmts })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn try_fuse_producer(
        &mut self,
        outer_id: LoopId,
        outer_slot: usize,
        var: &str,
        cpp: &str,
        first: &[String],
        seq: &Expr,
        outer_body: &Block,
        out: &mut String,
    ) -> bool {
        let Some(Producer {
            vec: vec_slot,
            id: inner_id,
            slot: inner_slot,
            iter: inner_iter,
            body: inner_body,
        }) = producer(seq)
        else {
            return false;
        };
        if self.hoisted.contains(vec_slot) {
            return false;
        }
        let written = outer_body;
        let mut outer_body = outer_body.clone();
        let Some(stops) = retarget_breaks(&mut outer_body, outer_id, *inner_id, 0) else {
            return false;
        };
        let outer_body = &outer_body;
        let mut inner_touch = self.block_touch(inner_body);
        iter_exprs(inner_iter, &mut |e| {
            inner_touch.merge(self.touch(e));
        });
        inner_touch.reads.remove(&(false, *vec_slot));
        inner_touch.writes.remove(&(false, *vec_slot));
        inner_touch.binds.remove(vec_slot);
        let mut outer_touch = self.block_touch(outer_body);
        outer_touch.reads.remove(&(false, outer_slot));
        outer_touch.writes.remove(&(false, outer_slot));
        outer_touch.binds.remove(&outer_slot);
        // Fused, the two bodies interleave: fine when neither has effects the
        // other sees, and one of them can neither trap nor stop early. A
        // producer that can do neither and writes only its own locals runs
        // unseen, so the consumer may print or trap.
        let traps = self.target.checks != crate::Checks::None;
        let unseen = !inner_touch.io
            && trap_free_block(inner_body, *vec_slot, traps)
            && writes_own(inner_body);
        if inner_touch.io || (outer_touch.io && !unseen) || inner_touch.conflicts(&outer_touch) {
            return false;
        }
        if !trap_free_block(inner_body, *vec_slot, traps)
            && !trap_free_block(outer_body, usize::MAX, traps)
        {
            return false;
        }
        // A `break` of the consumer skips the rest of the producer: fine when
        // the rest can neither trap nor write anything but its own locals.
        // A consumer fused onto this loop's pushes may break out of it, and
        // fused, this loop's end is the producer's.
        let fed = self.fed_breaks_to(outer_body, outer_id);
        if (stops || fed)
            && !(trap_free_block(inner_body, *vec_slot, traps) && writes_own(inner_body))
        {
            return false;
        }
        if stops {
            self.break_targets.insert(*inner_id);
        }
        self.fused_pushes.push((
            *vec_slot,
            FusedPush {
                var: var.to_string(),
                cpp: cpp.to_string(),
                first: first.to_vec(),
                body: std::rc::Rc::new(outer_body.clone()),
                slot: Some(outer_slot),
                sink: None,
            },
        ));
        if let (Some(_), Some((_, fused))) = (&self.sites, self.fused_pushes.last()) {
            carry(&mut self.counted, written, &fused.body);
            carry(&mut self.entries, written, &fused.body);
        }
        let inside = walk::inside_list(*inner_slot, inner_iter, inner_body, self.locals());
        self.inside.extend(
            inside.map(|(l, i)| crate::bounds::Inside::List((false, l, None), i, None, 0, 0)),
        );
        self.for_stmt(*inner_id, *inner_slot, inner_iter, inner_body, out);
        if fed {
            self.line(out, &format!("brk_{outer_id}:;"));
        }
        self.inside
            .truncate(self.inside.len() - usize::from(inside.is_some()));
        self.fused_pushes.pop();
        true
    }

    /// Whether a consumer fused onto a list `body` pushes to breaks out of
    /// the loop `id`, directly or through a consumer fused onto its own pushes.
    fn fed_breaks_to(&self, body: &Block, id: LoopId) -> bool {
        let mut consumers = Vec::new();
        crate::uses::split::each_value(body, &mut |e| {
            let ExprKind::Mutate(Mutation::Push, place, _) = &e.kind else {
                return;
            };
            let PlaceKind::Local(v) = place.kind else {
                return;
            };
            if let Some((_, c)) = self.fused_pushes.iter().rev().find(|(s, _)| *s == v) {
                if c.sink.is_none() {
                    consumers.push(c.body.clone());
                }
            }
        });
        consumers.iter().any(|b| {
            retarget_breaks(&mut (**b).clone(), id, id, 1).unwrap_or(true)
                || self.fed_breaks_to(b, id)
        })
    }
}

/// The statements of `copy` (a clone of `from`) put in `set` where the
/// ones they copy are: a profile's sites are keyed by address.
fn carry(set: &mut std::collections::HashSet<*const Stmt>, from: &Block, copy: &Block) {
    let froms = stmt_sites(from);
    let copies = stmt_sites(copy);
    debug_assert_eq!(froms.len(), copies.len());
    for (from, copy) in froms.into_iter().zip(copies) {
        if set.contains(&from) {
            set.insert(copy);
        }
    }
}

/// `do { let v = []; for .. { v.push(e) }; give v }`: a list built by one
/// loop that only pushes onto it.
pub(crate) struct Producer<'p> {
    pub vec: &'p usize,
    pub id: &'p LoopId,
    pub slot: &'p usize,
    pub iter: &'p Iter,
    pub body: &'p Block,
}

pub(crate) fn producer(seq: &Expr) -> Option<Producer<'_>> {
    let ExprKind::Do(do_block) = &seq.kind else {
        return None;
    };
    block_producer(do_block)
}

/// [`producer`] for the block of a `do`.
pub(crate) fn block_producer(do_block: &Block) -> Option<Producer<'_>> {
    stmts_producer(&do_block.stmts)
}

/// [`block_producer`] for a `do` block's statements.
pub(crate) fn stmts_producer(stmts: &[Stmt]) -> Option<Producer<'_>> {
    let [Stmt::Let(vec, Some(init)), Stmt::For(id, slot, iter, body), Stmt::Give(given)] = stmts
    else {
        return None;
    };
    if !matches!(init.ty, Ty::List(_))
        || !matches!(&init.kind, ExprKind::Tuple(items) if items.is_empty())
        || !matches!(given.kind, ExprKind::Local(g) if g == *vec)
    {
        return None;
    }
    let mut iter_uses_vec = false;
    iter_exprs(iter, &mut |e| iter_uses_vec |= expr_mentions_local(e, *vec));
    (!iter_uses_vec && only_pushed_in_block(*vec, body)).then_some(Producer {
        vec,
        id,
        slot,
        iter,
        body,
    })
}

pub(crate) fn iter_exprs<'p>(iter: &'p Iter, f: &mut dyn FnMut(&'p Expr)) {
    match iter {
        Iter::Range { lo, hi, step, .. } => {
            f(lo);
            f(hi);
            if let Some(s) = step {
                f(s);
            }
        }
        Iter::RangeVal(e, _) | Iter::Seq(e, _) => f(e),
        Iter::TreeRange {
            container, lo, hi, ..
        } => {
            f(container);
            if let Some((e, _)) = lo {
                f(e);
            }
            if let Some((e, _)) = hi {
                f(e);
            }
        }
    }
}

fn expr_mentions_local(e: &Expr, target: usize) -> bool {
    let mut found = false;
    each_part(e, &mut |sub| {
        if matches!(sub.kind, ExprKind::Local(s) if s == target) {
            found = true;
        }
    });
    found
}

fn place_mentions_local(p: &Place, target: usize) -> bool {
    if matches!(p.kind, PlaceKind::Local(s) if s == target) {
        return true;
    }
    let mut found = false;
    walk::place_exprs(p, &mut |e| {
        if expr_mentions_local(e, target) {
            found = true;
        }
    });
    found
}

fn only_pushed_in_block(vec_slot: usize, block: &Block) -> bool {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Expr(e) => {
                if let ExprKind::Mutate(Mutation::Push, place, args) = &e.kind {
                    if matches!(place.kind, PlaceKind::Local(s) if s == vec_slot) && args.len() == 1
                    {
                        if expr_mentions_local(&args[0], vec_slot) {
                            return false;
                        }
                        continue;
                    }
                }
                if expr_mentions_local(e, vec_slot) {
                    return false;
                }
            }
            Stmt::If(cond, then, otherwise) => {
                if expr_mentions_local(cond, vec_slot)
                    || !only_pushed_in_block(vec_slot, then)
                    || !only_pushed_in_block(vec_slot, otherwise)
                {
                    return false;
                }
            }
            Stmt::While(_, cond, body) => {
                if expr_mentions_local(cond, vec_slot) || !only_pushed_in_block(vec_slot, body) {
                    return false;
                }
            }
            Stmt::For(_, slot, iter, body) => {
                if *slot == vec_slot {
                    return false;
                }
                let mut used = false;
                iter_exprs(iter, &mut |e| used |= expr_mentions_local(e, vec_slot));
                if used || !only_pushed_in_block(vec_slot, body) {
                    return false;
                }
            }
            Stmt::Let(slot, init) => {
                if *slot == vec_slot
                    || init
                        .as_ref()
                        .is_some_and(|e| expr_mentions_local(e, vec_slot))
                {
                    return false;
                }
            }
            Stmt::Assign(place, _, val) => {
                if place_mentions_local(place, vec_slot) || expr_mentions_local(val, vec_slot) {
                    return false;
                }
            }
            Stmt::Give(_) => return false,
            Stmt::Return(e) => {
                if e.as_ref().is_some_and(|e| expr_mentions_local(e, vec_slot)) {
                    return false;
                }
            }
            Stmt::Assert { cond, message, .. } => {
                if expr_mentions_local(cond, vec_slot)
                    || message
                        .as_ref()
                        .is_some_and(|m| expr_mentions_local(m, vec_slot))
                {
                    return false;
                }
            }
            Stmt::DefDomain { lo, len, .. } => {
                if expr_mentions_local(lo, vec_slot) || expr_mentions_local(len, vec_slot) {
                    return false;
                }
            }
            Stmt::DefDomainFromVal { val, .. } => {
                if expr_mentions_local(val, vec_slot) {
                    return false;
                }
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => {}
        }
    }
    true
}

/// Points each `break` of loop `outer` in its body at loop `inner`
/// instead; `Some(true)` when there is one. `None` when the body has a
/// `continue` of `outer`, which the producer has no place for.
fn retarget_breaks(b: &mut Block, outer: LoopId, inner: LoopId, depth: usize) -> Option<bool> {
    let mut stops = false;
    for stmt in &mut b.stmts {
        match stmt {
            Stmt::Break(t) if *t == Some(outer) || (t.is_none() && depth == 0) => {
                *t = Some(inner);
                stops = true;
            }
            Stmt::Continue(t) if *t == Some(outer) || (t.is_none() && depth == 0) => return None,
            Stmt::If(_, then, otherwise) => {
                stops |= retarget_breaks(then, outer, inner, depth)?;
                stops |= retarget_breaks(otherwise, outer, inner, depth)?;
            }
            Stmt::While(_, _, body) | Stmt::For(_, _, _, body) => {
                stops |= retarget_breaks(body, outer, inner, depth + 1)?;
            }
            _ => {}
        }
    }
    Some(stops)
}
