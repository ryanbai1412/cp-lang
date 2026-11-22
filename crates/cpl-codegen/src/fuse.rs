//! Producer-consumer loop fusion: `for x in (do { let v = []; for .. { v.push(e) }; give v }) { body }`
//! runs `body` at each `push`, so the temporary list is never built. It
//! majorizes: taken only when the two loops' effects cannot be told apart
//! from running them one after the other.

use crate::emit::{Emitter, Touch};
use crate::walk;
use cpl_ir::{
    BinOp, Block, Cast, Expr, ExprKind, Iter, LoopId, Mutation, Place, PlaceKind, Stmt, Ty, UnOp,
};

/// The consumer loop fused onto a producer's `push`.
#[derive(Clone)]
pub(crate) struct FusedPush {
    pub var: String,
    pub cpp: String,
    pub first: Vec<String>,
    pub body: Block,
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

    fn merge_touch(&self, t: &mut Touch, e: &Expr) {
        let et = self.touch(e);
        t.reads.extend(et.reads);
        t.writes.extend(et.writes);
        t.reads_domain.extend(et.reads_domain);
        t.writes_domain.extend(et.writes_domain);
        t.io |= et.io;
    }

    fn block_touch(&self, block: &Block) -> Touch {
        let mut t = Touch::default();
        walk::deep_stmts(block, &mut |stmt| match stmt {
            Stmt::Let(slot, init) => {
                t.writes.insert((false, *slot));
                if let Some(e) = init {
                    self.merge_touch(&mut t, e);
                }
            }
            Stmt::Assign(place, _, val) => {
                t.writes.insert(walk::root(place));
                walk::place_exprs(place, &mut |e| self.merge_touch(&mut t, e));
                self.merge_touch(&mut t, val);
            }
            Stmt::Expr(e) | Stmt::Give(e) => self.merge_touch(&mut t, e),
            Stmt::Return(Some(e)) => self.merge_touch(&mut t, e),
            Stmt::If(c, ..) | Stmt::While(_, c, _) => self.merge_touch(&mut t, c),
            Stmt::For(_, s, it, _) => {
                t.writes.insert((false, *s));
                iter_exprs(it, &mut |e| self.merge_touch(&mut t, e));
            }
            Stmt::DefDomain { id, lo, len } => {
                t.writes_domain.insert(*id);
                self.merge_touch(&mut t, lo);
                self.merge_touch(&mut t, len);
            }
            Stmt::DefDomainFromVal { id, val } => {
                t.writes_domain.insert(*id);
                self.merge_touch(&mut t, val);
            }
            Stmt::Assert { cond, message, .. } => {
                t.io = true;
                self.merge_touch(&mut t, cond);
                if let Some(m) = message {
                    self.merge_touch(&mut t, m);
                }
            }
            Stmt::Trap(..) => t.io = true,
            Stmt::Return(None) | Stmt::Break(_) | Stmt::Continue(_) => {}
        });
        t
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
        let ExprKind::Do(do_block) = &seq.kind else {
            return false;
        };
        let [Stmt::Let(vec_slot, Some(init)), Stmt::For(inner_id, inner_slot, inner_iter, inner_body), Stmt::Give(given)] =
            do_block.stmts.as_slice()
        else {
            return false;
        };
        if !matches!(init.ty, Ty::List(_))
            || !matches!(&init.kind, ExprKind::Tuple(items) if items.is_empty())
            || !matches!(given.kind, ExprKind::Local(g) if g == *vec_slot)
            || self.hoisted.contains(vec_slot)
        {
            return false;
        }
        let mut iter_uses_vec = false;
        iter_exprs(inner_iter, &mut |e| {
            iter_uses_vec |= expr_mentions_local(e, *vec_slot);
        });
        if iter_uses_vec || !only_pushed_in_block(*vec_slot, inner_body) {
            return false;
        }
        let mut outer_escapes = false;
        walk::deep_stmts(outer_body, &mut |stmt| match stmt {
            Stmt::Break(t) | Stmt::Continue(t) if t.is_none() || *t == Some(outer_id) => {
                outer_escapes = true;
            }
            _ => {}
        });
        if outer_escapes {
            return false;
        }
        let mut inner_touch = self.block_touch(inner_body);
        iter_exprs(inner_iter, &mut |e| {
            let et = self.touch(e);
            inner_touch.reads.extend(et.reads);
            inner_touch.writes.extend(et.writes);
            inner_touch.reads_domain.extend(et.reads_domain);
            inner_touch.writes_domain.extend(et.writes_domain);
            inner_touch.io |= et.io;
        });
        inner_touch.reads.remove(&(false, *vec_slot));
        inner_touch.writes.remove(&(false, *vec_slot));
        let mut outer_touch = self.block_touch(outer_body);
        outer_touch.reads.remove(&(false, outer_slot));
        outer_touch.writes.remove(&(false, outer_slot));
        if inner_touch.io || outer_touch.io || inner_touch.conflicts(&outer_touch) {
            return false;
        }
        // Fused, the two bodies interleave: fine when neither has effects the
        // other sees, and one of them can neither trap nor stop early.
        if !trap_free_block(inner_body, *vec_slot) && !trap_free_block(outer_body, usize::MAX) {
            return false;
        }
        self.fused_pushes.push((
            *vec_slot,
            FusedPush {
                var: var.to_string(),
                cpp: cpp.to_string(),
                first: first.to_vec(),
                body: outer_body.clone(),
            },
        ));
        let inside = walk::inside_list(*inner_slot, inner_iter, inner_body, self.locals());
        self.inside.extend(inside);
        self.for_stmt(*inner_id, *inner_slot, inner_iter, inner_body, out);
        self.inside
            .truncate(self.inside.len() - usize::from(inside.is_some()));
        self.fused_pushes.pop();
        true
    }
}

fn iter_exprs<'p>(iter: &'p Iter, f: &mut dyn FnMut(&'p Expr)) {
    match iter {
        Iter::Range { lo, hi, step, .. } => {
            f(lo);
            f(hi);
            if let Some(s) = step {
                f(s);
            }
        }
        Iter::RangeVal(e, _) | Iter::Subsets(e) | Iter::Members(e) | Iter::Seq(e, _) => f(e),
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

/// True when running `block` can neither trap, stop early nor loop forever,
/// so where it runs relative to other effect-free code cannot be seen.
/// Pushes onto `vec` count as nothing; the fused consumer is checked apart.
fn trap_free_block(block: &Block, vec: usize) -> bool {
    block.stmts.iter().all(|stmt| match stmt {
        Stmt::Let(_, init) => init.as_ref().is_none_or(trap_free),
        Stmt::Assign(place, _, val) => matches!(place.kind, PlaceKind::Local(_)) && trap_free(val),
        Stmt::Expr(e) => match &e.kind {
            ExprKind::Mutate(Mutation::Push, place, args) if matches!(place.kind, PlaceKind::Local(s) if s == vec) => {
                args.iter().all(trap_free)
            }
            _ => trap_free(e),
        },
        Stmt::If(cond, then, otherwise) => {
            trap_free(cond) && trap_free_block(then, vec) && trap_free_block(otherwise, vec)
        }
        Stmt::For(_, _, Iter::Range { lo, hi, step: None, .. }, body) => {
            trap_free(lo) && trap_free(hi) && trap_free_block(body, vec)
        }
        Stmt::For(_, _, Iter::Seq(seq, _), body) => {
            matches!(seq.kind, ExprKind::Local(_)) && trap_free_block(body, vec)
        }
        Stmt::Continue(None) => true,
        _ => false,
    })
}

/// True when `e` cannot trap: no checked index, call, unwrap or division by
/// what may be zero.
fn trap_free(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Unit
        | ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Char(_)
        | ExprKind::None
        | ExprKind::Local(_)
        | ExprKind::Global(_) => true,
        ExprKind::Some(x)
        | ExprKind::Field(x, _)
        | ExprKind::IsSome(x)
        | ExprKind::IsVariant(x, _)
        | ExprKind::Unwrap(x, false)
        | ExprKind::Cast(Cast::IntToFloat, x) => trap_free(x),
        ExprKind::Unary(op, x) => *op != UnOp::Neg && trap_free(x),
        ExprKind::Binary(op, l, r) => {
            let safe = match op {
                BinOp::Div | BinOp::Rem => matches!(r.kind, ExprKind::Int(k) if k != 0),
                BinOp::Pow | BinOp::Shl | BinOp::Shr => false,
                _ => true,
            };
            safe && l.ty != Ty::Str && trap_free(l) && trap_free(r)
        }
        ExprKind::Cmp(_, l, r) => trap_free(l) && trap_free(r),
        ExprKind::Tuple(items) | ExprKind::Variant(_, items) => items.iter().all(trap_free),
        ExprKind::If(c, t, f) => trap_free(c) && trap_free(t) && trap_free(f),
        _ => false,
    }
}
