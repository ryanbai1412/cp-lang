//! Mo's algorithm: offline ordering and window stepping over `Range<D>`.
use std.seq.{ends}

/// Offline traversal order of `queries` minimizing window endpoint moves to O((n + q) sqrt n).
/// Uses alternating right-endpoint direction within each sqrt-sized left block.
/// Returns query indices `0..queries.len` in traversal order. O(q log q).
pub fn mo_order<D>(queries: List<Range<D>>) -> List<Int> = todo

/// Positions entering (`true`) or leaving (`false`) as a window moves from `from` to `to`.
/// Always expands before shrinking so the active window is never inverted. O(|dl| + |dr|).
pub fn mo_steps<D>(from: Range<D>, to: Range<D>) -> List<(pos: D, entering: Bool)> = todo

/// Answer offline range queries in Mo order using in-place window moves.
/// Calls `add(pos)` / `remove(pos)` O((n + q) * sqrt(n)) times and `answer(qi)` `q` times
/// (when `answer(qi)` runs, the active window is `queries[qi]`), returning results in original
/// query order. Always expands before shrinking so the active window is never inverted.
/// Allocates only the traversal order and the results.
pub fn mo<D, R>(
    queries: List<Range<D>>,
    add: D -> (),
    remove: D -> (),
    answer: Int -> R,
) -> List<R> = todo

/// O(q log q). Offline Mo traversal order using Hilbert-curve keys (typically 1.5-2x fewer
/// endpoint moves than fixed-block snake order on dense queries).
pub fn mo_order_hilbert<D>(queries: List<Range<D>>) -> List<Int> = todo

/// O(1) space. Stateful Mo window over `D` for incremental/online stepping across ranges
/// without inverting the active interval.
pub struct MoWindow<D>:
    lo: Int
    hi: Int

    /// O(1). Create a window initialized to `start` (or the empty window at `D.first.l..D.first.l`).
    pub fn new(start: ?Range<D> = none) -> MoWindow<D> = todo

    /// O(1). Current active half-open window.
    pub @get fn range(self) -> Range<D> = todo

    /// O(|dl| + |dr|). Move the active window to `target`, expanding before shrinking.
    pub fn move_to(mut self, target: Range<D>, add: D -> (), remove: D -> ()) = todo

    /// O(|dl| + |dr|). Zero-allocation generator advancing the window to `target`,
    /// yielding `(pos, entering)` events (expanding before shrinking).
    pub fn step_to(mut self, target: Range<D>) -> yields (pos: D, entering: Bool) = todo

/// O((n + q) * sqrt(n)) `add` calls, zero `remove` calls. Rollback Mo for add-only / undoable
/// structures (DSU, max, convex hull). Calls `reset()` once per block, `mark()`/`undo()` around
/// left-side extensions, and `answer(qi)` when `queries[qi]` is active.
pub fn mo_rollback<D, M, R>(
    queries: List<Range<D>>,
    mark: () -> M,
    add: D -> (),
    answer: Int -> R,
    undo: M -> (),
    reset: () -> (),
) -> List<R> = todo

/// O(n^(5/3)) moves/updates. 3D Mo's algorithm with point updates over `(range, time)` queries.
/// Calls `apply_update(u, active_window)` and `undo_update(u, active_window)` as time moves.
pub fn mo_with_updates<D, U, R>(
    queries: List<(range: Range<D>, time: Int)>,
    updates: List<U>,
    add: D -> (),
    remove: D -> (),
    apply_update: (U, Range<D>) -> (),
    undo_update: (U, Range<D>) -> (),
    answer: Int -> R,
) -> List<R> = todo

@test
fn mo_order_and_steps_answer_distinct_counts() = todo
