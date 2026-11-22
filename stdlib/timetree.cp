//! Offline segment tree over time (divide-and-conquer on life intervals).
//!
//! Turn any rollback-capable data structure (e.g. `Dsu` with `.mark()` / `.undo_to(snap)`)
//! into a dynamic structure with insertions and deletions over `[0, q)` in O(q log q)
//! apply/undo steps.
use std.dsu.{Dsu, Mark}

/// One step in a `TimeTree` DFS traversal:
/// - `Apply(id)`: apply event `tt.event(id)` on entry to a segment-tree node.
/// - `Query(t)`: answer time slot `t` at a leaf.
/// - `Undo`: revert the most recently applied event in LIFO order on exit.
pub enum TimeStep:
    Apply(id: Int)
    Query(t: Int)
    Undo

/// Offline segment tree over time `[0, q)` holding events of type `E` active on half-open
/// intervals `[lo, hi)`.
pub struct TimeTree<E>:
    q: Int
    size: Int
    head: List<-1..=2_000_000_000>
    next: List<-1..=2_000_000_000>
    ev_val: List<E>

    /// Create a timeline for `q` time slots `0..q`. O(q).
    pub fn new(q: Int) -> TimeTree<E>:
        assert q >= 0, "TimeTree needs q >= 0"
        let size = bit_ceil(max(1, q))
        TimeTree(q, size, List.filled(2 * size, -1), [], [])

    fn link(mut self, u: Int, ev: E):
        unchecked:
            let idx = ev_val.len
            ev_val.push(ev)
            next.push(head[u])
            head[u] = idx

    /// Schedule event `ev` to be active across the half-open time interval `[lo, hi)`. O(log q).
    pub fn add_interval(mut self, lo: Int, hi: Int, ev: E):
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

    /// Retrieve the event referenced by `Apply(id)`. O(1).
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

    /// Return the full DFS traversal of `Apply`, `Query`, and `Undo` steps over `0..q`.
    /// Total length is O(q + m log q) for `m` added intervals.
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
            let snap = mark()
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
            undo(snap)

    /// Walk the segment tree over `0..q` with zero step-list allocation, returning
    /// `[answer(0), ..., answer(q - 1)]`. Calls `mark()` once on entering each non-empty
    /// node, `apply(ev)` for each event active on that node, `answer(t)` at every leaf
    /// `t in 0..q` in ascending order, and `undo(snap)` on exit. O(q + m log q).
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

/// Offline divide-and-conquer over time `0..q` for `items` active on half-open
/// intervals `by(ev) = (lo, hi)`. Builds a `TimeTree<E>`, walks it with
/// `mark`/`apply`/`answer`/`undo` callbacks (zero step-list allocation), and returns
/// `[answer(0), ..., answer(q - 1)]`. O(q + m log q).
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
            Apply(id):
                let e = tt.event(id)
                marks.push(dsu.mark())
                dsu.unite(e.0, e.1)
            Query(t):
                comps[t] = dsu.components
            Undo:
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
