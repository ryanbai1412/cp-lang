//! Union-find that also tracks a side for every vertex: add edges one at a time and learn
//! the moment the graph stops being two-colourable.

/// Components of `V` with a parity per vertex. `add_edge` is O(log n) (union by size, no
/// path compression, so `mark` / `undo_to` can take edges back).
pub struct BipartiteDsu<V>:
    up: [V]V
    size: [V]Int
    /// Is the vertex on the other side from `up` of it?
    flip: [V]Bool
    /// One entry per edge that changed something, oldest first: the root it hung under
    /// another root, or (with `false`) an edge that closed an odd cycle.
    log: List<(V, Bool)>
    odd: Int

    pub fn new() -> BipartiteDsu<V>:
        BipartiteDsu([V] v => v, [V] v => 1, [V] v => false, [], 0)

    /// The root of `v`'s component, and whether `v` is on the other side from it.
    pub fn find(self, v: V) -> (V, Bool):
        var (x, side) = (v, false)
        while up[x] != x:
            side = side != flip[x]
            x = up[x]
        (x, side)

    /// Join `a` and `b` by an edge: they must get different sides.
    pub fn add_edge(mut self, a: V, b: V):
        let (ra, sa) = self.find(a)
        let (rb, sb) = self.find(b)
        if ra == rb:
            if sa == sb:
                odd += 1
                log.push((ra, false))
            return
        let (big, small) = if size[ra] >= size[rb]: (ra, rb) else: (rb, ra)
        up[small] = big
        flip[small] = sa == sb
        size[big] += size[small]
        log.push((small, true))

    /// Is every component still two-colourable?
    pub @get fn bipartite(self) -> Bool = odd == 0

    /// Are `a` and `b` joined, and if so are they on the same side?
    pub fn same_side(self, a: V, b: V) -> ?Bool:
        let (ra, sa) = self.find(a)
        let (rb, sb) = self.find(b)
        if ra != rb: none else: sa == sb

    /// A point in time to return to with `undo_to`.
    pub fn mark(self) -> Int = log.len

    /// Take back every edge added since `mark` returned `snap`.
    pub fn undo_to(mut self, snap: Int):
        while log.len > snap:
            let (small, joined) = log.pop()!
            if not joined:
                odd -= 1
            else:
                size[up[small]] -= size[small]
                up[small] = small
                flip[small] = false

@test
fn agrees_with_colouring_from_scratch():
    domain V = 0..8
    for _ in 0..40:
        var d = BipartiteDsu<V>()
        var edges: List<(V, V)> = []
        for _ in 0..rand(0, 10):
            let e = (V.nth(rand(0, 7)), V.nth(rand(0, 7)))
            edges.push(e)
            d.add_edge(e.0, e.1)
            // brute force: try every colouring
            var ok = false
            for mask in 0..256:
                if all((mask >> e.0.to_idx) % 2 != (mask >> e.1.to_idx) % 2 for e in edges): ok = true
            assert d.bipartite == ok

@test
fn edges_can_be_taken_back():
    domain V = 1..=4
    var d = BipartiteDsu<V>()
    d.add_edge(V(1), V(2))
    d.add_edge(V(2), V(3))
    assert d.same_side(V(1), V(3)) == true and d.same_side(V(1), V(4)) == none
    let snap = d.mark()
    d.add_edge(V(3), V(1))
    d.add_edge(V(3), V(4))
    assert not d.bipartite and d.same_side(V(3), V(4)) == false
    d.undo_to(snap)
    assert d.bipartite and d.same_side(V(1), V(4)) == none and d.same_side(V(1), V(2)) == false
