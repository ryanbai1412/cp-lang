//! Union-find that can undo: for offline dynamic connectivity and
//! divide-and-conquer over time, where edges are added on the way down a
//! recursion and removed on the way back up.

/// Disjoint sets with `undo`. No path compression (it cannot be undone),
/// so `find` is O(log n) by union by size; `unite` and `undo` are O(log n) and O(1).
pub struct RollbackDsu<V>:
    up: [V]V
    sz: [V]Int
    log: List<?(child: V, root: V)>
    parts: Int

    pub fn new() -> RollbackDsu<V>:
        RollbackDsu([V] v => v, [V] v => 1, [], V.K)

    pub fn find(self, v: V) -> V:
        var x = v
        while up[x] != x: x = up[x]
        x

    pub fn same(self, a: V, b: V) -> Bool = self.find(a) == self.find(b)

    /// The number of sets.
    pub @get fn count(self) -> Int = parts

    /// The number of `unite` calls that can still be undone.
    pub @get fn depth(self) -> Int = log.len

    /// Merge the sets of `a` and `b`. Every call is recorded, merging or not,
    /// so each `unite` pairs with exactly one `undo`.
    pub fn unite(mut self, a: V, b: V) -> Bool:
        var (x, y) = (self.find(a), self.find(b))
        if x == y:
            log.push(none)
            return false
        if sz[x] < sz[y]: swap(mut x, mut y)
        up[y] = x
        sz[x] += sz[y]
        parts -= 1
        log.push((child: y, root: x))
        true

    /// Take back the most recent `unite` that has not been undone.
    pub fn undo(mut self):
        if let step = log.pop():
            up[step.child] = step.child
            sz[step.root] -= sz[step.child]
            parts += 1

    /// Undo until only `depth` calls remain.
    pub fn rewind(mut self, depth: Int):
        while log.len > depth: self.undo()

@test
fn undo_restores_every_earlier_state():
    domain V = 0..9
    var d = RollbackDsu<V>()
    var history = List<[V]Int>()
    var pairs = List<(V, V)>()
    for _ in 0..30:
        history.push([V] v => d.find(v).to_idx)
        let (a, b) = (V.nth(rand(0, 8)), V.nth(rand(0, 8)))
        pairs.push((a, b))
        d.unite(a, b)
    assert d.depth == 30
    for step in (0..30).rev:
        d.undo()
        for v in V: assert d.find(v).to_idx == history[step][v]
    assert d.count == 9 and d.depth == 0

@test
fn rewind_to_a_checkpoint():
    domain V = 1..=5
    var d = RollbackDsu<V>()
    d.unite(V(1), V(2))
    let mark = d.depth
    d.unite(V(2), V(3))
    d.unite(V(1), V(3))
    d.unite(V(4), V(5))
    assert d.same(V(1), V(3)) and d.count == 2
    d.rewind(mark)
    assert d.same(V(1), V(2)) and not d.same(V(1), V(3)) and not d.same(V(4), V(5)) and d.count == 4
