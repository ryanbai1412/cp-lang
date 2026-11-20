//! Maximum flow.

/// A handle to one arc of a `FlowGraph`. Handles from `add` stay valid as arcs are added.
pub struct Arc(id: Int):
    /// The paired arc in the opposite direction. This is what `e ^ 1` means in C++.
    pub @get fn rev(self) -> Arc = Arc(id ^ 1)

/// A flow network over the vertices `V`, solved with Dinic's algorithm.
/// `add` is O(1). `max_flow` is O(V^2 E) in general, O(E sqrt(V)) on unit
/// networks (bipartite matching). Memory is two arcs per `add`.
pub struct FlowGraph<V>:
    head: [V]List<Arc>
    to: List<V>
    left: List<Int>
    cap0: List<Int>

    pub fn new() -> FlowGraph<V>:
        FlowGraph([V] v => [], [], [], [])

    /// Add an arc `u -> v` with capacity `cap` (and its reverse with capacity 0).
    pub fn add(mut self, u: V, v: V, cap: Int) -> Arc:
        assert cap >= 0
        let a = Arc(to.len)
        head[u].push(a)
        head[v].push(a.rev)
        to.push(v)
        to.push(u)
        left.push(cap)
        left.push(0)
        cap0.push(cap)
        cap0.push(0)
        a

    /// Add an undirected edge: `cap` in both directions.
    pub fn add_both(mut self, u: V, v: V, cap: Int) -> Arc:
        let a = self.add(u, v, cap)
        left[a.rev.id] = cap
        cap0[a.rev.id] = cap
        a

    /// The flow currently sent along `a` (negative on the reverse arc of a used arc).
    pub fn flow(self, a: Arc) -> Int = cap0[a.id] - left[a.id]

    /// The capacity `a` was added with.
    pub fn cap(self, a: Arc) -> Int = cap0[a.id]

    /// The vertex `a` points to.
    pub fn target(self, a: Arc) -> V = to[a.id]

    /// Push as much as possible from `s` to `t`, on top of the flow already there.
    /// Returns the amount added.
    pub fn max_flow(mut self, s: V, t: V) -> Int:
        assert s != t
        var total = 0
        while true:
            let level = self.levels(s)
            if level[t] == none: break
            var next: [V]Int = 0
            while true:
                let pushed = self.augment(s, t, level, mut next)
                if pushed == 0: break
                total += pushed
        total

    /// The source side of a minimum cut, after `max_flow(s, t)`.
    pub fn min_cut(self, s: V) -> [V]Bool:
        let level = self.levels(s)
        [V] v => level[v] != none

    fn levels(self, s: V) -> [V]?Int:
        var level: [V]?Int = none
        level[s] = 0
        var queue = [s]
        var at = 0
        while at < queue.len:
            let u = queue[at]
            at += 1
            for a in head[u]:
                let v = to[a.id]
                if left[a.id] > 0 and level[v] == none:
                    level[v] = level[u]! + 1
                    queue.push(v)
        level

    /// One augmenting path in the level graph, found without recursion.
    fn augment(mut self, s: V, t: V, level: [V]?Int, mut next: [V]Int) -> Int:
        var path = List<Arc>()
        var u = s
        while u != t:
            if next[u] < head[u].len:
                let a = head[u][next[u]]
                let v = to[a.id]
                if left[a.id] > 0 and level[v] == level[u]! + 1:
                    path.push(a)
                    u = v
                else:
                    next[u] += 1
            else:
                if path.is_empty: return 0
                let a = path.pop()
                u = to[a.rev.id]
                next[u] += 1
        let amount = min(left[a.id] for a in path)
        for a in path:
            left[a.id] -= amount
            left[a.rev.id] += amount
        amount

@test
fn textbook_network():
    domain V = 0..6
    var g = FlowGraph<V>()
    let arcs = [g.add(V(u), V(v), c) for (u, v, c) in [(0, 1, 16), (0, 2, 13), (1, 2, 10), (2, 1, 4), (1, 3, 12), (3, 2, 9), (2, 4, 14), (4, 3, 7), (3, 5, 20), (4, 5, 4)]]
    assert g.max_flow(V(0), V(5)) == 23
    // conservation at inner vertices, capacity on every arc
    var net: [V]Int = 0
    for a in arcs:
        assert 0 <= g.flow(a) <= g.cap(a)
        assert g.flow(a.rev) == -g.flow(a)
        net[g.target(a)] += g.flow(a)
        net[g.target(a.rev)] -= g.flow(a)
    assert [net[v] for v in V] == [-23, 0, 0, 0, 0, 23]
    assert g.max_flow(V(0), V(5)) == 0

@test
fn min_cut_equals_max_flow_on_random_networks():
    domain V = 0..6
    for _ in 0..30:
        var g = FlowGraph<V>()
        let es = [(V.nth(rand(0, 5)), V.nth(rand(0, 5)), rand(0, 8)) for _ in 0..12]
        let arcs = [g.add(u, v, c) for (u, v, c) in es]
        let f = g.max_flow(V.first, V.last)
        let side = g.min_cut(V.first)
        assert side[V.first] and not side[V.last]
        assert f == sum(c for (u, v, c) in es if side[u] and not side[v])
        // brute force: the smallest cut over all 16 source sides
        var best = Int.MAX
        for mask in 0..16:
            let inside = [V] v => v == V.first or (v != V.last and ((mask >> (v.to_idx - 1)) & 1) == 1)
            best min= sum(c for (u, v, c) in es if inside[u] and not inside[v])
        assert f == best

@test
fn bipartite_matching_with_undirected_and_unit_arcs():
    domain V = 0..8
    var g = FlowGraph<V>()
    // 0 = source, 1..=3 left, 4..=6 right, 7 = sink
    for l in 1..=3: g.add(V(0), V(l), 1)
    for r in 4..=6: g.add(V(r), V(7), 1)
    let pairs = [g.add(V(l), V(r), 1) for (l, r) in [(1, 4), (2, 4), (3, 4), (3, 5)]]
    assert g.max_flow(V(0), V(7)) == 2
    assert sum(g.flow(a) for a in pairs) == 2
    var h = FlowGraph<V>()
    h.add_both(V(0), V(1), 5)
    h.add_both(V(1), V(2), 3)
    assert h.max_flow(V(2), V(0)) == 3
