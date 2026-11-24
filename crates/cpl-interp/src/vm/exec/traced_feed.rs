//! A fed loop a traced chunk runs: the `yields` fn on the tree-walker's
//! call path, which reports the call and feeds each value to the loop's
//! body in this chunk.

use crate::observe::Observer;
use crate::value::Value;
use crate::vm::ir::{Chunk, TracedFeed};
use crate::{Fed, Interp, Res};
use cpl_diag::Span;
use cpl_ir::FnId;

impl<'p, 'io, O: Observer> Interp<'p, 'io, O> {
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
        self.fed_hosts.push((caller, chunk as *const Chunk<'p>));
        let returned = self.call_at(id, base, span, Some(fed));
        self.fed_hosts.pop();
        returned
    }
}
