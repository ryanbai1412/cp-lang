//! A fed loop a traced chunk runs: the `yields` fn on the tree-walker's
//! call path, which reports the call and feeds each value to the loop's
//! body in this chunk.

use crate::observe::Observer;
use crate::value::Value;
use crate::vm::helpers::{sync_from_stack, sync_to_stack, yielded_items};
use crate::vm::ir::{Chunk, TracedFeed};
use crate::{Fed, Interp, Res};
use cpl_diag::Span;
use cpl_ir::FnId;

impl<'p, 'io, O: Observer> Interp<'p, 'io, O> {
    /// Feeds the fed loop `val`, or with `each` each item of it, and returns
    /// the loop's updated value. The loop's body shows a lent argument in its
    /// caller's place, so the argument's slot holds its latest value.
    pub(super) fn feed_tree(
        &mut self,
        chunk: &Chunk<'p>,
        regs_i: &mut [i64],
        regs_v: &mut [Value],
        val: Value,
        each: bool,
        span: Span,
    ) -> Res<Value> {
        let lends = self.lends();
        if lends {
            let (slots, base) = (&chunk.slot_map, self.base);
            sync_to_stack(slots, regs_i, regs_v, &mut self.stack, base, true);
        }
        let items = match each {
            true => yielded_items(val, span)?,
            false => vec![val],
        };
        let mut updated = Ok(Value::Unit);
        for item in items {
            updated = self.feed(item, span);
            if updated.is_err() {
                break;
            }
        }
        if lends {
            let (slots, base) = (&chunk.slot_map, self.base);
            sync_from_stack(slots, regs_i, regs_v, &mut self.stack, base, true);
        }
        updated
    }

    pub(super) fn call_traced_feed(
        &mut self,
        chunk: &Chunk<'p>,
        feed: TracedFeed<'p>,
        id: FnId,
        base: usize,
        span: Span,
    ) -> Res<Value> {
        let caller = self.base;
        let fed = Fed {
            callee: 0,
            acc: feed.acc,
            caller,
            id: feed.id,
            slot: feed.slot,
            body: feed.body,
            left: None,
        };
        self.fed_hosts
            .push((caller, chunk as *const Chunk<'p> as *const ()));
        let returned = self.call_at(id, base, span, Some(fed));
        self.fed_hosts.pop();
        returned
    }
}
