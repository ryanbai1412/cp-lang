//! Mo's algorithm: offline ordering and window stepping over `Range<D>`.
use std.seq.{ends}

/// Offline traversal order of `queries` minimizing window endpoint moves to O((n + q) sqrt n).
/// Uses alternating right-endpoint direction within each sqrt-sized left block.
/// Returns query indices `0..queries.len` in traversal order. O(q log q).
pub fn mo_order<D>(queries: List<Range<D>>) -> List<Int>:
    let q = queries.len
    if q == 0: return []
    let b = max(1, D.K / max(1, isqrt(q)))
    var pairs: List<(Int, Int)> = []
    unchecked:
        let span = D.K + 1
        for i in 0..q:
            let (l, r) = queries[i].ends
            let blk = l / b
            let k = blk * span + (if blk % 2 == 0: r else: D.K - r)
            pairs.push((k, i))
        pairs.sort()
        return [p.1 for p in pairs]

/// Positions entering (`true`) or leaving (`false`) as a window moves from `from` to `to`.
/// Always expands before shrinking so the active window is never inverted. O(|dl| + |dr|).
pub fn mo_steps<D>(from: Range<D>, to: Range<D>) -> List<(pos: D, entering: Bool)>:
    var (l, r) = from.ends
    let (nl, nr) = to.ends
    var out: List<(pos: D, entering: Bool)> = []
    while l > nl:
        l -= 1
        out.push((pos: D.nth(l), entering: true))
    while r < nr:
        out.push((pos: D.nth(r), entering: true))
        r += 1
    while l < nl:
        out.push((pos: D.nth(l), entering: false))
        l += 1
    while r > nr:
        r -= 1
        out.push((pos: D.nth(r), entering: false))
    out

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
) -> List<R>:
    let q = queries.len
    if q == 0: return []
    let order = mo_order(queries)
    var (l, r) = (0, 0)
    unchecked:
        let q0 = order[0]
        let (nl0, nr0) = queries[q0].ends
        while l > nl0:
            l -= 1
            add(D.nth(l))
        while r < nr0:
            add(D.nth(r))
            r += 1
        while l < nl0:
            remove(D.nth(l))
            l += 1
        while r > nr0:
            r -= 1
            remove(D.nth(r))
        var out = List.filled(q, answer(q0))
        for i in 1..q:
            let qi = order[i]
            let (nl, nr) = queries[qi].ends
            while l > nl:
                l -= 1
                add(D.nth(l))
            while r < nr:
                add(D.nth(r))
                r += 1
            while l < nl:
                remove(D.nth(l))
                l += 1
            while r > nr:
                r -= 1
                remove(D.nth(r))
            out[qi] = answer(qi)
        return out

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
fn mo_order_and_steps_answer_distinct_counts():
    domain I = 1..=20
    let a = [I] _ => rand(0, 5)
    var queries: List<Range<I>> = []
    for _ in 0..40:
        let l = I.nth(rand(0, 19))
        let r = I.nth(rand(l.to_idx, 19))
        queries.push(l..=r)
    var freq = List.filled(6, 0)
    var distinct = 0
    var win = I.first.l..I.first.l
    var ans = List.filled(queries.len, 0)
    for qi in mo_order(queries):
        let target = queries[qi]
        for s in mo_steps(win, target):
            let v = a[s.pos]
            if s.entering:
                if freq[v] == 0: distinct += 1
                freq[v] += 1
            else:
                freq[v] -= 1
                if freq[v] == 0: distinct -= 1
        win = target
        ans[qi] = distinct
    for qi in 0..queries.len:
        var seen = List.filled(6, false)
        for i in queries[qi]: seen[a[i]] = true
        assert ans[qi] == count(seen[v] for v in 0..6)

    var freq2 = List.filled(6, 0)
    var distinct2 = 0
    fn add_pos(p: I):
        let v = a[p]
        if freq2[v] == 0: distinct2 += 1
        freq2[v] += 1
    fn rem_pos(p: I):
        let v = a[p]
        freq2[v] -= 1
        if freq2[v] == 0: distinct2 -= 1
    let ans2 = mo(queries, add: add_pos, remove: rem_pos, answer: _ => distinct2)
    assert ans2 == ans
