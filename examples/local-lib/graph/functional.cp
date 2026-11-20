//! Functional graphs: every vertex has exactly one way out. Each component is one ring
//! with trees hanging off it and pointing inward; this module splits the graph into those
//! parts once, and then answers "where am I after k steps" and "how far is b from a".

/// The shape of the graph `v -> next[v]` over the vertices `V`. Building is
/// O(n log n) time and memory (the jump table); `jump` and `dist` are O(log n) for any
/// `k`, however large; every other query is O(1).
pub struct Functional<V>:
    /// `up[j][v]` is the vertex `2 ** j` steps after `v`, for `2 ** j < n`.
    up: List<[V]V>
    /// The ring `v` drains into, as a position in `cycles`.
    ring: [V]Int
    /// Steps from `v` to its ring: 0 exactly on a ring.
    depth: [V]Int
    /// The place on its ring of the first ring vertex `v` reaches.
    pos: [V]Int
    cycles: List<List<V>>
    order: List<V>

    pub fn new(next: [V]V) -> Functional<V>:
        var indeg: [V]Int = 0
        for v in V: indeg[next[v]] += 1
        // peel the vertices nobody points at, over and over: what is left are the rings
        var order = [v for v in V if indeg[v] == 0]
        var head = 0
        while head < order.len:
            let to = next[order[head]]
            head += 1
            indeg[to] -= 1
            if indeg[to] == 0: order.push(to)
        var ring: [V]Int = -1
        var depth: [V]Int = 0
        var pos: [V]Int = 0
        var cycles: List<List<V>> = []
        for s in V:
            if indeg[s] == 0 or ring[s] != -1: continue
            var (cyc, v) = ([s], next[s])
            ring[s] = cycles.len
            while v != s:
                (ring[v], pos[v]) = (cycles.len, cyc.len)
                cyc.push(v)
                v = next[v]
            cycles.push(cyc)
        // a tree vertex is peeled before the vertex it points at, so walk backwards
        for v in order.rev:
            let to = next[v]
            (ring[v], depth[v], pos[v]) = (ring[to], depth[to] + 1, pos[to])
        var up = [next]
        while (2 << (up.len - 1)) < V.K:
            let last = up.last!
            up.push([V] v => last[last[v]])
        Functional(up, ring, depth, pos, cycles, order)

    /// Every ring, each listed in the order the graph walks it, starting at its
    /// smallest vertex. A self-loop is a ring of one.
    pub @get fn rings(self) -> List<List<V>> = cycles

    /// The vertices that are on no ring, each before the vertex it points at: fold a
    /// tree into its ring by walking this list.
    pub @get fn peel_order(self) -> List<V> = order

    pub fn on_ring(self, v: V) -> Bool = depth[v] == 0

    /// The ring `v` ends up on, as a position in `rings`.
    pub fn ring_of(self, v: V) -> Int = ring[v]

    /// How many steps `v` takes to reach its ring; 0 on a ring.
    pub fn steps_to_ring(self, v: V) -> Int = depth[v]

    /// The first ring vertex `v` reaches; `v` itself on a ring.
    pub fn root(self, v: V) -> V = cycles[ring[v]][pos[v]]

    /// The vertex `k >= 0` steps after `v`.
    pub fn jump(self, v: V, k: Int) -> V:
        assert k >= 0
        if k >= depth[v]:
            let cyc = cycles[ring[v]]
            return cyc[(pos[v] + k - depth[v]) % cyc.len]
        var (at, j) = (v, 0)
        while (k >> j) > 0:
            if (k >> j) % 2 == 1: at = up[j][at]
            j += 1
        at

    /// The fewest steps from `a` to `b`, or `none` if `a` never reaches `b`.
    pub fn dist(self, a: V, b: V) -> ?Int:
        if ring[a] != ring[b]: return none
        if depth[b] == 0:
            return depth[a] + (pos[b] - pos[a]) % cycles[ring[a]].len
        let d = depth[a] - depth[b]
        if d >= 0 and self.jump(a, d) == b: d else: none

    /// How many different vertices the walk from `v` visits, `v` included.
    pub fn orbit(self, v: V) -> Int = depth[v] + cycles[ring[v]].len

    /// Walk forward from `v` up to `max_steps` while `pred` stays true (`pred` must be true on a
    /// prefix of the walk from `v` and false afterwards). Returns `(u, steps)` for the farthest
    /// vertex `u = self.jump(v, steps)` with `pred(u)`, or `none` if `not pred(v)`. O(log^2 n) / O(log n).
    pub fn jump_while(self, v: V, pred: V -> Bool, max_steps: Int = V.K) -> ?(V, Int):
        assert max_steps >= 0
        if not pred(v): return none
        var (lo, hi) = (0, max_steps)
        while lo < hi:
            let mid = (lo + hi + 1) / 2
            if pred(self.jump(v, mid)): lo = mid else: hi = mid - 1
        (self.jump(v, lo), lo)

/// The walk from `v`, one step at a time, until it repeats a vertex.
fn walk<V>(next: [V]V, v: V) -> List<V>:
    var (path, at) = ([], v)
    while not path.contains(at):
        path.push(at)
        at = next[at]
    path

@test
fn jumps_and_distances_match_walking():
    for round in 0..60:
        let n = rand(1, 14)
        domain V = 0..n
        // a few rounds force long tails by pointing mostly forward
        let next = [V] v => if round % 3 == 0: V(
            min(v.to_idx + rand(1, 2), n - 1)
        ) else: V(rand(0, n - 1))
        let g = Functional<V>(next)
        for a in V:
            let path = walk(next, a)
            assert g.orbit(a) == path.len
            for k in 0..3 * n:
                var at = a
                for _ in 0..k: at = next[at]
                assert g.jump(a, k) == at
            for b in V:
                let i = path.find(b)
                assert g.dist(a, b) == i
            // far beyond any table: only the ring matters
            let far = rand(10 ** 12, 10 ** 18)
            let len = g.rings[g.ring_of(a)].len
            assert g.jump(a, far) == g.jump(
                a,
                g.steps_to_ring(a) + (far - g.steps_to_ring(a)) % len,
            )

@test
fn rings_and_trees_partition_the_graph():
    for _ in 0..60:
        let n = rand(1, 16)
        domain V = 0..n
        let next = [V] v => V(rand(0, n - 1))
        let g = Functional<V>(next)
        var seen: [V]Int = 0
        for id in 0..g.rings.len:
            let ring = g.rings[id]
            assert ring[0] == min(ring)
            for i in 0..ring.len:
                assert next[ring[i]] == ring[(i + 1) % ring.len]
                assert g.on_ring(ring[i]) and g.ring_of(ring[i]) == id and g.root(ring[i]) == ring[i]
                seen[ring[i]] += 1
        var done: [V]Bool = false
        for v in g.peel_order:
            // everything pointing at v was peeled before v
            assert not g.on_ring(v) and all(done[u] for u in V if next[u] == v)
            done[v] = true
            seen[v] += 1
        assert all(seen[v] == 1 for v in V)
        for v in V:
            let path = walk(next, v)
            let k = g.steps_to_ring(v)
            assert g.on_ring(path[k]) and g.root(v) == path[k] and (
                k == 0 or not g.on_ring(path[k - 1])
            )
            assert g.ring_of(v) == g.ring_of(next[v])

@test
fn how_many_vertices_drain_into_each_ring_vertex():
    // fold every tree into its ring with one pass over the peel order
    //   0 -> 1 -> 2 -> 0,   3 -> 0,   4 -> 3,   5 -> 3,   6 -> 6
    domain V = 0..7
    let to = [1, 2, 0, 0, 3, 3, 6]
    let next = [V] v => V(to[v.to_idx])
    let g = Functional<V>(next)
    var load: [V]Int = 1
    for v in g.peel_order: load[next[v]] += load[v]
    assert g.rings == [[V(0), V(1), V(2)], [V(6)]]
    assert load[V(0)] == 4 and load[V(3)] == 3 and load[V(6)] == 1
    assert g.jump(V(4), 10 ** 18) == V(2) and g.dist(V(4), V(2)) == 4 and g.dist(V(2), V(4)) == none
    assert g.dist(V(4), V(5)) == none and g.dist(V(0), V(6)) == none and g.orbit(V(5)) == 5

@test
fn jump_while_matches_walking():
    for _ in 0..40:
        let n = rand(2, 14)
        domain V = 0..n
        let next = [V] v => V(rand(0, n - 1))
        let g = Functional<V>(next)
        for v in V:
            let thresh = rand(0, 4)
            let limit = rand(0, n + 3)
            let got = g.jump_while(v, u => g.steps_to_ring(u) > thresh, max_steps: limit)
            var want: ?(V, Int) = none
            var cur = v
            for s in 0..=limit:
                if g.steps_to_ring(cur) > thresh: want = (cur, s)
                else: break
                cur = next[cur]
            assert got == want
