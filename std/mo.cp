//! Mo's algorithm: offline ordering and window stepping over `Range<D>`.
use std.range.{ends}
use std.math.{isqrt}

/// The order in which to visit `queries` so a sliding window moves its endpoints
/// \(O(n \sqrt q)\) times, as a permutation of `0..queries.len`.
///
/// Let `n` be the greatest end over all `q` queries and `b = max(1, n / isqrt(q))`. Query `i`
/// falls in block `queries[i].start / b`.
/// Blocks are visited in increasing order; within an even block queries go by increasing end,
/// within an odd block by decreasing end (the snake that saves the return sweep).
///
/// cost: \(O(q \log q)\)
/// empty: `mo_order(List<Range<Int>>()) == []`
/// ties: `mo_order([2..4, 0..1, 2..4]) == [1, 0, 2]`
/// order: increasing block, then end increasing in even blocks and decreasing in odd ones
/// traps: never
/// example: `mo_order([0..3, 2..4, 0..1, 3..4]) == [2, 0, 1, 3]`
/// example: `mo_order([5..5]) == [0]`
pub fn mo_order<D>(queries: List<Range<D>>) -> List<Int>:
    let q = queries.len
    if q == 0: return []
    let hi = queries.map(qr => qr.ends.1).max()!
    let b = max(1, hi / max(1, isqrt(q)))
    var pairs: List<(Int, Int)> = []
    unchecked:
        let span = hi + 1
        for i in 0..q:
            let (l, r) = queries[i].ends
            let blk = l / b
            let k = blk * span + (if blk % 2 == 0: r else: hi - r)
            pairs.push((k, i))
        pairs.sort()
        return [p.1 for p in pairs]

/// The positions that enter (`entering: true`) or leave (`entering: false`) a window as it
/// moves from `from` to `to`, in the order a caller should apply them.
///
/// The window grows before it shrinks, so it is never inverted even when `from` and `to` are
/// disjoint: first the left end moves left (entering positions in decreasing order), then the
/// right end moves right (entering, increasing), then the left end moves right (leaving,
/// increasing), then the right end moves left (leaving, decreasing).
///
/// cost: \(O(\lvert Δ\mathit{start} \rvert + \lvert Δ\mathit{end} \rvert)\)
/// empty: `mo_steps(1..3, 1..3) == []`
/// order: left growth, right growth, left shrink, right shrink, each outward-to-inward as above
/// traps: never
/// example: `mo_steps(1..3, 0..2) == [(pos: 0, entering: true), (pos: 2, entering: false)]`
/// example: `mo_steps(0..1, 2..3).len == 4`
/// example: `mo_steps(0..1, 2..3) == [(pos: 1, entering: true), (pos: 2, entering: true), (pos: 0, entering: false), (pos: 1, entering: false)]`
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

/// Answers offline range queries by moving one window through them in
/// [`mo_order`](mo_order) and calling `add(pos)` / `remove(pos)` for each step of
/// [`mo_steps`](mo_steps). When `answer(qi)` runs, the active window is exactly `queries[qi]`.
///
/// The window starts empty at the start of the first visited query. Results come back in the
/// original query order, so `result[i]` answers `queries[i]`. Allocates only the order and the
/// results.
///
/// cost: \(O((n + q) \sqrt n)\) calls to `add` and `remove`, `q` calls to `answer`, plus \(O(q \log q)\)
/// empty: `var calls = 0; let r = mo(List<Range<Int>>(), add: _ => calls += 1, remove: _ => calls += 1, answer: _ => calls); (r.len, calls) == (0, 0)`
/// order: `result[i]` answers `queries[i]`
/// traps: never; traps raised by the callbacks propagate
/// example: `let a = [1, 2, 3]; var s = 0; mo([0..2, 1..3], add: i => s += a[i], remove: i => s -= a[i], answer: _ => s) == [3, 5]`
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

/// The order in which to visit `queries`, by Hilbert-curve key: typically 1.5-2x fewer endpoint
/// moves than [`mo_order`](mo_order) on dense queries.
///
/// Let `lo` be the least start and `n` the distance from `lo` to the greatest end, and `k` the
/// least integer with `2^k > n`. Query `i` is the point `(x, y) = (start - lo, end - lo)` on the
/// `2^k × 2^k` grid and its key is that point's index on the standard Hilbert curve (the
/// `xy2d` construction that starts at `(0, 0)`, visits `(0, 1)` second and ends at `(2^k - 1, 0)`).
///
/// cost: \(O(q \log q + q \log n)\)
/// empty: `mo_order_hilbert(List<Range<Int>>()) == []`
/// ties: `mo_order_hilbert([0..2, 1..1, 0..2]) == [1, 0, 2]`
/// order: increasing Hilbert key
/// traps: never
/// example: `mo_order_hilbert([1..1, 0..1, 0..0]) == [2, 1, 0]`
/// example: `mo_order_hilbert([0..2, 1..3, 2..2]).sorted() == [0, 1, 2]`
pub fn mo_order_hilbert<D>(queries: List<Range<D>>) -> List<Int>:
    let q = queries.len
    if q == 0: return []
    var lo = queries[0].ends.0
    var hi = queries[0].ends.1
    for i in 1..q:
        let (l, r) = queries[i].ends
        lo = min(lo, l)
        hi = max(hi, r)
    let n = hi - lo
    var side = 1
    while side <= n: side *= 2
    var keyed: List<(Int, Int)> = []
    for i in 0..q:
        let (l, r) = queries[i].ends
        var x = l - lo
        var y = r - lo
        var d = 0
        var s = side / 2
        while s > 0:
            let rx = (x / s) % 2
            let ry = (y / s) % 2
            d += s * s * (if ry == 0: 3 * rx else: 1 + rx)
            if ry == 0:
                if rx == 1:
                    x = side - 1 - x
                    y = side - 1 - y
                let t = x
                x = y
                y = t
            s /= 2
        keyed.push((d, i))
    keyed.sort()
    [p.1 for p in keyed]

/// Mo's algorithm for structures that can add and undo but not remove (DSU, running max, convex
/// hull): answers every query with `add` calls only, restoring state with `mark` / `undo` instead
/// of `remove`.
///
/// When `answer(qi)` runs, the state is exactly the result of adding the positions of `queries[qi]`
/// to a reset structure.
///
/// Blocks are as in [`mo_order`](mo_order). A query that lies inside one block is answered on
/// its own: `m = mark()`, add its positions, answer, `undo(m)`. The other queries of a block go
/// by increasing end, the right part growing from the block's end with `add`, and each query's
/// left part is added between a `mark()` and the matching `undo`. Entering a block whose predecessor grew a right part calls `reset()` first,
/// so in-block queries never see the previous block's positions. Results come back in the
/// original query order.
///
/// cost: \(O((n + q) \sqrt n)\) calls to `add`, no removals, at most one `reset` per block
/// empty: `domain P = 0..3; var calls = 0; let r = mo_rollback(List<Range<P>>(), mark: () => calls, add: _ => calls += 1, answer: _ => calls, undo: _ => calls += 1, reset: () => calls += 1); (r.len, calls) == (0, 0)`
/// order: `result[i]` answers `queries[i]`
/// traps: never; traps raised by the callbacks propagate
///
/// The blocks need the size of the position domain, so the queries range over a declared one:
///
/// ```cp
/// domain P = 0..3
/// let a: [P]Int = [1, 2, 3]
/// var s = 0
/// let qs: List<Range<P>> = [P(0)..P(2), P(1)..]
/// mo_rollback(
///     qs,
///     mark: () => s,
///     add: i => s += a[i],
///     answer: _ => s,
///     undo: m => s = m,
///     reset: () => s = 0,
/// ) == [3, 5]
/// ```
pub fn mo_rollback<D, M, R>(
    queries: List<Range<D>>,
    mark: () -> M,
    add: D -> (),
    answer: Int -> R,
    undo: M -> (),
    reset: () -> (),
) -> List<R>:
    let q = queries.len
    if q == 0: return []
    let b = max(1, D.count / max(1, isqrt(q)))
    var keyed: List<(Int, Int)> = []
    for i in 0..q:
        let (l, r) = queries[i].ends
        let blk = l / b
        let inside = r <= min((blk + 1) * b, D.count)
        keyed.push((blk * (D.count + 2) + (if inside: 0 else: r + 1), i))
    keyed.sort()
    var res: List<?R> = [None for _ in 0..q]
    var cur: ?Int = None
    var dirty = false
    var rr = 0
    for (_, i) in keyed:
        let (l, r) = queries[i].ends
        let blk = l / b
        let be = min((blk + 1) * b, D.count)
        if cur != Some(blk):
            cur = blk
            if dirty:
                reset()
                dirty = false
            rr = be
        if r <= be:
            let m = mark()
            for p in l..r: add(D.nth(p))
            res[i] = Some(answer(i))
            undo(m)
        else:
            dirty = true
            while rr < r:
                add(D.nth(rr))
                rr += 1
            let m = mark()
            var p = be
            while p > l:
                p -= 1
                add(D.nth(p))
            res[i] = Some(answer(i))
            undo(m)
    [x! for x in res]

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
/// cost: \(O(n^{5/3} + q \log q)\) window moves and updates for `q ~ n` queries
/// empty: `var calls = 0; let r = mo_with_updates(List<(range: Range<Int>, time: Int)>(), List<Int>(), add: _ => calls += 1, remove: _ => calls += 1, apply_update: (_, _) => calls += 1, undo_update: (_, _) => calls += 1, answer: _ => calls); (r.len, calls) == (0, 0)`
/// ties: `var seen: List<Int> = []; let qs = [(range: 0..2, time: 0), (range: 0..2, time: 0)]; mo_with_updates(qs, List<Int>(), add: _ => (), remove: _ => (), apply_update: (_, _) => (), undo_update: (_, _) => (), answer: qi => seen.push(qi)); seen == [0, 1]`
/// order: `result[i]` answers `queries[i]`
/// traps: when a `time` is outside `0..=updates.len`
/// example: `var a = [1, 2, 3]; let qs = [(range: 0..2, time: 1), (range: 1..3, time: 0)]; mo_with_updates(qs, [(pos: 1, by: 10)], add: _ => (), remove: _ => (), apply_update: (u, _) => a[u.pos] += u.by, undo_update: (u, _) => a[u.pos] -= u.by, answer: qi => a[qs[qi].range].sum()) == [13, 5]`
pub fn mo_with_updates<D, U, R>(
    queries: List<(range: Range<D>, time: Int)>,
    updates: List<U>,
    add: D -> (),
    remove: D -> (),
    apply_update: (U, Range<D>) -> (),
    undo_update: (U, Range<D>) -> (),
    answer: Int -> R,
) -> List<R>:
    let q = queries.len
    if q == 0: return []
    var lo = queries[0].range.ends.0
    var hi = queries[0].range.ends.1
    for i in 0..q:
        let (l, r) = queries[i].range.ends
        let t = queries[i].time
        assert 0 <= t <= updates.len, "a `time` is outside `0..=updates.len`"
        lo = min(lo, l)
        hi = max(hi, r)
    let n = hi - lo
    var b = 1
    while (b + 1) * (b + 1) * (b + 1) <= n * n: b += 1
    var keyed: List<(Int, Int, Int, Int)> = []
    for i in 0..q:
        let (l, r) = queries[i].range.ends
        keyed.push(((l - lo) / b, (r - lo) / b, queries[i].time, i))
    keyed.sort()
    var res: List<?R> = [None for _ in 0..q]
    var l = queries[keyed[0].3].range.ends.0
    var r = l
    var t = 0
    for (_, _, time, qi) in keyed:
        let (nl, nr) = queries[qi].range.ends
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
        let w = queries[qi].range
        while t < time:
            apply_update(updates[t], w)
            t += 1
        while t > time:
            t -= 1
            undo_update(updates[t], w)
        res[qi] = Some(answer(qi))
    [x! for x in res]

@test
fn mo_order_and_steps_answer_distinct_counts():
    domain I = 1..=20
    let a = [I] _ => rand(0, 5)
    var queries: List<Range<I>> = []
    for _ in 0..40:
        let l = rand(1, 20)
        let r = rand(l, 20)
        queries.push(I(l)..=I(r))
    var freq = List.filled(6, 0)
    var distinct = 0
    var win: Range<I> = I(1)..I(1)
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
        let want = count(seen[v] for v in 0..6)
        assert ans[qi] == want
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
