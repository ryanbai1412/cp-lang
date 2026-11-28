//! Offline segment tree over time (divide-and-conquer on life ranges).
//!
//! start: `cdq`, `TimeTree`, `offline_over`
//! example: `var pairs = 0; cdq([1, 2, 3], on_cross: (l, r) => pairs += l.len * r.len); pairs == 3`
//!
//! Turn any rollback-capable data structure (e.g. `Dsu` with `.mark()` / `.undo_to(snap)`)
//! into a dynamic structure with insertions and deletions over `[0, q)` in \(O(q \log q)\)
//! apply/undo steps.
use std.dsu.{Dsu, Mark}
use std.math.{bit_ceil}
use std.range.{Len}
use std.list.{List}
use std.ops.{max, min}
use std.optional.{Some}
use std.ranges.{Range}

/// One step in a `TimeTree` DFS traversal:
/// - `Apply(id)`: apply event `tt.event(id)` on entry to a segment-tree node.
/// - `Query(t)`: answer time slot `t` at a leaf.
/// - `Undo`: revert the most recently applied event in LIFO order on exit.
pub enum TimeStep:
    Apply(id: Int)
    Query(t: Int)
    Undo

/// Offline segment tree over time `[0, q)` holding events of type `E` active on half-open
/// ranges `[lo, hi)`.
pub struct TimeTree<E>:
    pub q: Int
    size: Len
    head: List<-1..=2_000_000_000>
    next: List<-1..=2_000_000_000>
    ev_val: List<E>

    /// An empty timeline of `q` time slots `0..q`, with no events scheduled yet.
    ///
    /// cost: \(O(q)\) time and memory
    /// traps: when `q < 0` or `q >= 2^31`
    /// example: `TimeTree<Int>(2).steps().len == 2`
    pub fn new(q: Len) -> TimeTree<E>:
        assert q >= 0, "TimeTree needs q >= 0"
        let size = bit_ceil(max(1, q))
        TimeTree(q, size, List.filled(2 * size, -1), [], [])

    fn link(mut self, u: Int, ev: E):
        unchecked:
            let idx = ev_val.len
            ev_val.push(ev)
            next.push(head[u])
            head[u] = idx

    /// Schedule `ev` to be active at every time slot in `lo..hi` (half-open). The range is
    /// stored on the \(O(\log q)\) segment-tree nodes that cover it, one copy of `ev` on each.
    /// An empty range (`lo >= hi`) schedules nothing.
    ///
    /// cost: \(O(\log q)\)
    /// traps: when `lo < 0` or `hi > q` on a non-empty range
    /// example: `var tt = TimeTree<Str>(2); tt.add_interval(0, 2, "e"); tt.steps().len == 4`
    pub fn add_interval(mut self, lo: 0.., hi: Int, ev: E):
        let ql = max(0, lo)
        let qr = min(q, hi)
        if ql >= qr: return
        var (l, r) = (ql + size, qr + size)
        while l < r:
            if l % 2 == 1:
                self.link(l, ev)
                l += 1
            if r % 2 == 1:
                r -= 1
                self.link(r, ev)
            l /= 2
            r /= 2

    /// The event carried by `TimeStep.Apply(id)`. Ids are `0, 1, 2, ...` in the order the
    /// node copies were stored by `add_interval` / `add_range`; read them from the steps, never
    /// compute them.
    ///
    /// cost: \(O(1)\)
    /// traps: when `id` is not an id that some `Apply` step carries
    /// example: `var tt = TimeTree<Str>(2); tt.add_interval(0, 2, "e"); tt.event(0) == "e"`
    pub fn event(self, id: Int) -> E = ev_val[id]

    fn emit_dfs(self, u: Int, nl: Int, nr: Int, mut out: List<TimeStep>):
        var k = 0
        var e = head[u]
        while e != -1:
            out.push(Apply(e))
            k += 1
            e = next[e]
        if nr - nl == 1:
            if nl < q: out.push(Query(nl))
        else:
            let mid = (nl + nr) / 2
            self.emit_dfs(2 * u, nl, mid, mut out)
            if mid < q: self.emit_dfs(2 * u + 1, mid, nr, mut out)
        for _ in 0..k:
            out.push(Undo)

    /// The whole depth-first walk over `0..q` as a list.
    ///
    /// Entering a node emits one `Apply(id)` per event stored there, in scheduling order; a leaf
    /// `t` then emits `Query(t)`; leaving the node emits one `Undo` per `Apply` it emitted, so
    /// undos match applies in LIFO order. Children are visited left (earlier times) before right.
    ///
    /// cost: \(O(q + m \log q)\) time and memory for `m` scheduled ranges
    /// empty: `TimeTree<Int>(0).steps() == []`
    /// order: DFS order; `Query(t)` appears once per `t`, in increasing `t`
    /// traps: never
    /// example: `TimeTree<Int>(2).steps() == [TimeStep.Query(0), TimeStep.Query(1)]`
    /// example: `var tt = TimeTree<Str>(2); tt.add_interval(0, 2, "e"); tt.steps() == [TimeStep.Apply(0), TimeStep.Query(0), TimeStep.Query(1), TimeStep.Undo]`
    /// see: `run`, which walks the same order without building the list
    pub fn steps(self) -> List<TimeStep>:
        var out: List<TimeStep> = []
        if q == 0: return out
        self.emit_dfs(1, 0, size, mut out)
        out

    fn run_node<M, R>(
        self,
        u: Int,
        nl: Int,
        nr: Int,
        mark: () -> M,
        apply: E -> (),
        answer: Int -> R,
        undo: M -> (),
        mut out: List<R>,
    ):
        unchecked:
            var snap: ?M = None
            if head[u] != -1: snap = Some(mark())
            var e = head[u]
            while e != -1:
                apply(ev_val[e])
                e = next[e]
            if nr - nl == 1:
                if nl < q: out.push(answer(nl))
            else:
                let mid = (nl + nr) / 2
                self.run_node(2 * u, nl, mid, mark, apply, answer, undo, mut out)
                if mid < q:
                    self.run_node(2 * u + 1, mid, nr, mark, apply, answer, undo, mut out)
            if let m = snap: undo(m)

    /// Walk the same order as `steps` with callbacks instead of a list.
    ///
    /// On entering a node that holds at least one event it calls `mark()` once, then `apply(ev)`
    /// for each event there in scheduling order; at each leaf `t` it calls `answer(t)`; on leaving
    /// such a node it calls `undo(m)` once with the value its `mark()` returned. Nodes with no
    /// events call neither.
    ///
    /// cost: \(O(q + m \log q)\) callback calls for `m` scheduled ranges, \(O(\log q)\) extra stack
    /// empty: `TimeTree<Int>(0).run(mark: () => 0, apply: e => (), answer: t => t, undo: m => ()) == []`
    /// order: `[answer(0), ..., answer(q - 1)]`, indexed by time
    /// traps: never (callbacks may trap)
    /// example: `TimeTree<Int>(2).run(mark: () => 0, apply: e => (), answer: t => t * 10, undo: m => ()) == [0, 10]`
    pub fn run<M, R>(
        self,
        mark: () -> M,
        apply: E -> (),
        answer: Int -> R,
        undo: M -> (),
    ) -> List<R>:
        var out: List<R> = []
        if q == 0: return out
        self.run_node(1, 0, size, mark, apply, answer, undo, mut out)
        out

    /// Schedule `ev` to be active at every time slot in `r`; `add_interval` with the range
    /// form. An empty `r` schedules nothing.
    ///
    /// cost: \(O(\log q)\)
    /// empty: `var tt = TimeTree<Str>(2); tt.add_range(1..1, "e"); tt.steps().len == 2`
    /// traps: when a non-empty `r` reaches outside `0..q`
    /// example: `var tt = TimeTree<Str>(3); tt.add_range(1..3, "e"); tt.steps().len == 7`
    pub fn add_range(mut self, r: Range<Int>, ev: E):
        if r.start >= r.end: return
        assert Int(r.start) >= 0 and Int(r.end) <= q, "the range reaches outside `0..q`"
        self.add_interval(Int(r.start), Int(r.end), ev)

    /// The steps of `steps`, yielded one at a time without building the list.
    ///
    /// cost: \(O(1)\) amortized per step, \(O(\log q)\) extra stack
    /// order: the same DFS order as `steps`
    /// traps: never
    /// example: `TimeTree<Int>(2).walk().collect() == [TimeStep.Query(0), TimeStep.Query(1)]`
    pub fn walk(self) -> yields TimeStep:
        if q == 0: return
        var su = List<Int>()
        var sl = List<Int>()
        var sr = List<Int>()
        su.push(1)
        sl.push(0)
        sr.push(size)
        while let u = su.pop():
            let nl = sl.pop()!
            let nr = sr.pop()!
            if u < 0:
                for _ in 0..nl: yield Undo
                continue
            var k = 0
            var e = head[u]
            while e != -1:
                yield Apply(e)
                k += 1
                e = next[e]
            su.push(-1)
            sl.push(k)
            sr.push(0)
            if nr - nl == 1:
                if nl < q: yield Query(nl)
            else:
                let mid = (nl + nr) / 2
                if mid < q:
                    su.push(2 * u + 1)
                    sl.push(mid)
                    sr.push(nr)
                su.push(2 * u)
                sl.push(nl)
                sr.push(mid)

/// CDQ divide and conquer over `events` in their given order.
///
/// A list of length `n >= 2` splits into `left`, the first `n / 2` events, and `right`, the rest;
/// it solves `left`, then `right`, then calls `on_cross(left, right)` once, so `on_cross` sees each
/// pair of events `i < j` in exactly one call, with `i` in `left` and `j` in `right`.
///
/// cost: \(O(n \log n)\) plus the callbacks; each level copies the halves, \(O(n)\) extra memory
/// empty: `var calls = 0; cdq(List<Int>(), on_cross: (l, r) => calls += 1); cdq([7], on_cross: (l, r) => calls += 1); calls == 0`
/// order: halves keep input order; calls go bottom-up, left subproblem before right
/// traps: never (the callback may trap)
/// example: `var pairs = 0; cdq([1, 2, 3], on_cross: (l, r) => pairs += l.len * r.len); pairs == 3`
pub fn cdq<E>(events: List<E>, on_cross: (List<E>, List<E>) -> ()):
    let n = events.len
    if n < 2: return
    var left = List<E>()
    var right = List<E>()
    for i in 0..n:
        if i < n / 2: left.push(events[i])
        else: right.push(events[i])
    cdq(left, on_cross)
    cdq(right, on_cross)
    on_cross(left, right)

/// Offline divide and conquer over time `0..q`: each item is active on the half-open slots
/// `by(item) = (lo, hi)`.
///
/// Builds a `TimeTree`, schedules every item in list order and walks it with `run`; `TimeTree(q)`
/// with `add_interval` and `run` gives the same calls.
///
/// cost: \(O(q + m \log q)\) callback calls for `m` items
/// empty: `var calls = 0; let out = offline_over(2, List<Int>(), by: e => (0, 1), mark: () => 0, apply: e => calls += 1, answer: t => t + 1, undo: m => calls += 1); out == [1, 2] and calls == 0`
/// order: `[answer(0), ..., answer(q - 1)]`, indexed by time
/// traps: when an item's non-empty range reaches outside `0..q`
/// example: `offline_over(2, [], by: e => (0, 1), mark: () => 0, apply: e => (), answer: t => t + 1, undo: m => ()) == [1, 2]`
pub fn offline_over<E, M, R>(
    q: Int,
    items: List<E>,
    by: E -> (Int, Int),
    mark: () -> M,
    apply: E -> (),
    answer: Int -> R,
    undo: M -> (),
) -> List<R>:
    var tt = TimeTree<E>.new(q)
    for ev in items:
        let (lo, hi) = by(ev)
        tt.add_interval(lo, hi, ev)
    tt.run(mark:, apply:, answer:, undo:)

@test
fn timetree_dynamic_connectivity_with_rollback_dsu():
    domain V = 0..6
    let q = 8
    var tt = TimeTree<(V, V)>(q)
    tt.add_interval(0, 5, (V(0), V(1)))
    tt.add_interval(2, 7, (V(1), V(2)))
    tt.add_interval(4, 8, (V(2), V(3)))
    tt.add_interval(6, 8, (V(0), V(3)))

    var dsu = Dsu<V>()
    var marks: List<Mark> = []
    var comps = List.filled(q, 0)
    for step in tt.steps():
        match step:
            Apply(id) =>
                let e = tt.event(id)
                marks.push(dsu.mark())
                dsu.unite(e.0, e.1)
            Query(t) =>
                comps[t] = dsu.components
            Undo =>
                if let m = marks.pop(): dsu.undo_to(m)
    assert comps == [5, 5, 4, 4, 3, 4, 3, 4]

    var dsu2 = Dsu<V>()
    let comps2 = tt.run(
        mark: () => dsu2.mark(),
        apply: e => dsu2.unite(e.0, e.1),
        answer: _ => dsu2.components,
        undo: m => dsu2.undo_to(m),
    )
    assert comps2 == comps

@test
fn offline_over_matches_timetree():
    domain V = 0..6
    let q = 8
    let items = [
        (lo: 0, hi: 5, u: V(0), v: V(1)),
        (lo: 2, hi: 7, u: V(1), v: V(2)),
        (lo: 4, hi: 8, u: V(2), v: V(3)),
        (lo: 6, hi: 8, u: V(0), v: V(3)),
    ]
    var dsu = Dsu<V>()
    let comps = offline_over(
        q,
        items,
        by: e => (e.lo, e.hi),
        mark: () => dsu.mark(),
        apply: e => dsu.unite(e.u, e.v),
        answer: _ => dsu.components,
        undo: m => dsu.undo_to(m),
    )
    assert comps == [5, 5, 4, 4, 3, 4, 3, 4]
