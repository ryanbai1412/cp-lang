//! Mo's algorithm: offline ordering and window stepping over `Range<D>`.
use std.seq.{ends}

/// The order in which to visit `queries` so a sliding window moves its endpoints
/// O((n + q) sqrt n) times, as a permutation of `0..queries.len`.
///
/// Let `lo` be the least start and `n` the distance from `lo` to the greatest end over all
/// queries, and `b = max(1, isqrt(n))`. Query `i` falls in block `(queries[i].start - lo) / b`.
/// Blocks are visited in increasing order; within an even block queries go by increasing end,
/// within an odd block by decreasing end (the snake that saves the return sweep).
///
/// cost: O(q log q)
/// empty: `[]`
/// ties: equal block and end keep the earlier query first
/// order: increasing block, then end increasing in even blocks and decreasing in odd ones
/// traps: never
/// example: `mo_order([0..3, 2..4, 0..1, 3..4]) == [2, 0, 1, 3]`
/// example: `mo_order([5..5]) == [0]`
pub fn mo_order<D>(queries: List<Range<D>>) -> List<Int> = todo

/// The positions that enter (`entering: true`) or leave (`entering: false`) a window as it
/// moves from `from` to `to`, in the order a caller should apply them.
///
/// The window grows before it shrinks, so it is never inverted even when `from` and `to` are
/// disjoint: first the left end moves left (entering positions in decreasing order), then the
/// right end moves right (entering, increasing), then the left end moves right (leaving,
/// increasing), then the right end moves left (leaving, decreasing).
///
/// cost: O(|Δstart| + |Δend|)
/// empty: `[]` when `from == to`
/// order: left growth, right growth, left shrink, right shrink, each outward-to-inward as above
/// traps: never
/// example: `mo_steps(1..3, 0..2) == [(pos: 0, entering: true), (pos: 2, entering: false)]`
/// example: `mo_steps(0..1, 2..3).len == 4`
/// example: `mo_steps(0..1, 2..3) == [(pos: 1, entering: true), (pos: 2, entering: true), (pos: 0, entering: false), (pos: 1, entering: false)]`
pub fn mo_steps<D>(from: Range<D>, to: Range<D>) -> List<(pos: D, entering: Bool)> = todo

/// Answers offline range queries by moving one window through them in
/// [`mo_order`](mo_order) and calling `add(pos)` / `remove(pos)` for each step of
/// [`mo_steps`](mo_steps). When `answer(qi)` runs, the active window is exactly `queries[qi]`.
///
/// The window starts empty at the start of the first visited query. Results come back in the
/// original query order, so `result[i]` answers `queries[i]`. Allocates only the order and the
/// results.
///
/// cost: O((n + q) sqrt n) calls to `add` and `remove`, `q` calls to `answer`, plus O(q log q)
/// empty: `[]`, calling nothing
/// order: `result[i]` answers `queries[i]`
/// traps: never; traps raised by the callbacks propagate
/// example: `let a = [1, 2, 3]; var s = 0; mo([0..2, 1..3], add: i => s += a[i], remove: i => s -= a[i], answer: _ => s) == [3, 5]`
pub fn mo<D, R>(
    queries: List<Range<D>>,
    add: D -> (),
    remove: D -> (),
    answer: Int -> R,
) -> List<R> = todo

/// The order in which to visit `queries`, by Hilbert-curve key: typically 1.5-2x fewer endpoint
/// moves than [`mo_order`](mo_order) on dense queries.
///
/// Let `lo` be the least start and `n` the distance from `lo` to the greatest end, and `k` the
/// least integer with `2^k > n`. Query `i` is the point `(x, y) = (start - lo, end - lo)` on the
/// `2^k × 2^k` grid and its key is that point's index on the standard Hilbert curve (the
/// `xy2d` construction that starts at `(0, 0)`, visits `(0, 1)` second and ends at `(2^k - 1, 0)`).
///
/// cost: O(q log q + q log n)
/// empty: `[]`
/// ties: equal keys (equal ranges) keep the earlier query first
/// order: increasing Hilbert key
/// traps: never
/// example: `mo_order_hilbert([1..1, 0..1, 0..0]) == [2, 1, 0]`
/// example: `mo_order_hilbert([0..2, 1..3, 2..2]).sorted() == [0, 1, 2]`
pub fn mo_order_hilbert<D>(queries: List<Range<D>>) -> List<Int> = todo

/// Mo's algorithm for structures that can add and undo but not remove (DSU, running max,
/// convex hull): answers every query with `add` calls only, restoring state with `mark` /
/// `undo` instead of `remove`. When `answer(qi)` runs, the state is exactly the result of
/// adding the positions of `queries[qi]` to a reset structure.
///
/// Blocks are as in [`mo_order`](mo_order). A query that lies inside one block is answered on
/// its own: `m = mark()`, add its positions, answer, `undo(m)`. The other queries of a block go
/// by increasing end: `reset()` once at the block's first such query, the right part grows from
/// the block's end with `add`, and each query's left part is added between a `mark()` and the
/// matching `undo`. Results come back in the original query order.
///
/// cost: O((n + q) sqrt n) calls to `add`, no removals, one `reset` per block
/// empty: `[]`, calling nothing
/// order: `result[i]` answers `queries[i]`
/// traps: never; traps raised by the callbacks propagate
/// example: `let a = [1, 2, 3]; var s = 0; mo_rollback([0..2, 1..3], mark: () => s, add: i => s += a[i], answer: _ => s, undo: m => s = m, reset: () => s = 0) == [3, 5]`
pub fn mo_rollback<D, M, R>(
    queries: List<Range<D>>,
    mark: () -> M,
    add: D -> (),
    answer: Int -> R,
    undo: M -> (),
    reset: () -> (),
) -> List<R> = todo

/// Mo's algorithm with point updates: query `i` asks about `queries[i].range` after the first
/// `queries[i].time` updates, `updates[0..time]`, have been applied.
///
/// Moving forward in time calls `apply_update(updates[t], window)`, moving back calls
/// `undo_update(updates[t], window)`, where `window` is the active range, so the callback can
/// fix the window's aggregate when the updated position is inside it. With `n` the span of the
/// query ranges and `b = max(1, floor(n^(2/3)))`, queries go by `(start block, end block, time)`,
/// all increasing. Time starts at 0 and the window starts empty. Results come back in the
/// original query order.
///
/// cost: O(n^(5/3) + q log q) window moves and updates for `q ~ n` queries
/// empty: `[]`, calling nothing
/// ties: equal keys keep the earlier query first
/// order: `result[i]` answers `queries[i]`
/// traps: when a `time` is outside `0..=updates.len`
/// example: `var a = [1, 2, 3]; let qs = [(range: 0..2, time: 1), (range: 1..3, time: 0)]; mo_with_updates(qs, [(pos: 1, by: 10)], add: _ => (), remove: _ => (), apply_update: (u, _) => a[u.pos] += u.by, undo_update: (u, _) => a[u.pos] -= u.by, answer: qi => a.slice(qs[qi].range).sum()) == [13, 5]`
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
