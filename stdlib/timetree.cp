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
    events: List<E>
    buckets: List<List<Int>>

    /// Create a timeline for `q` time slots `0..q`. O(q).
    pub fn new(q: Int) -> TimeTree<E>:
        assert q >= 0, "TimeTree needs q >= 0"
        let sz = max(1, 4 * max(1, q))
        TimeTree(q, [], [[] for _ in 0..sz])

    fn add_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, id: Int):
        if qr <= nl or nr <= ql: return
        if ql <= nl and nr <= qr:
            buckets[u].push(id)
            return
        let mid = (nl + nr) / 2
        self.add_rec(2 * u, nl, mid, ql, qr, id)
        self.add_rec(2 * u + 1, mid, nr, ql, qr, id)

    /// Schedule event `ev` to be active across the half-open time interval `[lo, hi)`. O(log q).
    pub fn add_interval(mut self, lo: Int, hi: Int, ev: E):
        let ql = max(0, lo)
        let qr = min(q, hi)
        if ql >= qr: return
        let id = events.len
        events.push(ev)
        self.add_rec(1, 0, q, ql, qr, id)

    /// Retrieve the event referenced by `Apply(id)`. O(1).
    pub fn event(self, id: Int) -> E = events[id]

    fn emit_dfs(self, u: Int, nl: Int, nr: Int, mut out: List<TimeStep>):
        let k = buckets[u].len
        for id in buckets[u]:
            out.push(Apply(id))
        if nr - nl == 1:
            out.push(Query(nl))
        else:
            let mid = (nl + nr) / 2
            self.emit_dfs(2 * u, nl, mid, mut out)
            self.emit_dfs(2 * u + 1, mid, nr, mut out)
        for _ in 0..k:
            out.push(Undo)

    /// Return the full DFS traversal of `Apply`, `Query`, and `Undo` steps over `0..q`.
    /// Total length is O(q + m log q) for `m` added intervals.
    pub fn steps(self) -> List<TimeStep>:
        var out: List<TimeStep> = []
        if q == 0: return out
        self.emit_dfs(1, 0, q, mut out)
        out

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
