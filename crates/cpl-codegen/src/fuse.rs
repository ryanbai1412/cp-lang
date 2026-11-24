//! Producer-consumer loop fusion: `for x in (do { let v = []; for .. { v.push(e) }; give v }) { body }`
//! runs `body` at each `push`, so the temporary list is never built. It
//! majorizes: taken only when the two loops' effects cannot be told apart
//! from running them one after the other.

use crate::emit::Emitter;
use crate::uses::effects::{trap_free_block, writes_own};
use crate::walk;
use cpl_ir::{Block, Expr, ExprKind, Iter, LoopId, Mutation, Place, PlaceKind, Stmt, Ty};

/// The consumer loop fused onto a producer's `push`.
#[derive(Clone)]
pub(crate) struct FusedPush {
    pub var: String,
    pub cpp: String,
    pub first: Vec<String>,
    pub body: Block,
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
        self.block(&consumer.body, out);
        self.indent -= 1;
        self.line(out, "}");
        true
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
        if inner_touch.io || outer_touch.io || inner_touch.conflicts(&outer_touch) {
            return false;
        }
        // Fused, the two bodies interleave: fine when neither has effects the
        // other sees, and one of them can neither trap nor stop early.
        let traps = self.target.checks != crate::Checks::None;
        if !trap_free_block(inner_body, *vec_slot, traps)
            && !trap_free_block(outer_body, usize::MAX, traps)
        {
            return false;
        }
        // A `break` of the consumer skips the rest of the producer: fine when
        // the rest can neither trap nor write anything but its own locals.
        if stops && !(trap_free_block(inner_body, *vec_slot, traps) && writes_own(inner_body)) {
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
                body: outer_body.clone(),
                sink: None,
            },
        ));
        let inside = walk::inside_list(*inner_slot, inner_iter, inner_body, self.locals());
        self.inside.extend(
            inside.map(|(l, i)| crate::bounds::Inside::List((false, l, None), i, None, 0, 0)),
        );
        self.for_stmt(*inner_id, *inner_slot, inner_iter, inner_body, out);
        self.inside
            .truncate(self.inside.len() - usize::from(inside.is_some()));
        self.fused_pushes.pop();
        true
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
    let [Stmt::Let(vec, Some(init)), Stmt::For(id, slot, iter, body), Stmt::Give(given)] =
        do_block.stmts.as_slice()
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

fn iter_exprs<'p>(iter: &'p Iter, f: &mut dyn FnMut(&'p Expr)) {
    match iter {
        Iter::Walk { .. } => unreachable!("Iter::Walk is expanded before codegen"),
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
    walk::deep_expr(e, &mut |sub| {
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
