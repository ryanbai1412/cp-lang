//! A fed loop a traced chunk runs: the `yields` fn on the tree-walker's
//! call path, which reports the call and feeds each value to the loop's
//! body in this chunk.

use crate::observe::Observer;
use crate::value::Value;
use crate::vm::helpers::{place_indices, sync_from_stack, sync_to_stack, yielded_items};
use crate::vm::ir::{Chunk, TracedFeed};
use crate::{Fed, Interp, Res};
use cpl_diag::Span;
use cpl_ir::{Arg, FnId};

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
        let fed = self.host_feed(chunk, feed);
        let returned = self.call_at(id, base, span, Some(fed));
        self.fed_hosts.pop();
        returned
    }

    /// The loop `feed` of `chunk`, whose body this frame now hosts.
    fn host_feed(&mut self, chunk: &Chunk<'p>, feed: TracedFeed<'p>) -> Fed<'p> {
        let caller = self.base;
        self.fed_hosts
            .push((caller, chunk as *const Chunk<'p> as *const ()));
        Fed {
            callee: 0,
            acc: feed.acc,
            caller,
            id: feed.id,
            slot: feed.slot,
            body: feed.body,
            left: None,
        }
    }

    /// At `CallTraced` op `pc` in a run from a checkpoint, what the call
    /// returned or returns when the statement there waits on it.
    /// The `mut` arguments in `lends` are lent from their places, which
    /// hold the values the callee had, as the call lends them.
    #[cold]
    pub(super) fn resume_traced(
        &mut self,
        chunk: &Chunk<'p>,
        pc: usize,
        (args, lends, regs_v, first): (&mut [Value], Option<&'p [Arg]>, &mut [Value], usize),
    ) -> Res<Option<Value>> {
        if let Some(value) = self.returned_at(chunk, pc) {
            return Ok(Some(value));
        }
        let Some(mut frames) = self.resumed_feed(chunk, pc) else {
            return Ok(None);
        };
        let mut at = first;
        for (i, arg) in lends.into_iter().flatten().enumerate() {
            let Arg::Mut(place) = arg else {
                continue;
            };
            let mut exprs = Vec::new();
            place_indices(place, &mut exprs);
            let indices: Vec<Value> = regs_v[at..at + exprs.len()].to_vec();
            at += exprs.len();
            let part = self.part_of(place, &mut indices.into_iter());
            let now = std::mem::replace(&mut frames[0].locals[i], Value::Unit);
            *self.part_mut(&part, place.span)? = now;
        }
        self.lend_args(lends, args, regs_v, first)?;
        for (i, arg) in lends.into_iter().flatten().enumerate() {
            if matches!(arg, Arg::Mut(_)) {
                frames[0].locals[i] = args[i].clone();
            }
        }
        let feed = (chunk.traced_feeds.iter()).find(|f| f.0 == pc as u32);
        let fed = feed.map(|feed| self.host_feed(chunk, feed.1));
        let feeding = fed.is_some();
        let returned = self.resume_callee(&frames, 0, fed);
        if feeding {
            self.fed_hosts.pop();
        }
        returned.map(Some)
    }
}
