//! `[f(x) for x in xs]`: the list a comprehension fills takes its source's
//! size up front, so it grows once instead of doubling. A filtered one,
//! `[f(x) for x in xs if p(x)]`, takes at most [`FILTERED_CAP`] of it: it
//! skips the first doublings and holds at most that many unused slots.

use crate::emit::{Emitter, ADD, PRIMARY};
use crate::walk;
use cpl_ir::{Block, ElemLoc, Expr, ExprKind, Iter, Mutation, PlaceKind, Stmt, Ty};

/// The most a list filled under a condition reserves ahead.
const FILTERED_CAP: usize = 16;

/// `t.push(e)` as a statement.
fn pushes_to(stmt: &Stmt, slot: usize) -> bool {
    matches!(stmt, Stmt::Expr(Expr { kind: ExprKind::Mutate(Mutation::Push, place, _), .. })
        if place.kind == PlaceKind::Local(slot))
}

impl<'p> Emitter<'p> {
    /// Writes `stmts[k]`, then the `reserve` [`Self::presize`] finds for it.
    pub(crate) fn stmt_presized(&mut self, stmts: &[Stmt], k: usize, out: &mut String) {
        self.stmt(&stmts[k], out);
        if let Some(line) = self.presize(&stmts[k..]) {
            self.line(out, &line);
        }
    }

    /// `t.reserve(n);` for `let t = []` that the next loop pushes to once a
    /// round over `n` rounds it knows ahead: a list or a plain range.
    pub(crate) fn presize(&mut self, stmts: &[Stmt]) -> Option<String> {
        let [Stmt::Let(slot, Some(init)), Stmt::For(_, _, iter, body), ..] = stmts else {
            return None;
        };
        let empty = matches!(&init.kind, ExprKind::Tuple(items) if items.is_empty());
        let loc = ElemLoc::Local(self.current_scope(), *slot);
        if !empty
            || !matches!(init.ty, Ty::List(_))
            || (self.closure_id.is_none() && self.devirt.dead_local(self.fn_id, *slot))
            || self.reused.contains(slot)
            || (self.pushing == Some(*slot) && self.closure_id.is_none())
            || self.elem_layout(loc).is_some()
            || !self.ty_for_loc(&init.ty, loc).starts_with("std::vector<")
        {
            return None;
        }
        let once = body.stmts.iter().filter(|s| pushes_to(s, *slot)).count() == 1
            && body
                .stmts
                .iter()
                .all(|s| matches!(s, Stmt::Let(..)) || pushes_to(s, *slot));
        if !once && !at_most_once(body, *slot) {
            return None;
        }
        if let Iter::Seq(src, _) = iter {
            if let Some(Expr {
                kind: ExprKind::Tuple(items),
                ..
            }) = self.inline_list_call(src)
            {
                let n = if once {
                    items.len()
                } else {
                    items.len().min(FILTERED_CAP)
                };
                return Some(format!("{}.reserve({n});", self.names[*slot]));
            }
        }
        let n = match iter {
            Iter::Seq(src, _)
                if walk::is_simple(src)
                    && (matches!(src.ty, Ty::List(_) | Ty::ListView(..) | Ty::Str)
                        || matches!(&src.ty, Ty::Array { dims, .. } if dims.len() == 1)) =>
            {
                format!("{}.size()", self.expr(src).at(PRIMARY))
            }
            Iter::Range {
                lo,
                hi,
                step: None,
                closed,
                ..
            } if [lo, hi]
                .iter()
                .all(|e| walk::is_simple(e) && e.ty == Ty::Int) =>
            {
                let (lo, hi) = (self.expr(lo).at(ADD + 1), self.expr(hi).at(ADD));
                let one = if *closed { " + 1" } else { "" };
                format!("(size_t)std::max<ll>(0, {hi} - {lo}{one})")
            }
            _ => return None,
        };
        let n = if once {
            n
        } else {
            format!("std::min<size_t>({n}, {FILTERED_CAP})")
        };
        Some(format!("{}.reserve({n});", self.names[*slot]))
    }
}

/// The loop body pushes to `slot` at most once a round, under conditions,
/// and does nothing else with it.
fn at_most_once(body: &Block, slot: usize) -> bool {
    let (mut pushes, mut loops, mut named) = (0, false, false);
    walk::deep_stmts(body, &mut |stmt| {
        pushes += usize::from(pushes_to(stmt, slot));
        loops |= matches!(stmt, Stmt::For(..) | Stmt::While(..));
    });
    walk::deep_exprs(body, &mut |e| {
        named |= matches!(e.kind, ExprKind::Local(s) if s == slot)
            || matches!(&e.kind, ExprKind::Mutate(_, place, _) if place.kind == PlaceKind::Local(slot) && !matches!(e.kind, ExprKind::Mutate(Mutation::Push, ..)));
    });
    pushes == 1 && !loops && !named
}
