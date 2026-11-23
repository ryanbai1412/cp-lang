//! Flows, cuts and matchings: Dinic's max flow and min cut, min-cost flow, maximum
//! closure, bipartite and general matching (weighted too), assignment, bounded flows,
//! Gomory-Hu and global min cut.
//!
//! Two networks, `FlowGraph` and `CostFlow`, share `Arc` and the accessors `flow`, `cap`,
//! `source`, `target`. Everything whose input is a table or an edge list is a function
//! over the caller's domains: a matching is `[L]?R`, a bounded flow is `[E]Int`.
use std.graph.{Next}
use std.heap.{MinHeap}
use std.seq.{collect, next_permutation}
use std.math.{bit}

/// A handle to one arc of a `FlowGraph`. Handles from `add` stay valid as arcs are added.
pub struct Arc(pub id: 0..2_147_483_648):
    /// The paired arc in the opposite direction, what `e ^ 1` means in C++. Every `add` makes an
    /// arc and its reverse together, so `a.rev.rev == a`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(5)); g.source(a.rev) == 1 and a.rev.rev == a`
    @get pub fn rev(self) -> Arc = Arc(id ^ 1)

/// A flow network over the vertices `V`, solved with Dinic's algorithm (`max_flow`) or push-relabel (`push_relabel`).
/// `add` is \(O(1)\). `max_flow` is \(O(V^2 E)\) in general, \(O(E \sqrt{V})\) on unit
/// networks (bipartite matching). Memory is two arcs per `add`.
pub struct FlowGraph<V>:
    head: [V]List<Arc>
    to: List<V>
    left: List<Int>
    cap0: List<Int>
    cut_from: ?V

    /// An empty flow network over the vertices `V`.
    ///
    /// cost: \(O(n)\) for n = `V.count`
    /// traps: never
    /// example: `let g = FlowGraph<0..3>(); g.arcs.collect() == []`
    pub fn new() -> FlowGraph<V>:
        FlowGraph([V] v => [], [], [], [], None)

    /// Adds an arc `u -> v` with capacity `cap`, and its reverse `v -> u` with capacity 0, and
    /// returns the forward arc. A `cap` of `None` is an arc no cut can afford: capacity 2^60.
    /// Self-loops are allowed and never carry flow.
    ///
    /// cost: \(O(1)\) amortized; two arcs of memory
    /// traps: when `cap` is negative
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(5)); g.cap(a) == 5 and g.cap(a.rev) == 0`
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, None); g.cap(a) == 1 << 60`
    pub fn add(mut self, u: V, v: V, cap: ?Int) -> Arc:
        let cap = cap ?? 1 << 60
        assert cap >= 0
        cut_from = None
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

    /// Adds an undirected edge between `u` and `v`: an arc `u -> v` whose reverse also has
    /// capacity `cap`. Returns the `u -> v` arc.
    ///
    /// cost: \(O(1)\) amortized; two arcs of memory
    /// traps: when `cap` is negative
    /// example: `var g = FlowGraph<0..2>(); let a = g.add_both(0, 1, 3); g.cap(a.rev) == 3 and g.max_flow(1, 0) == 3`
    pub fn add_both(mut self, u: V, v: V, cap: Int) -> Arc:
        let a = self.add(u, v, cap)
        left[a.rev.id] = cap
        cap0[a.rev.id] = cap
        a

    /// The flow currently sent along `a`: negative on the reverse arc of a used arc, so
    /// `flow(a.rev) == -flow(a)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(5)); g.max_flow(0, 1); g.flow(a) == 5 and g.flow(a.rev) == -5`
    pub fn flow(self, a: Arc) -> Int = cap0[a.id] - left[a.id]

    /// The capacity `a` has now: what `add` or the last `set_cap` gave it, 0 for the reverse arc of
    /// an `add`, `cap` for either arc of an `add_both`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(4)); g.cap(a) == 4`
    pub fn cap(self, a: Arc) -> Int = cap0[a.id]

    /// The vertex `a` leaves.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var g = FlowGraph<0..3>(); let a = g.add(2, 0, Some(1)); g.source(a) == 2 and g.source(a.rev) == 0`
    pub fn source(self, a: Arc) -> V = to[a.id ^ 1]

    /// The vertex `a` points to.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var g = FlowGraph<0..3>(); let a = g.add(2, 0, Some(1)); g.target(a) == 0 and g.target(a.rev) == 2`
    pub fn target(self, a: Arc) -> V = to[a.id]

    /// Pushes as much flow as possible from `s` to `t`, on top of the flow already there but no more
    /// than `limit` in this call, and returns the amount added.
    ///
    /// Dinic's algorithm: BFS levels, then blocking flows by DFS scanning each vertex's arcs in
    /// `add` order. Calling it again with no change adds 0; after it runs to completion (returns
    /// less than `limit`, or `limit` is `None`), `min_cut(s)` and `cut_arcs(s)` are valid.
    ///
    /// cost: \(O(V^2 E)\); \(O(E \sqrt V)\) when every capacity is 1
    /// ties: among maximum flows, the one Dinic finds scanning arcs in `add` order
    /// traps: when `s == t` or `limit` is negative
    /// example: `var g = FlowGraph<0..4>(); g.add(0, 1, Some(3)); g.add(0, 2, Some(2)); g.add(1, 2, Some(1)); g.add(1, 3, Some(2)); g.add(2, 3, Some(3)); g.max_flow(0, 3) == 5`
    /// example: `var g = FlowGraph<0..4>(); g.add(0, 1, Some(3)); g.add(0, 2, Some(2)); g.add(1, 2, Some(1)); g.add(1, 3, Some(2)); g.add(2, 3, Some(3)); g.max_flow(0, 3, limit: Some(4)) == 4 and g.max_flow(0, 3) == 1`
    pub fn max_flow(mut self, s: V, t: V, limit: ?Int = None) -> Int:
        assert s != t
        cut_from = None
        let limit = limit ?? Int.MAX
        var total = 0
        while total < limit:
            let level = self.levels(s)
            if level[t] == None:
                cut_from = s
                break
            var next: [V]Int = 0
            while total < limit:
                let pushed = self.augment(s, t, level, mut next, limit - total)
                if pushed == 0: break
                total += pushed
        total

    /// The source side of a minimum `s`-`t` cut: the vertices `s` still reaches in the residual
    /// network, after `max_flow(s, t)` (or `push_relabel(s, t)`) ran to completion.
    ///
    /// cost: \(O(V + E)\)
    /// ties: the smallest source side among minimum cuts (exactly what `s` reaches)
    /// order: keyed by `V`
    /// traps: when no completed `max_flow` or `push_relabel` from `s` came since the last `add`, `set_cap` or `reset`
    /// example: `var g = FlowGraph<0..3>(); g.add(0, 1, Some(5)); g.add(1, 2, Some(1)); g.max_flow(0, 2); g.min_cut(0) == [true, true, false]`
    pub fn min_cut(self, s: V) -> [V]Bool:
        assert cut_from == Some(s), "min_cut(s) needs a completed max_flow(s, t) first"
        let level = self.levels(s)
        [V] v => level[v] != None

    /// Clears all flow, so every arc is back at its current capacity with nothing sent.
    ///
    /// cost: \(O(E)\)
    /// traps: never
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(5)); g.max_flow(0, 1); g.reset(); g.flow(a) == 0 and g.max_flow(0, 1) == 5`
    pub fn reset(mut self):
        cut_from = None
        for k in 0..left.len: left[k] = cap0[k]

    /// Changes arc `a`'s capacity to `cap` (`None`: 2^60), keeping its flow; a later `max_flow`
    /// continues from the current flow. The reverse arc keeps its own capacity.
    ///
    /// cost: \(O(1)\)
    /// traps: when the flow on `a` exceeds the new capacity (lower it with `reset` first), or `cap` is negative
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(2)); g.max_flow(0, 1); g.set_cap(a, Some(5)); g.max_flow(0, 1) == 3`
    pub fn set_cap(mut self, a: Arc, cap: ?Int):
        let c = cap ?? 1 << 60
        let f = cap0[a.id] - left[a.id]
        assert c >= 0 and f <= c, "`set_cap(a, cap)` needs `0 <= flow(a) <= cap`"
        cap0[a.id] = c
        left[a.id] = c - f
        cut_from = None

    /// The capacity left on `a` in the residual network: `cap(a) - flow(a)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(5)); g.max_flow(0, 1, limit: Some(2)); g.residual(a) == 3 and g.residual(a.rev) == 2`
    pub fn residual(self, a: Arc) -> Int = left[a.id]

    /// Every arc `add` or `add_both` returned, lazily; never a reverse arc. \(O(1)\) to make.
    ///
    /// cost: \(O(E)\) for the whole walk
    /// empty: yields nothing before the first `add`
    /// order: the order `add` returned them
    /// traps: never
    /// example: `var g = FlowGraph<0..3>(); let a = g.add(0, 1, Some(1)); let b = g.add(1, 2, Some(1)); g.arcs.collect() == [a, b]`
    @get pub fn arcs(self) -> yields Arc:
        for k in 0..to.len / 2: yield Arc(2 * k)

    /// The arcs `add` or `add_both` returned whose source is `u`, lazily; never a reverse arc.
    ///
    /// cost: \(O(\deg u)\) for the whole walk
    /// empty: yields nothing when no added arc leaves `u`
    /// order: the order `add` returned them
    /// traps: never
    /// example: `var g = FlowGraph<0..3>(); let a = g.add(0, 1, Some(1)); g.add(1, 2, Some(1)); let c = g.add(0, 2, Some(1)); g.out(0).collect() == [a, c]`
    pub fn out(self, u: V) -> yields Arc:
        for a in head[u]:
            if a.id % 2 == 0: yield a

    /// The added arcs crossing the minimum cut `min_cut(s)` from the source side to the other, after
    /// `max_flow(s, t)` ran to completion. Every one is saturated and their capacities sum to the
    /// flow value.
    ///
    /// cost: \(O(V + E)\)
    /// order: the order `add` returned them
    /// traps: like `min_cut`, when the flow from `s` is missing or stale
    /// example: `var g = FlowGraph<0..3>(); g.add(0, 1, Some(5)); let b = g.add(1, 2, Some(1)); g.max_flow(0, 2); g.cut_arcs(0) == [b]`
    pub fn cut_arcs(self, s: V) -> List<Arc>:
        let side = self.min_cut(s)
        [Arc(2 * k) for k in 0..to.len / 2 if side[to[2 * k + 1]] and not side[to[2 * k]]]

    /// The current `s`-`t` flow split into paths, each with the amount it carries; the amounts sum
    /// to the net flow out of `s`. Cycles of flow are skipped. Does not change the flow.
    ///
    /// Each path walks from `s` along the first arc, in `add` order, that still carries unclaimed
    /// flow, until it reaches `t`; its amount is the least unclaimed flow on the way.
    ///
    /// cost: \(O(P (V + E))\) for P paths
    /// empty: yields nothing when no flow goes from `s` to `t`
    /// order: discovery order, as above
    /// traps: when `s == t`
    /// example: `var g = FlowGraph<0..4>(); let a = g.add(0, 1, Some(1)); let b = g.add(1, 3, Some(1)); let c = g.add(0, 2, Some(1)); let d = g.add(2, 3, Some(1)); g.max_flow(0, 3); g.paths(0, 3).collect() == [(arcs: [a, b], amount: 1), (arcs: [c, d], amount: 1)]`
    pub fn paths(self, s: V, t: V) -> yields (arcs: List<Arc>, amount: Int):
        assert s != t, "`paths(s, t)` needs `s != t`"
        var spare = [if k % 2 == 0: cap0[k] - left[k] else: 0 for k in 0..to.len]
        var next: [V]Int = 0
        var on: [V]?Int = None
        while true:
            var path = List<Arc>()
            var u = s
            on[s] = 0
            while u != t:
                while next[u] < head[u].len and spare[head[u][next[u]].id] <= 0: next[u] += 1
                if next[u] == head[u].len: break
                let a = head[u][next[u]]
                let v = to[a.id]
                path.push(a)
                if let k = on[v]:
                    var least = Int.MAX
                    for i in k..path.len: least = min(least, spare[path[i].id])
                    for i in k..path.len: spare[path[i].id] -= least
                    while path.len > k:
                        on[to[path.last!.id]] = None
                        path.pop()
                    on[v] = k
                    u = v
                else:
                    on[v] = path.len
                    u = v
            for a in path: on[to[a.id]] = None
            on[s] = None
            if u != t: return
            var least = Int.MAX
            for a in path: least = min(least, spare[a.id])
            for a in path: spare[a.id] -= least
            yield (arcs: path, amount: least)

    /// The same maximum flow amount as `max_flow(s, t)` by push-relabel (highest label, gap
    /// heuristic), often several times faster than Dinic on dense or unit-capacity grids. Adds to
    /// the flow already there and returns the amount added; `min_cut(s)` works after it.
    ///
    /// cost: \(O(V^2 \sqrt E)\)
    /// traps: when `s == t`
    /// example: `var g = FlowGraph<0..4>(); g.add(0, 1, Some(3)); g.add(0, 2, Some(2)); g.add(1, 2, Some(1)); g.add(1, 3, Some(2)); g.add(2, 3, Some(3)); g.push_relabel(0, 3) == 5`
    pub fn push_relabel(mut self, s: V, t: V) -> Int:
        assert s != t
        cut_from = s
        let n = V.K
        var height: [V]Int = 0
        var excess: [V]Int = 0
        var next: [V]Int = 0
        var active = List.filled(2 * n, List<V>())
        var count = List.filled(2 * n, 0)
        height[s] = n
        // t is never active: a vertex is queued only when its excess leaves 0
        excess[t] = 1
        count[0] = n - 1
        for a in head[s]: self.push(a, left[a.id], mut excess, mut active, height)
        var top = 0
        unchecked:
            while true:
                while active[top].is_empty:
                    if top == 0: return -excess[s]
                    top -= 1
                let u = active[top].pop()
                while excess[u] > 0:
                    if next[u] == head[u].len:
                        height[u] = Int.MAX
                        for k in 0..head[u].len:
                            let a = head[u][k]
                            if left[a.id] > 0 and height[to[a.id]] < height[u] - 1:
                                height[u] = height[to[a.id]] + 1
                                next[u] = k
                        count[height[u]] += 1
                        count[top] -= 1
                        if count[top] == 0 and top < n:
                            for v in V:
                                if top < height[v] and height[v] < n:
                                    count[height[v]] -= 1
                                    height[v] = n + 1
                        top = height[u]
                    else:
                        let a = head[u][next[u]]
                        if left[a.id] > 0 and height[u] == height[to[a.id]] + 1:
                            self.push(a, min(excess[u], left[a.id]), mut excess, mut active, height)
                        else:
                            next[u] += 1
        0

    fn push(
        mut self,
        a: Arc,
        f: Int,
        mut excess: [V]Int,
        mut active: List<List<V>>,
        height: [V]Int,
    ):
        unchecked:
            let v = to[a.id]
            if excess[v] == 0 and f > 0: active[height[v]].push(v)
            left[a.id] -= f
            left[a.rev.id] += f
            excess[v] += f
            excess[to[a.rev.id]] -= f

    fn close(mut self, a: Arc):
        left[a.id] = 0
        left[a.rev.id] = 0

    fn levels(self, s: V) -> [V]?Int:
        var level: [V]?Int = None
        level[s] = 0
        var queue = [s]
        var at = 0
        unchecked:
            while at < queue.len:
                let u = queue[at]
                at += 1
                for a in head[u]:
                    let v = to[a.id]
                    if left[a.id] > 0 and level[v] == None:
                        level[v] = level[u]! + 1
                        queue.push(v)
        level

    /// One augmenting path in the level graph, found without recursion.
    pub fn augment(mut self, s: V, t: V, level: [V]?Int, mut next: [V]Int, most: Int) -> Int:
        var path = List<Arc>()
        var u = s
        unchecked:
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
        let amount = min(most, [left[a.id] for a in path].min()!)
        for a in path:
            left[a.id] -= amount
            left[a.rev.id] += amount
        amount

/// A flow network with a cost per unit on every arc, solved by successive shortest paths
/// with Johnson potentials. Arc handles and accessors are those of `FlowGraph`.
/// `add` is \(O(1)\); memory is two arcs per `add`.
pub struct CostFlow<V>:
    head: [V]List<Arc>
    to: List<V>
    left: List<Int>
    cap0: List<Int>
    price: List<Int>

    /// An empty min-cost flow network over the vertices `V`.
    ///
    /// cost: \(O(n)\) for n = `V.count`
    /// traps: never
    /// example: `let g = CostFlow<0..3>(); g.arcs.collect() == []`
    pub fn new() -> CostFlow<V>:
        CostFlow([V] v => [], [], [], [], [])

    /// Adds an arc `u -> v` with capacity `cap` and cost `cost` per unit; its reverse has capacity 0
    /// and cost `-cost`. `cap: None` is 2^60. Costs may be negative; `min_cost_flow` needs no
    /// residual cycle of negative total cost (run `cancel_cycles` first when there may be one).
    ///
    /// cost: \(O(1)\) amortized; two arcs of memory
    /// traps: when `cap` is negative
    /// example: `var g = CostFlow<0..2>(); let a = g.add(0, 1, Some(2), 7); g.cost(a) == 7 and g.cost(a.rev) == -7`
    pub fn add(mut self, u: V, v: V, cap: ?Int, cost: Int) -> Arc:
        let cap = cap ?? 1 << 60
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
        price.push(cost)
        price.push(-cost)
        a

    /// The flow currently sent along `a`; `flow(a.rev) == -flow(a)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var g = CostFlow<0..2>(); let a = g.add(0, 1, Some(2), 1); g.min_cost_flow(0, 1); g.flow(a) == 2`
    pub fn flow(self, a: Arc) -> Int = cap0[a.id] - left[a.id]

    /// The capacity `a` has now: what `add` or the last `set_cap` gave it, 0 for a reverse arc.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var g = CostFlow<0..2>(); let a = g.add(0, 1, Some(2), 1); g.cap(a) == 2 and g.cap(a.rev) == 0`
    pub fn cap(self, a: Arc) -> Int = cap0[a.id]

    /// The cost per unit of flow along `a`; `cost(a.rev) == -cost(a)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var g = CostFlow<0..2>(); let a = g.add(0, 1, Some(2), -3); g.cost(a) == -3`
    pub fn cost(self, a: Arc) -> Int = price[a.id]

    /// The vertex `a` leaves.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var g = CostFlow<0..3>(); let a = g.add(2, 0, Some(1), 0); g.source(a) == 2`
    pub fn source(self, a: Arc) -> V = to[a.id ^ 1]

    /// The vertex `a` points to.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var g = CostFlow<0..3>(); let a = g.add(2, 0, Some(1), 0); g.target(a) == 0`
    pub fn target(self, a: Arc) -> V = to[a.id]

    /// Changes arc `a`'s capacity, keeping its flow, as [`FlowGraph.set_cap`](FlowGraph.set_cap).
    ///
    /// cost: \(O(1)\)
    /// traps: when the flow on `a` exceeds the new capacity, or `cap` is negative
    /// example: `var g = CostFlow<0..2>(); let a = g.add(0, 1, Some(1), 2); g.min_cost_flow(0, 1); g.set_cap(a, Some(3)); g.min_cost_flow(0, 1) == (flow: 2, cost: 4)`
    pub fn set_cap(mut self, a: Arc, cap: ?Int):
        let c = cap ?? 1 << 60
        let f = cap0[a.id] - left[a.id]
        assert c >= 0 and f <= c, "`set_cap(a, cap)` needs `0 <= flow(a) <= cap`"
        cap0[a.id] = c
        left[a.id] = c - f

    /// Every arc `add` returned, lazily; never a reverse arc. \(O(1)\) to make.
    ///
    /// cost: \(O(E)\) for the whole walk
    /// empty: yields nothing before the first `add`
    /// order: the order `add` returned them
    /// traps: never
    /// example: `var g = CostFlow<0..2>(); let a = g.add(0, 1, Some(1), 0); let b = g.add(1, 0, Some(1), 0); g.arcs.collect() == [a, b]`
    @get pub fn arcs(self) -> yields Arc:
        for k in 0..to.len / 2: yield Arc(2 * k)

    /// Saturates every negative-cost cycle of the residual network, so the stored flow becomes a
    /// min-cost circulation plus the `s`-`t` flows already sent, and returns the cost added (zero or
    /// negative). Negative cycles are allowed here; afterwards `min_cost_flow` is safe.
    ///
    /// cost: \(O(V E)\) per cancelled cycle (Bellman-Ford), at most `|c|` cycles for the returned cost `c`
    /// traps: never
    /// example: `var g = CostFlow<0..2>(); g.add(0, 1, Some(1), -2); g.add(1, 0, Some(1), 1); g.cancel_cycles() == -1`
    /// example: `var g = CostFlow<0..2>(); g.add(0, 1, Some(1), 2); g.cancel_cycles() == 0`
    pub fn cancel_cycles(mut self) -> Int:
        with lint(unproven_list_index: allow)
        let n = head.len
        if n == 0: return 0
        var total = 0
        loop:
            var dist: [V]Int = [V] _ => 0
            var par_v: [V]?V = [V] _ => None
            var par_a: [V]Int = [V] _ => -1
            var hit: ?V = None
            for _ in 0..n:
                hit = None
                for u in V:
                    for a in head[u]:
                        if left[a.id] > 0:
                            let v = to[a.id]
                            let nd = dist[u] + price[a.id]
                            if nd < dist[v]:
                                dist[v] = nd
                                par_v[v] = Some(u)
                                par_a[v] = a.id
                                hit = Some(v)
                if hit == None: break
            if hit == None: break
            var cur = hit!
            for _ in 0..n: cur = par_v[cur]!
            let start = cur
            var cyc: List<Int> = []
            var f = 1 << 60
            var c = 0
            loop:
                let aid = par_a[cur]
                cyc.push(aid)
                f min= left[aid]
                c += price[aid]
                cur = par_v[cur]!
                if cur == start: break
            for aid in cyc:
                left[aid] -= f
                left[aid ^ 1] += f
            total += f * c
        total

    /// Sends the most flow from `s` to `t`, up to `limit` in this call, at the least cost among
    /// flows of that amount, on top of the flow already there. Returns the amount and cost added.
    ///
    /// Successive shortest paths with Johnson potentials: each augmentation takes a cheapest
    /// residual path, so the cost added is convex in the amount (see `slope`). Costs may be
    /// negative; the first call with a negative residual arc runs Bellman-Ford once.
    ///
    /// cost: \(O(F E \log V)\) for F augmenting paths, plus \(O(V E)\) once when a residual arc has a negative cost
    /// ties: among cheapest flows, the one the shortest paths find (Dijkstra breaking ties by least vertex)
    /// traps: when `s == t`, `limit` is negative, or the residual network has a negative-cost cycle
    /// example: `var g = CostFlow<0..3>(); g.add(0, 1, Some(2), 1); g.add(1, 2, Some(2), 1); g.add(0, 2, Some(1), 5); g.min_cost_flow(0, 2) == (flow: 3, cost: 9)`
    /// example: `var g = CostFlow<0..3>(); g.add(0, 1, Some(2), 1); g.add(1, 2, Some(2), 1); g.add(0, 2, Some(1), 5); g.min_cost_flow(0, 2, limit: Some(2)) == (flow: 2, cost: 4)`
    pub fn min_cost_flow(mut self, s: V, t: V, limit: ?Int = None) -> (flow: Int, cost: Int):
        let curve = self.slope(s, t, limit)
        curve.last!

    /// The breakpoints of the cheapest cost of sending each amount from `s` to `t`, up to `limit`:
    /// `(0, 0)` first, then one `(flow, cost)` per change of slope, ending at the largest amount.
    /// Between breakpoints the cost is linear and the slopes never decrease. Sends the flow, as
    /// `min_cost_flow` does; amounts and costs are relative to the flow already there.
    ///
    /// cost: as `min_cost_flow`
    /// order: increasing `flow`
    /// traps: as `min_cost_flow`
    /// example: `var g = CostFlow<0..3>(); g.add(0, 1, Some(2), 1); g.add(1, 2, Some(2), 1); g.add(0, 2, Some(1), 5); g.slope(0, 2) == [(flow: 0, cost: 0), (flow: 2, cost: 4), (flow: 3, cost: 9)]`
    /// example: `var g = CostFlow<0..2>(); g.slope(0, 1) == [(flow: 0, cost: 0)]`
    pub fn slope(mut self, s: V, t: V, limit: ?Int = None) -> List<(flow: Int, cost: Int)>:
        assert s != t
        let limit = limit ?? Int.MAX
        var pot = self.potentials()
        var curve = [(flow: 0, cost: 0)]
        var (flow, total) = (0, 0)
        var last_unit: ?Int = None
        unchecked:
            while flow < limit:
                var dist: [V]?Int = None
                var via: [V]?Arc = None
                var done: [V]Bool = false
                dist[s] = 0
                var heap = MinHeap<(Int, V)>()
                heap.push((0, s))
                while let (d, u) = heap.pop():
                    if done[u]: continue
                    done[u] = true
                    if u == t: break
                    for a in head[u]:
                        if left[a.id] == 0: continue
                        let v = to[a.id]
                        let nd = d + price[a.id] + pot[u] - pot[v]
                        if dist[v] == None or nd < dist[v]!:
                            dist[v] = nd
                            via[v] = a
                            heap.push((nd, v))
                if not done[t]: break
                let dt = dist[t]!
                for v in V:
                    if done[v]: pot[v] += dist[v]!
                    else: pot[v] += dt
                var amount = limit - flow
                var v = t
                while v != s:
                    let a = via[v]!
                    amount min= left[a.id]
                    v = to[a.id ^ 1]
                v = t
                while v != s:
                    let a = via[v]!
                    left[a.id] -= amount
                    left[a.id ^ 1] += amount
                    v = to[a.id ^ 1]
                let unit = pot[t] - pot[s]
                flow += amount
                total += amount * unit
                if last_unit == unit: curve.pop()
                curve.push((flow:, cost: total))
                last_unit = unit
        curve

    /// Potentials under which every residual arc has a nonnegative reduced cost: all zero
    /// when no residual arc is negative, else Bellman-Ford from every vertex at once.
    pub fn potentials(self) -> [V]Int:
        var pot: [V]Int = 0
        var changed = true
        var rounds = 0
        unchecked:
            while changed:
                changed = false
                rounds += 1
                assert rounds <= V.len + 1, "a negative-cost cycle has spare capacity"
                for u in V:
                    for a in head[u]:
                        if left[a.id] > 0 and pot[u] + price[a.id] < pot[to[a.id]]:
                            pot[to[a.id]] = pot[u] + price[a.id]
                            changed = true
        pot

/// A maximum-weight closure (project selection): the set `take` of vertices with the largest
/// total `profit` such that an arc `u -> v` of `before` means taking `v` forces taking `u`
/// (prerequisites point at what they unlock, the direction `peel` and `topo` read).
///
/// Profits may be negative; the empty set is a closure, so `value >= 0`. One `max_flow` on
/// V + 2 vertices and V + E arcs; `take` is the source side of its minimum cut.
///
/// cost: one `max_flow` on V + 2 vertices and V + E arcs, \(O(V + E)\) calls of `profit`
/// ties: the smallest optimal closure (contained in every other)
/// order: `take` keyed by `V`
/// traps: never
/// example: `domain V = 0..3; let before: [V]List<V> = [[], [0], []]; let p = [3, -1, -5]; max_closure(before, profit: v => p[v.to_idx]) == (value: 2, take: [true, true, false])`
/// example: `domain V = 0..1; let before: [V]List<V> = [[]]; max_closure(before, profit: _ => 0) == (value: 0, take: [false])`
pub fn max_closure<V, G: Next<V>>(before: G, profit: V -> Int) -> (value: Int, take: [V]Bool):
    domain N = 0..V.K + 2
    let (s, t) = (N(V.K), N(V.K + 1))
    var g = FlowGraph<N>()
    var gain = 0
    for v in V:
        let p = profit(v)
        if p > 0:
            g.add(s, N(v.to_idx), p)
            gain += p
        elif p < 0: g.add(N(v.to_idx), t, -p)
        for a in before.out(v): g.add(N(a.to.to_idx), N(v.to_idx), cap: None)
    let lost = g.max_flow(s, t)
    let side = g.min_cut(s)
    (value: gain - lost, take: [V] v => side[N(v.to_idx)])

/// A maximum matching between `L` and `R`, where `adj[l]` lists the partners `l` may take: its
/// `size`, and the partner of each vertex on either side (`None` when unmatched).
///
/// Hopcroft-Karp without recursion, scanning `L` in increasing order and each `adj[l]` in list
/// order; repeated entries are harmless.
///
/// cost: \(O(E \sqrt V)\) time, \(O(V + E)\) memory
/// empty: `(size: 0, ...)` with every entry `None` when `L` or every list is empty
/// ties: the maximum matching Hopcroft-Karp finds in the scan order above
/// traps: never
/// ```cp
/// domain L = 0..2
/// domain R = 0..2
/// let adj: [L]List<R> = [[0, 1], [0]]
/// let m = max_matching(adj)
/// m.size == 2 and m.mate == [Some(1), Some(0)] and m.back == [Some(1), Some(0)]
/// ```
pub fn max_matching<L, R>(adj: [L]List<R>) -> (size: Int, mate: [L]?R, back: [R]?L):
    var mate: [L]?R = None
    var back: [R]?L = None
    var size = 0
    for u in L:
        for v in adj[u]:
            if back[v] == None:
                mate[u] = Some(v)
                back[v] = Some(u)
                size += 1
                break
    size += grow_matching(adj, mut mate, mut back)
    (size:, mate:, back:)

fn grow_matching<L, R>(adj: [L]List<R>, mut mate: [L]?R, mut back: [R]?L) -> Int:
    var added = 0
    while true:
        var level = alternating_layers(adj, mate, back)
        var open = false
        for u in L:
            if open: break
            if level[u] != None:
                for v in adj[u]:
                    if back[v] == None:
                        open = true
                        break
        if not open: break
        var next: [L]Int = 0
        for root in L:
            if mate[root] != None: continue
            var path = [root]
            while not path.is_empty:
                let u = path.last!
                if next[u] == adj[u].len:
                    level[u] = None
                    let _ = path.pop()
                    continue
                let v = adj[u][next[u]]
                next[u] += 1
                if let w = back[v]:
                    if level[w] != None and level[w]! == level[u]! + 1: path.push(w)
                else:
                    flip_path(path, v, mut mate, mut back)
                    added += 1
                    path = []
    added

fn flip_path<L, R>(path: List<L>, end: R, mut mate: [L]?R, mut back: [R]?L):
    var free = end
    for i in (0..path.len).rev:
        let x = path[i]
        let old = mate[x]
        mate[x] = Some(free)
        back[free] = Some(x)
        if let o = old: free = o

/// A minimum vertex cover of the bipartite graph `adj`: every edge has a marked end, and exactly
/// `max_matching(adj).size` vertices are marked (König). The unmarked vertices are a maximum
/// independent set.
///
/// With `Z` the vertices reachable from free left vertices by alternating paths (unmatched edge
/// left to right, matched edge right to left), the cover is the left vertices outside `Z` and the
/// right vertices inside it; this cover does not depend on which maximum matching is used.
///
/// cost: \(O(E \sqrt V)\)
/// empty: nothing marked when there are no edges
/// ties: the König cover above
/// order: `left` keyed by `L`, `right` keyed by `R`
/// traps: never
/// ```cp
/// domain L = 0..2
/// domain R = 0..1
/// let adj: [L]List<R> = [[0], [0]]
/// min_vertex_cover(adj) == (left: [false, false], right: [true])
/// ```
pub fn min_vertex_cover<L, R>(adj: [L]List<R>) -> (left: [L]Bool, right: [R]Bool):
    let m = max_matching(adj)
    let level = alternating_layers(adj, m.mate, m.back)
    var right: [R]Bool = false
    for u in L:
        if level[u] != None:
            for v in adj[u]: right[v] = true
    (left: [L] u => level[u] == None, right:)

/// Breadth-first layers of the left side by alternating paths: free vertices are layer 0,
/// and a matched vertex is one deeper than a left vertex adjacent to its partner.
fn alternating_layers<L, R>(adj: [L]List<R>, mate: [L]?R, back: [R]?L) -> [L]?(0..2_147_483_648):
    var level: [L]?(0..2_147_483_648) = None
    var queue = [u for u in L if mate[u] == None]
    for u in queue: level[u] = 0
    var at = 0
    unchecked:
        while at < queue.len:
            let u = queue[at]
            at += 1
            for v in adj[u]:
                if let w = back[v]:
                    if level[w] == None:
                        level[w] = Some(level[u]! + 1)
                        queue.push(w)
    level

/// The maximum-weight assignment: every `l` in `L` gets a distinct `mate[l]` in `R`, maximizing
/// `total`, the sum of `weight(l, mate[l])`. Needs `L.count <= R.count`; with fewer left vertices
/// some of `R` stay unused.
///
/// cost: \(O(L^2 R)\) time, \(O(L R)\) calls of `weight`, by the Hungarian algorithm
/// ties: the assignment the Hungarian algorithm finds adding rows in increasing order and scanning columns in increasing order
/// order: `mate` keyed by `L`
/// traps: when `L.count > R.count`
/// ```cp
/// domain L = 0..2
/// domain R = 0..2
/// let w: [L][R]Int = [[1, 5], [4, 3]]
/// let m = max_weight_matching<L, R>(weight: (l, r) => w[l][r])
/// m.total == 9 and m.mate == [1, 0]
/// ```
/// ```cp
/// domain L = 0..1
/// domain R = 0..2
/// let m = max_weight_matching<L, R>(weight: (_, r) => Int(r))
/// m.total == 1 and m.mate == [1]
/// ```
pub fn max_weight_matching<L, R>(weight: (L, R) -> Int) -> (total: Int, mate: [L]R):
    let mate = hungarian<L, R>((l, r) => -weight(l, r))
    (total: sum(weight(l, mate[l]) for l in L), mate:)

/// The minimum-weight assignment: every `l` in `L` gets a distinct `mate[l]` in `R`, minimizing
/// `total`, the sum of `weight(l, mate[l])`. Needs `L.count <= R.count`.
///
/// cost: \(O(L^2 R)\) time, \(O(L R)\) calls of `weight`, by the Hungarian algorithm
/// ties: the assignment the Hungarian algorithm finds adding rows in increasing order and scanning columns in increasing order
/// order: `mate` keyed by `L`
/// traps: when `L.count > R.count`
/// ```cp
/// domain L = 0..2
/// domain R = 0..2
/// let w: [L][R]Int = [[1, 5], [4, 3]]
/// let m = min_weight_matching<L, R>(weight: (l, r) => w[l][r])
/// m.total == 4 and m.mate == [0, 1]
/// ```
pub fn min_weight_matching<L, R>(weight: (L, R) -> Int) -> (total: Int, mate: [L]R):
    let mate = hungarian<L, R>(weight)
    (total: sum(weight(l, mate[l]) for l in L), mate:)

/// Minimum-cost assignment of rows `L` into columns `R` with potentials `u`, `v`; row and
/// column 0 are the sentinel of the e-maxx formulation, so indices are 1-based inside.
fn hungarian<L, R>(cost: (L, R) -> Int) -> [L]R:
    let (n, m) = (L.K, R.K)
    assert n <= m
    var u = List.filled(n + 1, 0)
    var v = List.filled(m + 1, 0)
    var p = List.filled(m + 1, 0)
    var way = List.filled(m + 1, 0)
    var minv = List.filled(m + 1, 0)
    var used = List.filled(m + 1, false)
    unchecked:
        for i in 1..=n:
            p[0] = i
            var j0 = 0
            for j in 0..=m:
                minv[j] = Int.MAX
                used[j] = false
            while true:
                used[j0] = true
                let i0 = p[j0]
                let row = L.nth(i0 - 1)
                var delta = Int.MAX
                var j1 = 0
                for j in 1..=m:
                    if used[j]: continue
                    let cur = cost(row, R.nth(j - 1)) - u[i0] - v[j]
                    if cur < minv[j]:
                        minv[j] = cur
                        way[j] = j0
                    if minv[j] < delta:
                        delta = minv[j]
                        j1 = j
                for j in 0..=m:
                    if used[j]:
                        u[p[j]] += delta
                        v[j] -= delta
                    else:
                        minv[j] -= delta
                j0 = j1
                if p[j0] == 0: break
            while j0 != 0:
                let j1 = way[j0]
                p[j0] = p[j1]
                j0 = j1
    var col = List.filled(n, 0)
    for j in 1..=m:
        if p[j] != 0: col[p[j] - 1] = j - 1
    [L] i => R.nth(col[i.to_idx])

/// A maximum matching in the undirected graph `adj` (each edge listed from both ends): its
/// `size` and each vertex's partner, `None` when unmatched.
///
/// Edmonds' blossom algorithm, growing a search from each free vertex in increasing order and
/// scanning `adj[v]` in list order; that fixes which maximum matching comes back.
///
/// cost: \(O(V³)\)
/// empty: `size == 0` and every `mate` `None` when there are no edges
/// traps: never
/// ```cp
/// domain V = 0..4
/// let adj: [V]List<V> = [[1], [0, 2], [1, 3], [2]]
/// let m = general_matching(adj)
/// m.size == 2 and m.mate == [Some(1), Some(0), Some(3), Some(2)]
/// ```
/// ```cp
/// domain V = 0..3
/// let adj: [V]List<V> = [[1, 2], [0, 2], [0, 1]]
/// general_matching(adj).size == 1
/// ```
pub fn general_matching<V>(adj: [V]List<V>) -> (size: Int, mate: [V]?V):
    let n = V.K
    let nb = [[w.to_idx for w in adj[V.nth(i)]] for i in 0..n]
    var mate = List.filled(n, -1)
    var size = 0
    unchecked:
        for x in 0..n:
            if mate[x] != -1: continue
            for y in nb[x]:
                if y != x and mate[y] == -1:
                    mate[x] = y
                    mate[y] = x
                    size += 1
                    break
        var parent = List.filled(n, -1)
        var base = List.filled(n, 0)
        var inq = List.filled(n, false)
        var blossom = List.filled(n, false)
        var seen = List.filled(n, false)
        for root in 0..n:
            if mate[root] != -1: continue
            for i in 0..n:
                parent[i] = -1
                base[i] = i
                inq[i] = false
            inq[root] = true
            var queue = [root]
            var at = 0
            var end = -1
            while at < queue.len and end == -1:
                let x = queue[at]
                at += 1
                for y in nb[x]:
                    if base[x] == base[y] or mate[x] == y: continue
                    if y == root or (mate[y] != -1 and parent[mate[y]] != -1):
                        // odd cycle: contract the blossom through the common ancestor
                        for i in 0..n: seen[i] = false
                        var a = x
                        while true:
                            a = base[a]
                            seen[a] = true
                            if mate[a] == -1: break
                            a = parent[mate[a]]
                        var b = y
                        while true:
                            b = base[b]
                            if seen[b]: break
                            b = parent[mate[b]]
                        let top = b
                        for i in 0..n: blossom[i] = false
                        for (from, child) in [(x, y), (y, x)]:
                            var w = from
                            var c = child
                            while base[w] != top:
                                blossom[base[w]] = true
                                blossom[base[mate[w]]] = true
                                parent[w] = c
                                c = mate[w]
                                w = parent[mate[w]]
                        for i in 0..n:
                            if blossom[base[i]]:
                                base[i] = top
                                if not inq[i]:
                                    inq[i] = true
                                    queue.push(i)
                    elif parent[y] == -1:
                        parent[y] = x
                        if mate[y] == -1:
                            end = y
                            break
                        inq[mate[y]] = true
                        queue.push(mate[y])
            var y = end
            if y != -1: size += 1
            while y != -1:
                let x = parent[y]
                let next = mate[x]
                mate[y] = x
                mate[x] = y
                y = next
    (size:, mate: [V] v => if mate[v.to_idx] == -1: None else: V.nth(mate[v.to_idx]))

/// Weighted blossom state over vertices 1..=n and blossoms n+1..=2n; index 0 is "None".
/// Edge cells are a (2n+1)^2 table: cell k = u * m + v holds the original edge (eu, ev, ew)
/// that represents u -> v once u or v is a blossom.
struct Blossom:
    n: Int
    m: Int
    nx: Int
    eu: List<Int>
    ev: List<Int>
    ew: List<Int>
    lab: List<Int>
    mate: List<Int>
    slack: List<Int>
    st: List<Int>
    pa: List<Int>
    from: List<Int>
    side: List<Int>
    vis: List<Int>
    stamp: Int
    flower: List<List<Int>>
    queue: List<Int>
    at: Int

    fn delta(self, k: Int) -> Int:
        unchecked:
            return lab[eu[k]] + lab[ev[k]] - 2 * ew[k]

    fn update_slack(mut self, u: Int, x: Int):
        unchecked:
            if slack[x] == 0 or self.delta(u * m + x) < self.delta(slack[x] * m + x): slack[x] = u

    fn set_slack(mut self, x: Int):
        slack[x] = 0
        unchecked:
            for u in 1..=n:
                if ew[u * m + x] > 0 and st[u] != x and side[st[u]] == 0: self.update_slack(u, x)

    fn push(mut self, x: Int):
        if x <= n: queue.push(x)
        else:
            for i in 0..flower[x].len: self.push(flower[x][i])

    fn set_st(mut self, x: Int, b: Int):
        st[x] = b
        if x > n:
            for i in 0..flower[x].len: self.set_st(flower[x][i], b)

    fn get_pr(mut self, b: Int, xr: Int) -> Int:
        var pr = 0
        while flower[b][pr] != xr: pr += 1
        if pr % 2 == 0: return pr
        var (i, j) = (1, flower[b].len - 1)
        while i < j:
            flower[b].swap(i, j)
            i += 1
            j -= 1
        flower[b].len - pr

    fn set_match(mut self, u: Int, v: Int):
        let k = u * m + v
        mate[u] = ev[k]
        if u <= n: return
        let xr = from[u * (n + 1) + eu[k]]
        let pr = self.get_pr(u, xr)
        for i in 0..pr: self.set_match(flower[u][i], flower[u][i ^ 1])
        self.set_match(xr, v)
        let f = flower[u].copy()
        flower[u] = [f[(i + pr) % f.len] for i in 0..f.len]

    fn augment(mut self, u: Int, v: Int):
        var (u, v) = (u, v)
        while true:
            let xnv = st[mate[u]]
            self.set_match(u, v)
            if xnv == 0: return
            self.set_match(xnv, st[pa[xnv]])
            u = st[pa[xnv]]
            v = xnv

    fn get_lca(mut self, u: Int, v: Int) -> Int:
        stamp += 1
        var (a, b) = (u, v)
        while a != 0 or b != 0:
            if a != 0:
                if vis[a] == stamp: return a
                vis[a] = stamp
                a = st[mate[a]]
                if a != 0: a = st[pa[a]]
            (a, b) = (b, a)
        0

    fn add_blossom(mut self, u: Int, lca: Int, v: Int):
        var b = n + 1
        while b <= nx and st[b] != 0: b += 1
        if b > nx: nx += 1
        lab[b] = 0
        side[b] = 0
        mate[b] = mate[lca]
        flower[b] = [lca]
        for (start, flip) in [(u, true), (v, false)]:
            var x = start
            while x != lca:
                flower[b].push(x)
                let y = st[mate[x]]
                flower[b].push(y)
                self.push(y)
                x = st[pa[y]]
            if flip:
                var (i, j) = (1, flower[b].len - 1)
                while i < j:
                    flower[b].swap(i, j)
                    i += 1
                    j -= 1
        self.set_st(b, b)
        for x in 1..=nx:
            ew[b * m + x] = 0
            ew[x * m + b] = 0
        for x in 1..=n: from[b * (n + 1) + x] = 0
        for i in 0..flower[b].len:
            let xs = flower[b][i]
            for x in 1..=nx:
                if ew[b * m + x] == 0 or self.delta(xs * m + x) < self.delta(b * m + x):
                    let (bx, sx) = (b * m + x, xs * m + x)
                    (eu[bx], ev[bx], ew[bx]) = (eu[sx], ev[sx], ew[sx])
                    let (xb, xxs) = (x * m + b, x * m + xs)
                    (eu[xb], ev[xb], ew[xb]) = (eu[xxs], ev[xxs], ew[xxs])
            for x in 1..=n:
                if from[xs * (n + 1) + x] != 0: from[b * (n + 1) + x] = xs
        self.set_slack(b)

    fn expand_blossom(mut self, b: Int):
        for i in 0..flower[b].len: self.set_st(flower[b][i], flower[b][i])
        let xr = from[b * (n + 1) + eu[b * m + pa[b]]]
        let pr = self.get_pr(b, xr)
        var i = 0
        while i < pr:
            let (xs, xns) = (flower[b][i], flower[b][i + 1])
            pa[xs] = eu[xns * m + xs]
            side[xs] = 1
            side[xns] = 0
            slack[xs] = 0
            self.set_slack(xns)
            self.push(xns)
            i += 2
        side[xr] = 1
        pa[xr] = pa[b]
        for i in pr + 1..flower[b].len:
            let xs = flower[b][i]
            side[xs] = -1
            self.set_slack(xs)
        st[b] = 0

    fn on_found(mut self, k: Int) -> Bool:
        let (u, v) = (st[eu[k]], st[ev[k]])
        if side[v] == -1:
            pa[v] = eu[k]
            side[v] = 1
            let nu = st[mate[v]]
            slack[v] = 0
            slack[nu] = 0
            side[nu] = 0
            self.push(nu)
        elif side[v] == 0:
            let lca = self.get_lca(u, v)
            if lca == 0:
                self.augment(u, v)
                self.augment(v, u)
                return true
            self.add_blossom(u, lca, v)
        false

    /// One augmentation with dual adjustments; false when no augmenting path pays.
    fn phase(mut self) -> Bool:
        for x in 1..=nx:
            side[x] = -1
            slack[x] = 0
        queue = []
        at = 0
        for x in 1..=nx:
            if st[x] == x and mate[x] == 0:
                pa[x] = 0
                side[x] = 0
                self.push(x)
        if queue.is_empty: return false
        while true:
            while at < queue.len:
                let u = queue[at]
                at += 1
                if side[st[u]] == 1: continue
                for v in 1..=n:
                    let k = u * m + v
                    if ew[k] > 0 and st[u] != st[v]:
                        if self.delta(k) == 0:
                            if self.on_found(k): return true
                        else: self.update_slack(u, st[v])
            var d = Int.MAX
            for b in n + 1..=nx:
                if st[b] == b and side[b] == 1: d min= lab[b] / 2
            for x in 1..=nx:
                if st[x] == x and slack[x] != 0:
                    if side[x] == -1: d min= self.delta(slack[x] * m + x)
                    elif side[x] == 0: d min= self.delta(slack[x] * m + x) / 2
            for u in 1..=n:
                if side[st[u]] == 0 and lab[u] <= d: return false
            for u in 1..=n:
                if side[st[u]] == 0: lab[u] -= d
                elif side[st[u]] == 1: lab[u] += d
            for b in n + 1..=nx:
                if st[b] == b:
                    if side[b] == 0: lab[b] += 2 * d
                    elif side[b] == 1: lab[b] -= 2 * d
            queue = []
            at = 0
            for x in 1..=nx:
                if st[x] == x and slack[x] != 0 and st[slack[x]] != x and self.delta(slack[x] * m + x) == 0:
                    if self.on_found(slack[x] * m + x): return true
            for b in n + 1..=nx:
                if st[b] == b and side[b] == 1 and lab[b] == 0: self.expand_blossom(b)
        false

/// A maximum-weight matching in the undirected graph `edges`: no vertex in two chosen edges,
/// maximizing `total`, the sum of chosen `w`, not necessarily of largest size. Edges with
/// `w <= 0` never help and are ignored; of parallel edges the heaviest counts.
///
/// cost: \(O(V³)\) time, \(O(V^2)\) memory, by Edmonds' weighted blossom algorithm with dual labels
/// ties: the matching the algorithm finds; `total` is exact
/// traps: never
/// ```cp
/// domain E = 0..3
/// domain V = 0..4
/// let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 2), (u: 1, v: 2, w: 3), (u: 2, v: 3, w: 2)]
/// let m = max_weight_general_matching(es)
/// m.total == 4 and m.mate == [Some(1), Some(0), Some(3), Some(2)]
/// ```
pub fn max_weight_general_matching<V, E>(
    edges: [E](u: V, v: V, w: Int)
) -> (total: Int, mate: [V]?V):
    let n = V.K
    let m = 2 * n + 1
    var eu = List.filled(m * m, 0)
    var ev = List.filled(m * m, 0)
    var ew = List.filled(m * m, 0)
    for u in 1..=n:
        for v in 1..=n:
            eu[u * m + v] = u
            ev[u * m + v] = v
    var top = 0
    for e in E:
        let (a, b, w) = (edges[e].u.to_idx + 1, edges[e].v.to_idx + 1, edges[e].w)
        if a != b and w > ew[a * m + b]:
            ew[a * m + b] = w
            ew[b * m + a] = w
            top max= w
    var from = List.filled(m * (n + 1), 0)
    for u in 1..=n: from[u * (n + 1) + u] = u
    var g = Blossom(
        n,
        m,
        n,
        eu,
        ev,
        ew,
        [if 1 <= i <= n: top else: 0 for i in 0..m],
        List.filled(m, 0),
        List.filled(m, 0),
        [i for i in 0..m],
        List.filled(m, 0),
        from,
        List.filled(m, -1),
        List.filled(m, 0),
        0,
        [[] for _ in 0..m],
        [],
        0,
    )
    var more = true
    while more: more = g.phase()
    let total = sum(g.ew[u * m + g.mate[u]] for u in 1..=n if g.mate[u] > u)
    (
        total:,
        mate: [V] v => if g.mate[v.to_idx + 1] == 0: None else: V.nth(g.mate[v.to_idx + 1] - 1),
    )

/// A flow `f` with `lo <= f[e] <= hi` on every arc that is conserved at every vertex, or `None`
/// when there is None.
///
/// Sends `lo` on every arc, then routes each vertex's excess from a super source to a super sink
/// with one `max_flow`; the result is `lo` plus that flow.
///
/// cost: one `max_flow` on V + 2 vertices and E + V arcs
/// order: keyed by `E`
/// traps: when some `lo > hi`
/// ```cp
/// domain E = 0..2
/// domain V = 0..2
/// let es: [E](u: V, v: V, lo: Int, hi: Int) = [(u: 0, v: 1, lo: 2, hi: 2), (u: 1, v: 0, lo: 0, hi: 5)]
/// circulation(es) == Some([2, 2])
/// ```
/// ```cp
/// domain E = 0..1
/// domain V = 0..2
/// let es: [E](u: V, v: V, lo: Int, hi: Int) = [(u: 0, v: 1, lo: 1, hi: 1)]
/// circulation(es) == None
/// ```
pub fn circulation<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int)) -> ?[E]Int:
    let r = bounded_flow(arcs, None, 0) ?? return None
    r.flow

/// The largest `s -> t` flow with `lo <= flow[e] <= hi` on every arc, conserved at every vertex
/// but `s` and `t`, or `None` when no such flow exists. `value` is the net flow out of `s`.
///
/// cost: two `max_flow`s on V + 2 vertices and E + V + 1 arcs
/// ties: among maximum flows, the one the two `max_flow`s find
/// order: `flow` keyed by `E`
/// traps: when some `lo > hi`, or `s == t`
/// ```cp
/// domain E = 0..2
/// domain V = 0..3
/// let es: [E](u: V, v: V, lo: Int, hi: Int) = [(u: 0, v: 1, lo: 1, hi: 3), (u: 1, v: 2, lo: 0, hi: 2)]
/// max_bounded_flow(es, 0, 2) == Some((value: 2, flow: [2, 2]))
/// ```
/// ```cp
/// domain E = 0..2
/// domain V = 0..3
/// let es: [E](u: V, v: V, lo: Int, hi: Int) = [(u: 0, v: 1, lo: 3, hi: 3), (u: 1, v: 2, lo: 0, hi: 2)]
/// max_bounded_flow(es, 0, 2) == None
/// ```
pub fn max_bounded_flow<V, E>(
    arcs: [E](u: V, v: V, lo: Int, hi: Int),
    s: V,
    t: V,
) -> ?(value: Int, flow: [E]Int):
    bounded_flow(arcs, (s, t), 1)

/// The smallest `s -> t` flow with `lo <= flow[e] <= hi` on every arc, conserved at every vertex
/// but `s` and `t`, never sending more back into `s` than leaves it (`value >= 0`), or `None`
/// when no such flow exists.
///
/// cost: as `max_bounded_flow`
/// ties: among minimum flows, the one the two `max_flow`s find
/// order: `flow` keyed by `E`
/// traps: when some `lo > hi`, or `s == t`
/// ```cp
/// domain E = 0..2
/// domain V = 0..3
/// let es: [E](u: V, v: V, lo: Int, hi: Int) = [(u: 0, v: 1, lo: 1, hi: 3), (u: 1, v: 2, lo: 0, hi: 2)]
/// min_bounded_flow(es, 0, 2) == Some((value: 1, flow: [1, 1]))
/// ```
pub fn min_bounded_flow<V, E>(
    arcs: [E](u: V, v: V, lo: Int, hi: Int),
    s: V,
    t: V,
) -> ?(value: Int, flow: [E]Int):
    bounded_flow(arcs, (s, t), -1)

/// Lower bounds by demands: send `lo` on every arc up front, then route each vertex's
/// excess from a super source to a super sink. With `ends`, an unbounded `t -> s` arc
/// makes the problem a circulation; `sense` then pushes more `s -> t` (1) or back (-1).
fn bounded_flow<V, E>(
    arcs: [E](u: V, v: V, lo: Int, hi: Int),
    ends: ?(V, V),
    sense: Int,
) -> ?(value: Int, flow: [E]Int):
    domain N = 0..V.K + 2
    let (ss, tt) = (N(V.K), N(V.K + 1))
    var g = FlowGraph<N>()
    var excess: [V]Int = 0
    let ids = [E] e => g.add(N(arcs[e].u.to_idx), N(arcs[e].v.to_idx), arcs[e].hi - arcs[e].lo)
    for e in E:
        assert 0 <= arcs[e].lo <= arcs[e].hi
        excess[arcs[e].v] += arcs[e].lo
        excess[arcs[e].u] -= arcs[e].lo
    var demand = 0
    for v in V:
        if excess[v] > 0:
            g.add(ss, N(v.to_idx), excess[v])
            demand += excess[v]
        elif excess[v] < 0:
            g.add(N(v.to_idx), tt, -excess[v])
    let back: ?Arc = if let (s, t) = ends: g.add(N(t.to_idx), N(s.to_idx), cap: None) else: None
    if g.max_flow(ss, tt) != demand: return None
    if let (s, t) = ends:
        let (ns, nt) = (N(s.to_idx), N(t.to_idx))
        let back = back!
        let base = g.flow(back)
        g.close(back)
        let extra = if sense > 0: g.max_flow(ns, nt) else: -g.max_flow(nt, ns, limit: base)
        return (value: base + extra, flow: [E] e => arcs[e].lo + g.flow(ids[e]))
    (value: 0, flow: [E] e => arcs[e].lo + g.flow(ids[e]))

/// A Gomory-Hu tree of the undirected graph `edges`: `V.count - 1` edges on `V` such that for
/// every `s != t` the minimum `s`-`t` cut of `edges` is the smallest `w` on the tree path from
/// `s` to `t`. Vertices in different components are joined by `w = 0`.
///
/// Gusfield's algorithm: every vertex starts with parent `V.first`; for each `u` after it in
/// order, `w` is the minimum cut between `u` and its parent `p`, the tree gains `(u, p, w)`, and
/// every later vertex with parent `p` on `u`'s side (what `u` reaches in the residual) takes
/// parent `u`.
///
/// cost: `V.count - 1` max flows on one reused network, each \(O(V^2 E)\)
/// empty: `[]` when `V` has at most one position
/// order: one edge per vertex but `V.first`, by increasing `u`; `v` is its parent
/// traps: when some `w` is negative
/// ```cp
/// domain E = 0..2
/// domain V = 0..3
/// let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 2), (u: 1, v: 2, w: 3)]
/// gomory_hu(es) == [(u: 1, v: 0, w: 2), (u: 2, v: 1, w: 3)]
/// ```
pub fn gomory_hu<V, E>(edges: [E](u: V, v: V, w: Int)) -> List<(u: V, v: V, w: Int)>:
    var g = FlowGraph<V>()
    for e in E: g.add_both(edges[e].u, edges[e].v, edges[e].w)
    var parent: [V]V = V.first!
    var tree = List<(u: V, v: V, w: Int)>()
    for i in V.after(V.first!):
        g.reset()
        let w = g.max_flow(i, parent[i])
        let side = g.min_cut(i)
        for j in V.after(i):
            if side[j] and parent[j] == parent[i]: parent[j] = i
        tree.push((u: i, v: parent[i], w:))
    tree

/// A minimum cut of the undirected graph `edges` over all vertex pairs: its weight `w` and one
/// side of it, the side without `V.first`. A disconnected graph has `w == 0`.
///
/// Stoer-Wagner on a dense V x V matrix. Each phase starts from `V.first` (or the merged vertex
/// holding it) and adds the most tightly connected vertex, the least on ties; the first phase
/// whose cut is minimum gives `side`.
///
/// cost: \(O(V³)\) time, \(O(V^2)\) memory
/// order: `side` keyed by `V`
/// traps: when `V` has fewer than two positions, or some `w` is negative
/// ```cp
/// domain E = 0..2
/// domain V = 0..3
/// let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 2), (u: 1, v: 2, w: 3)]
/// global_min_cut(es) == (w: 2, side: [false, true, true])
/// ```
pub fn global_min_cut<V, E>(edges: [E](u: V, v: V, w: Int)) -> (w: Int, side: [V]Bool):
    let n = V.K
    assert n >= 2
    var mat = List.filled(n * n, 0)
    for e in E:
        let (a, b) = (edges[e].u.to_idx, edges[e].v.to_idx)
        assert edges[e].w >= 0
        if a != b:
            mat[a * n + b] += edges[e].w
            mat[b * n + a] += edges[e].w
    var group = [[i] for i in 0..n]
    var alive = List.filled(n, true)
    var best = Int.MAX
    var best_group = List<Int>()
    var key = List.filled(n, 0)
    var added = List.filled(n, false)
    for phase in 1..n:
        for i in 0..n:
            key[i] = 0
            added[i] = false
        var s = 0
        var t = 0
        for step in 0..n - phase + 1:
            var u: ?Int = None
            for i in 0..n if alive[i] and not added[i]:
                if u == None or key[i] > key[u!]: u = i
            added[u!] = true
            s = t
            t = u!
            for i in 0..n if alive[i] and not added[i]: key[i] += mat[t * n + i]
        if key[t] < best:
            best = key[t]
            best_group = group[t].copy()
        for x in group[t].copy(): group[s].push(x)
        alive[t] = false
        for i in 0..n:
            mat[s * n + i] += mat[t * n + i]
            mat[i * n + s] = mat[s * n + i]
    var side: [V]Bool = false
    for i in best_group: side[V.nth(i)] = true
    (w: best, side:)

@test
fn textbook_network():
    domain V = 0..6
    var g = FlowGraph<V>()
    let arcs = [
        g.add(V(u), V(v), c) for (u, v, c) in [
            (0, 1, 16),
            (0, 2, 13),
            (1, 2, 10),
            (2, 1, 4),
            (1, 3, 12),
            (3, 2, 9),
            (2, 4, 14),
            (4, 3, 7),
            (3, 5, 20),
            (4, 5, 4),
        ]
    ]
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
        var es: List<(Int, Int, Int)> = []
        for _ in 0..rand(0, 12):
            let (u, v, c) = (rand(0, 5), rand(0, 5), rand(0, 6))
            if u == v: continue
            g.add(V(u), V(v), c)
            es.push((u, v, c))
        let f = g.max_flow(V(0), V(5))
        var best = Int.MAX
        for m in 0..64:
            if m & 1 == 0 or m & 32 != 0: continue
            best min= sum(c for (u, v, c) in es if m.bit(u) and not m.bit(v))
        assert f == best
        let side = g.min_cut(V(0))
        assert side[V(0)] and not side[V(5)]
        assert sum(c for (u, v, c) in es if side[V(u)] and not side[V(v)]) == f
        assert sum(g.cap(a) for a in g.cut_arcs(V(0))) == f

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

@test
fn hungarian_matches_brute_force_on_random_matrices():
    domain L = 0..4
    domain R = 0..4
    for _ in 0..15:
        let w = [L] i => [R] j => rand(-50, 50)
        let h = max_weight_matching<L, R>((i, j) => w[i][j])
        var p = [0, 1, 2, 3]
        var best = Int.MIN
        while true:
            best max= sum(w[L(i)][R(p[i])] for i in 0..4)
            if not next_permutation(mut p): break
        assert h.total == best
        assert sorted([h.mate[i].to_idx for i in L]) == [0, 1, 2, 3]

@test
fn push_relabel_agrees_with_dinic_and_leaves_a_valid_flow():
    domain V = 0..7
    for _ in 0..30:
        var g = FlowGraph<V>()
        var h = FlowGraph<V>()
        var arcs: List<Arc> = []
        for _ in 0..rand(0, 15):
            let (u, v) = (V.nth(rand(0, 6)), V.nth(rand(0, 6)))
            let c = rand(0, 9)
            g.add(u, v, c)
            arcs.push(h.add(u, v, c))
        let want = g.max_flow(V(0), V(6))
        assert h.push_relabel(V(0), V(6)) == want
        var net: [V]Int = 0
        for a in arcs:
            assert 0 <= h.flow(a) <= h.cap(a)
            net[h.source(a)] -= h.flow(a)
            net[h.target(a)] += h.flow(a)
        for v in V:
            assert net[v] == (if v == V(0): -want elif v == V(6): want else: 0)
        assert h.min_cut(V(0)) == g.min_cut(V(0))

@test
fn flow_over_a_product_domain_matches_flat_numbering():
    domain R = 0..3
    domain C = 0..4
    domain P = (R, C)
    domain N = 0..12
    for _ in 0..20:
        var g = FlowGraph<P>()
        var h = FlowGraph<N>()
        for _ in 0..rand(0, 20):
            let (a, b) = (P.nth(rand(0, 11)), P.nth(rand(0, 11)))
            let c = rand(0, 5)
            g.add(a, b, c)
            h.add(N(a.to_idx), N(b.to_idx), c)
        let (s, t) = (P.first!, P.last!)
        assert g.max_flow(s, t) == h.max_flow(N(0), N(11))
        let side = g.min_cut(s)
        let flat = h.min_cut(N(0))
        for p in P: assert side[p] == flat[N(p.to_idx)]

@test
fn an_arc_without_a_capacity_is_never_cut():
    domain V = 0..4
    var g = FlowGraph<V>()
    g.add(V(0), V(1), 5)
    g.add(V(1), V(2), None)
    g.add(V(2), V(3), 3)
    assert g.max_flow(V(0), V(3)) == 3
    let side = g.min_cut(V(0))
    // the cut is the arc of capacity 3, not the one between
    assert side[V(1)] and side[V(2)] and not side[V(3)]

@test
fn reset_restores_initial_capacities():
    domain V = 0..4
    var g = FlowGraph<V>()
    g.add_both(V(0), V(1), 5)
    g.add_both(V(1), V(2), 3)
    g.add_both(V(2), V(3), 4)
    assert g.max_flow(V(0), V(3)) == 3
    g.reset()
    assert g.max_flow(V(0), V(1)) == 5
    g.reset()
    assert g.max_flow(V(0), V(3)) == 3

@test
fn a_limit_stops_the_flow_and_arcs_know_their_ends():
    domain V = 0..5
    for _ in 0..30:
        var g = FlowGraph<V>()
        var full = FlowGraph<V>()
        for _ in 0..rand(1, 10):
            let (u, v) = (V.nth(rand(0, 4)), V.nth(rand(0, 4)))
            let c = rand(0, 6)
            let a = g.add(u, v, c)
            full.add(u, v, c)
            assert g.source(a) == u and g.target(a) == v
            assert g.source(a.rev) == v and g.target(a.rev) == u and a.rev.rev == a
        let want = full.max_flow(V(0), V(4))
        let k = rand(0, want + 2)
        let first = g.max_flow(V(0), V(4), limit: k)
        assert first == min(k, want)
        assert first + g.max_flow(V(0), V(4)) == want

@test
fn min_cost_flow_agrees_with_hungarian_and_its_slope_is_convex():
    domain L = 0..4
    domain R = 0..4
    domain Node = 0..10
    for _ in 0..15:
        let w = [L] i => [R] j => rand(-50, 50)
        let h = min_weight_matching<L, R>((i, j) => w[i][j])
        let (s, t) = (Node(8), Node(9))
        var net = CostFlow<Node>()
        for i in L: net.add(s, Node(i.to_idx), 1, cost: 0)
        for j in R: net.add(Node(4 + j.to_idx), t, 1, cost: 0)
        var arcs = List<Arc>()
        for i in L:
            for j in R: arcs.push(net.add(Node(i.to_idx), Node(4 + j.to_idx), 1, cost: w[i][j]))
        let curve = net.slope(s, t)
        assert curve.first! == (flow: 0, cost: 0) and curve.last! == (flow: 4, cost: h.total)
        for k in 2..curve.len:
            let (a, b, c) = (curve[k - 2], curve[k - 1], curve[k])
            assert (b.cost - a.cost) * (c.flow - b.flow) < (c.cost - b.cost) * (b.flow - a.flow)
        assert sum(net.flow(a) * net.cost(a) for a in arcs) == h.total
        assert net.min_cost_flow(s, t) == (flow: 0, cost: 0)

@test
fn min_cost_flow_respects_the_limit_and_negative_costs():
    domain V = 0..4
    var net = CostFlow<V>()
    net.add(V(0), V(1), 2, cost: -3)
    net.add(V(1), V(3), 2, cost: 1)
    net.add(V(0), V(2), 5, cost: 2)
    net.add(V(2), V(3), None, cost: 2)
    assert net.min_cost_flow(V(0), V(3), limit: 3) == (flow: 3, cost: -4 + 4)
    assert net.min_cost_flow(V(0), V(3)) == (flow: 4, cost: 16)

@test
fn max_closure_matches_brute_force():
    domain V = 0..6
    for _ in 0..40:
        let before = [V] _ => [V.nth(rand(0, 5)) for _ in 0..rand(0, 2)]
        let p = [V] _ => rand(-6, 6)
        let got = max_closure(before, profit: v => p[v])
        var best = 0
        for m in 0..64:
            var closed = true
            for u in V:
                for v in before[u]:
                    if m.bit(v.to_idx) and not m.bit(u.to_idx): closed = false
            if closed: best max= sum(p[v] for v in V if m.bit(v.to_idx))
        assert got.value == best
        assert sum(p[v] for v in V if got.take[v]) == best
        for u in V:
            for v in before[u]: assert not got.take[v] or got.take[u]

@test
fn max_matching_matches_brute_force_and_the_cover_proves_it():
    domain L = 0..5
    domain R = 0..5
    for _ in 0..40:
        let adj = [L] _ => [R.nth(rand(0, 4)) for _ in 0..rand(0, 3)]
        let m = max_matching(adj)
        // best[mask]: the largest matching of the first left vertices into the rights in `mask`
        var best = List.filled(32, 0)
        for l in L:
            var next = best.copy()
            for mask in 0..32:
                for r in adj[l]:
                    let bit = 1 << r.to_idx
                    if mask & bit != 0: next[mask] max= best[mask ^ bit] + 1
            best = next
        assert m.size == best[31]
        assert sum(1 for l in L if m.mate[l] != None) == m.size
        for l in L:
            if let r = m.mate[l]:
                assert adj[l].contains(r) and m.back[r] == Some(l)
        let cover = min_vertex_cover(adj)
        assert sum(1 for l in L if cover.left[l]) + sum(1 for r in R if cover.right[r]) == m.size
        for l in L:
            for r in adj[l]: assert cover.left[l] or cover.right[r]

@test
fn rectangular_assignment_matches_brute_force_both_ways():
    domain L = 0..3
    domain R = 0..5
    for _ in 0..20:
        let w = [L] i => [R] j => rand(-20, 20)
        let lo = min_weight_matching<L, R>((i, j) => w[i][j])
        let hi = max_weight_matching<L, R>((i, j) => w[i][j])
        var (small, big) = (Int.MAX, Int.MIN)
        for a in R:
            for b in R:
                for c in R:
                    if a == b or b == c or a == c: continue
                    let s = w[L(0)][a] + w[L(1)][b] + w[L(2)][c]
                    small min= s
                    big max= s
        assert lo.total == small and hi.total == big
        assert lo.total == sum(w[i][lo.mate[i]] for i in L)
        assert sorted([lo.mate[i].to_idx for i in L]).len == 3

@test
fn general_matching_matches_brute_force():
    domain V = 0..8
    for _ in 0..40:
        var edge = [V] _ => [V] _ => false
        for _ in 0..rand(0, 12):
            let (u, v) = (V.nth(rand(0, 7)), V.nth(rand(0, 7)))
            if u != v:
                edge[u][v] = true
                edge[v][u] = true
        let adj = [V] u => [v for v in V if edge[u][v]]
        let got = general_matching(adj)
        // best[mask]: the largest matching inside the vertex set `mask`
        var best = List.filled(256, 0)
        for mask in 1..256:
            var low = 0
            while not mask.bit(low): low += 1
            let rest = mask ^ (1 << low)
            best[mask] = best[rest]
            for v in V:
                if rest.bit(v.to_idx) and edge[V(low)][v]:
                    best[mask] max= best[rest ^ (1 << v.to_idx)] + 1
        assert got.size == best[255]
        assert sum(1 for v in V if got.mate[v] != None) == 2 * got.size
        for v in V:
            if let u = got.mate[v]: assert edge[u][v] and got.mate[u] == Some(v)

@test
fn weighted_general_matching_matches_brute_force():
    domain V = 0..6
    for _ in 0..40:
        let es = [
            (
                u: V.nth(u),
                v: V.nth((u + rand(1, 5)) % 6),
                w: rand(0, 20),
            ) for u in [rand(0, 5) for _ in 0..rand(1, 9)]
        ]
        var wt = [V] _ => [V] _ => -1
        for e in es:
            wt[e.u][e.v] max= e.w
            wt[e.v][e.u] max= e.w
        let got = max_weight_general_matching(es)
        var best = List.filled(64, 0)
        for mask in 1..64:
            var low = 0
            while not mask.bit(low): low += 1
            let rest = mask ^ (1 << low)
            best[mask] = best[rest]
            for v in V:
                if rest.bit(v.to_idx) and wt[V(low)][v] >= 0:
                    best[mask] max= best[rest ^ (1 << v.to_idx)] + wt[V(low)][v]
        assert got.total == best[63]
        var total = 0
        for v in V:
            if let u = got.mate[v]:
                assert got.mate[u] == Some(v) and wt[u][v] >= 0
                if u.to_idx < v.to_idx: total += wt[u][v]
        assert total == got.total

fn all_bounded_flows<V, E>(
    arcs: [E](u: V, v: V, lo: Int, hi: Int),
    s: ?V,
    t: ?V,
) -> List<(Int, List<Int>)>:
    // every integer flow with lo <= flow[e] <= hi, conserved at every vertex but `s` and `t`,
    // with its net outflow at `s` (0 for a circulation)
    var out: List<(Int, List<Int>)> = []
    var f = [arcs[e].lo for e in E]
    while true:
        var net: [V]Int = 0
        for e in E:
            net[arcs[e].u] -= f[e.to_idx]
            net[arcs[e].v] += f[e.to_idx]
        if V.all(v => net[v] == 0 or Some(v) == s or Some(v) == t):
            out.push((if let x = s: -net[x] else: 0, f.copy()))
        var i = 0
        while i < E.K and f[i] == arcs[E.nth(i)].hi:
            f[i] = arcs[E.nth(i)].lo
            i += 1
        if i == E.K: break
        f[i] += 1
    out

@test
fn bounded_flows_match_brute_force():
    domain V = 0..4
    var (circ, feasible) = (0, 0)
    for _ in 0..80:
        let arcs = [
            (
                u: V.nth(rand(0, 3)),
                v: V.nth(rand(0, 3)),
                lo:,
                hi: lo + rand(0, 2),
            ) for lo in [rand(0, 5) / 4 for _ in 0..rand(1, 6)]
        ]
        let every = all_bounded_flows(arcs, None, None)
        let got = circulation(arcs)
        assert (got == None) == every.is_empty
        if let f = got:
            assert every.contains((0, f.collect()))
            circ += 1
        let (s, t) = (V.first!, V.last!)
        let st = [x for x in all_bounded_flows(arcs, s, t) if x.0 >= 0]
        let (big, small) = (max_bounded_flow(arcs, s, t), min_bounded_flow(arcs, s, t))
        assert (big == None) == st.is_empty and (small == None) == st.is_empty
        if big == None: continue
        feasible += 1
        assert big!.value == [x.0 for x in st].max()! and small!.value == [x.0 for x in st].min()!
        assert st.contains((big!.value, big!.flow.collect()))
        assert st.contains((small!.value, small!.flow.collect()))
    assert circ > 0 and feasible > 10

@test
fn gomory_hu_tree_gives_every_pairwise_min_cut():
    domain V = 0..6
    for _ in 0..30:
        let es = [
            (u: V.nth(rand(0, 5)), v: V.nth(rand(0, 5)), w: rand(0, 7)) for _ in 0..rand(0, 10)
        ]
        let tree = gomory_hu(es)
        assert tree.len == 5
        // the min cut between a and b is the lightest tree edge on their path
        for a in V:
            var light = [V] _ => Int.MAX
            var seen = [V] _ => false
            var stack: List<V> = [a]
            seen[a] = true
            while let x = stack.pop():
                for e in tree:
                    for (p, q) in [(e.u, e.v), (e.v, e.u)]:
                        if p == x and not seen[q]:
                            seen[q] = true
                            light[q] = min(light[x], e.w)
                            stack.push(q)
            for b in V:
                if a == b: continue
                var cut = Int.MAX
                for m in 0..64:
                    if m.bit(a.to_idx) and not m.bit(b.to_idx):
                        cut min= sum(
                            e.w for e in es if ((m >> e.u.to_idx) & 1) != ((m >> e.v.to_idx) & 1)
                        )
                assert seen[b] and light[b] == cut

@test
fn global_min_cut_matches_brute_force():
    domain V = 0..6
    for _ in 0..30:
        let es = [
            (u: V.nth(rand(0, 5)), v: V.nth(rand(0, 5)), w: rand(0, 7)) for _ in 0..rand(0, 10)
        ]
        let got = global_min_cut(es)
        var best = Int.MAX
        for m in 1..63:
            best min= sum(e.w for e in es if ((m >> e.u.to_idx) & 1) != ((m >> e.v.to_idx) & 1))
        assert got.w == best
        assert V.any(v => got.side[v]) and V.any(v => not got.side[v])
        assert sum(e.w for e in es if got.side[e.u] != got.side[e.v]) == best

/// \(O(E \sqrt V)\) for the first `augment_all`, \(O(V + E)\) per later `add` + `augment`.
/// A bipartite matching that grows with the graph: edges arrive online and each
/// `augment(l)` tries one augmenting path from `l` (Kuhn), so a contest that adds
/// vertices or edges between queries keeps a maximum matching without re-solving.
pub struct Matching<L, R>:
    adj: [L]List<R>
    pub mate: [L]?R
    pub back: [R]?L
    pub size: Int

    /// An online bipartite matching between `L` and `R` with no edges and nothing matched.
    ///
    /// cost: \(O(L + R)\)
    /// traps: never
    /// example: `let m = Matching<0..2, 0..2>(); m.len == 0 and m.mate(0) == None`
    pub fn new() -> Matching<L, R>:
        let adj: [L]List<R> = []
        let mate: [L]?R = None
        let back: [R]?L = None
        Matching(adj, mate, back, 0)

    /// Allows `l` to match `r`. Does not augment; a repeated edge is harmless.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(0, 1); m.len == 0 and m.augment(0) and m.mate(0) == Some(1)`
    pub fn add(mut self, l: L, r: R) = adj[l].push(r)

    /// Looks for one augmenting path from the unmatched `from` (Kuhn's DFS, trying edges in `add`
    /// order); `true` when it grew the matching. `false` also when `from` is already matched.
    ///
    /// cost: \(O(V + E)\)
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(0, 0); m.add(0, 1); m.add(1, 0); m.augment(0); m.augment(1) and m.mate(0) == Some(1) and m.mate(1) == Some(0)`
    pub fn augment(mut self, from: L) -> Bool:
        if mate[from] != None: return false
        var seen: [L]Bool = false
        var next: [L]Int = 0
        seen[from] = true
        var path = [from]
        while not path.is_empty:
            let u = path.last!
            if next[u] == adj[u].len:
                let _ = path.pop()
                continue
            let v = adj[u][next[u]]
            next[u] += 1
            if let w = back[v]:
                if not seen[w]:
                    seen[w] = true
                    path.push(w)
            else:
                flip_path(path, v, mut mate, mut back)
                size += 1
                return true
        false

    /// Grows the current matching to a maximum one by Hopcroft-Karp and returns the number of pairs
    /// added.
    ///
    /// cost: \(O(E \sqrt V)\)
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(0, 0); m.add(1, 0); m.add(1, 1); m.augment_all() == 2 and m.len == 2`
    pub fn augment_all(mut self) -> Int:
        let added = grow_matching(adj, mut mate, mut back)
        size += added
        added

    /// Unmatches `l` and its mate `r`, then tries to rematch both: `augment(l)` first, then one
    /// augmenting path ending at `r` if it is still free. Returns the new size; the matching stays
    /// maximum when it was before. No edge is removed.
    ///
    /// cost: \(O(V + E)\)
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(0, 0); m.add(1, 0); m.augment(0); m.remove(0) == 1 and m.mate(0) == Some(0)`
    pub fn remove(mut self, l: L) -> Int:
        let Some(r) = mate[l] else:
            let _ = self.augment(l)
            return size
        mate[l] = None
        back[r] = None
        size -= 1
        let _ = self.augment(l)
        if back[r] != None: return size
        var radj: [R]List<L> = []
        for u in L:
            for v in adj[u]: radj[v].push(u)
        var seen: [R]Bool = false
        var next: [R]Int = 0
        seen[r] = true
        var path = [r]
        while not path.is_empty:
            let v = path.last!
            if next[v] == radj[v].len:
                let _ = path.pop()
                continue
            let u = radj[v][next[v]]
            next[v] += 1
            if let w = mate[u]:
                if not seen[w]:
                    seen[w] = true
                    path.push(w)
            else:
                var free = u
                for i in (0..path.len).rev:
                    let y = path[i]
                    let old = back[y]
                    back[y] = Some(free)
                    mate[free] = Some(y)
                    if let o = old: free = o
                size += 1
                break
        size

    /// `l`'s partner, `None` when unmatched.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(1, 0); m.augment(1); m.mate(1) == Some(0) and m.mate(0) == None`
    pub fn mate(self, l: L) -> ?R = mate[l]
    /// `r`'s partner, `None` when unmatched.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(1, 0); m.augment(1); m.back(0) == Some(1) and m.back(1) == None`
    pub fn back(self, r: R) -> ?L = back[r]
    /// The number of matched pairs.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(1, 0); m.augment(1); m.len == 1`
    @get pub fn len(self) -> Int = size
    /// The König vertex cover of the current matching, built as in
    /// [`min_vertex_cover`](min_vertex_cover); a minimum cover when the matching is maximum.
    ///
    /// cost: \(O(V + E)\)
    /// order: `left` keyed by `L`, `right` keyed by `R`
    /// traps: never
    /// example: `var m = Matching<0..2, 0..1>(); m.add(0, 0); m.add(1, 0); m.augment_all(); m.cover() == (left: [false, false], right: [true])`
    pub fn cover(self) -> (left: [L]Bool, right: [R]Bool):
        let level = alternating_layers(adj, mate, back)
        var right: [R]Bool = false
        for u in L:
            if level[u] != None:
                for v in adj[u]: right[v] = true
        (left: [L] u => level[u] == None, right:)

/// The maximum (or, with `min: true`, minimum) total-weight matching among the listed edges, not
/// necessarily perfect: sparse weights through min-cost flow, where `max_weight_matching` needs
/// every pair's weight. `size:` forces exactly that many pairs, `None` when impossible.
///
/// One `CostFlow` on L + R + 2 vertices: each edge costs its weight times `L.count + 1`, plus 1,
/// so without `size` (where an extra zero-cost `s -> t` arc lets pairs go unused) of equal totals
/// the one with fewer pairs comes back. `used[e]` marks the chosen edges; of parallel edges at
/// most one is used.
///
/// cost: \(O(F E \log V)\), F = matching size
/// ties: the matching successive shortest paths finds, fewer pairs first
/// order: `mate` keyed by `L`, `used` keyed by `E`
/// traps: when `size` is negative, or a weight times `L.count + 1` overflows
/// ```cp
/// domain E = 0..3
/// domain L = 0..2
/// domain R = 0..2
/// let es: [E](l: L, r: R, w: Int) = [(l: 0, r: 0, w: 5), (l: 0, r: 1, w: 3), (l: 1, r: 0, w: 4)]
/// best_matching(es) == Some((total: 7, mate: [Some(1), Some(0)], used: [false, true, true]))
/// ```
/// ```cp
/// domain E = 0..3
/// domain L = 0..2
/// domain R = 0..2
/// let es: [E](l: L, r: R, w: Int) = [(l: 0, r: 0, w: 5), (l: 0, r: 1, w: 3), (l: 1, r: 0, w: 4)]
/// best_matching(es, size: Some(1)) == Some((total: 5, mate: [Some(0), None], used: [true, false, false]))
/// ```
/// ```cp
/// domain E = 0..3
/// domain L = 0..2
/// domain R = 0..2
/// let es: [E](l: L, r: R, w: Int) = [(l: 0, r: 0, w: 5), (l: 0, r: 1, w: 3), (l: 1, r: 0, w: 4)]
/// best_matching(es, size: Some(3)) == None and best_matching(es, min: true)!.total == 0
/// ```
pub fn best_matching<L, R, E>(
    edges: [E](l: L, r: R, w: Int),
    size: ?Int = None,
    min: Bool = false,
) -> ?(total: Int, mate: [L]?R, used: [E]Bool):
    if let k = size: assert k >= 0, "`best_matching` needs `size >= 0`"
    domain N = 0..L.count + R.count + 2
    let (s, t) = (N(L.count + R.count), N(L.count + R.count + 1))
    // each pair also costs 1 under a weight scale above any pair count: fewest pairs among best totals
    let scale = L.count + 1
    var g = CostFlow<N>()
    for u in L: g.add(s, N(u.to_idx), Some(1), 0)
    for v in R: g.add(N(L.count + v.to_idx), t, Some(1), 0)
    var arc: List<Arc> = []
    for e in E:
        let w = if min: edges[e].w else: -edges[e].w
        arc.push(
            g.add(N(edges[e].l.to_idx), N(L.count + edges[e].r.to_idx), Some(1), w * scale + 1)
        )
    if size == None: g.add(s, t, None, 0)
    let want = size ?? L.count
    if g.min_cost_flow(s, t, limit: Some(want)).flow < want: return None
    var mate: [L]?R = None
    var used: [E]Bool = false
    var total = 0
    for e in E:
        if g.flow(arc[e.to_idx]) > 0:
            used[e] = true
            mate[edges[e].l] = Some(edges[e].r)
            total += edges[e].w
    Some((total:, mate:, used:))

/// Gale-Shapley stable matching with proposals from `L`: `pref_l[l]` lists the `R`s `l` accepts,
/// best first, and likewise `pref_r`. A pair can match only when each lists the other. The result
/// is the stable matching best for every `l`, which is unique.
///
/// cost: \(O(L R)\)
/// empty: `[]` when `L` is empty; `None` for every `l` whose list is empty
/// traps: when a preference list names the same vertex twice
/// ```cp
/// domain L = 0..2
/// domain R = 0..2
/// let pl: [L]List<R> = [[0, 1], [0, 1]]
/// let pr: [R]List<L> = [[1, 0], [1, 0]]
/// stable_matching(pl, pr) == [Some(1), Some(0)]
/// ```
/// ```cp
/// domain L = 0..1
/// domain R = 0..1
/// let pl: [L]List<R> = [[0]]
/// let pr: [R]List<L> = [[]]
/// stable_matching(pl, pr) == [None]
/// ```
pub fn stable_matching<L, R>(pref_l: [L]List<R>, pref_r: [R]List<L>) -> [L]?R:
    var rank: [R, L]Int = L.count
    for r in R:
        for i in 0..pref_r[r].len:
            let l = pref_r[r][i]
            assert rank[r, l] == L.count
            rank[r, l] = i
    var listed: [L, R]Bool = false
    for l in L:
        for r in pref_l[l]:
            assert not listed[l, r]
            listed[l, r] = true
    var mate_l: [L]?R = None
    var mate_r: [R]?L = None
    var next: [L]Int = 0
    var free = [l for l in L]
    while let l = free.pop():
        if next[l] == pref_l[l].len: continue
        let r = pref_l[l][next[l]]
        next[l] += 1
        if rank[r, l] == L.count:
            free.push(l)
        elif let c = mate_r[r]:
            if rank[r, l] < rank[r, c]:
                mate_l[c] = None
                free.push(c)
                mate_r[r] = Some(l)
                mate_l[l] = Some(r)
            else:
                free.push(l)
        else:
            mate_r[r] = Some(l)
            mate_l[l] = Some(r)
    mate_l

/// Colours the edges of a bipartite multigraph so no two edges at a vertex share a colour, with
/// exactly max-degree colours `0..colors` (König's edge colouring theorem).
///
/// Edges are coloured in `E` order. For edge `(l, r)`, `a` is the least colour free at `l` and
/// `b` the least free at `r`; when they differ, the path from `r` alternating colours `a` and `b`
/// has them swapped, which frees `a` at `r`; then the edge takes `a`.
///
/// cost: \(O(E V)\)
/// order: `color` keyed by `E`
/// traps: never
/// ```cp
/// domain E = 0..3
/// domain L = 0..2
/// domain R = 0..2
/// let es: [E](l: L, r: R) = [(l: 0, r: 0), (l: 0, r: 1), (l: 1, r: 0)]
/// edge_coloring(es) == (colors: 2, color: [1, 0, 0])
/// ```
pub fn edge_coloring<L, R, E>(edges: [E](l: L, r: R)) -> (colors: Int, color: [E]Int):
    var deg_l: [L]Int = 0
    var deg_r: [R]Int = 0
    for e in E:
        deg_l[edges[e].l] += 1
        deg_r[edges[e].r] += 1
    let k = max(deg_l.max() ?? 0, deg_r.max() ?? 0)
    var at_l: [L]List<?E> = [L] _ => [None for _ in 0..k]
    var at_r: [R]List<?E> = [R] _ => [None for _ in 0..k]
    var color: [E]Int = 0
    for e in E:
        let (l, r) = (edges[e].l, edges[e].r)
        var a = 0
        while at_l[l][a] != None: a += 1
        var b = 0
        while at_r[r][b] != None: b += 1
        if a != b:
            var path: List<E> = []
            var y = r
            while let f = at_r[y][a]:
                path.push(f)
                let x = edges[f].l
                let Some(g) = at_l[x][b] else: break
                path.push(g)
                y = edges[g].r
            for f in path:
                at_l[edges[f].l][color[f]] = None
                at_r[edges[f].r][color[f]] = None
            for f in path:
                color[f] = if color[f] == a: b else: a
                at_l[edges[f].l][color[f]] = Some(f)
                at_r[edges[f].r][color[f]] = Some(f)
        color[e] = a
        at_l[l][a] = Some(e)
        at_r[r][a] = Some(e)
    (colors: k, color:)

fn random_prefs<A, B>() -> [A]List<B>:
    [A] _ => do:
        var p = [b for b in B if rand(0, 3) > 0]
        for i in 1..p.len:
            let j = rand(0, i)
            let t = p[i]
            p[i] = p[j]
            p[j] = t
        p

fn pref_ranks<A, B>(pref: [A]List<B>) -> [A, B]Int:
    var rk: [A, B]Int = B.count
    for a in A:
        for i in 0..pref[a].len: rk[a, pref[a][i]] = i
    rk

fn is_stable<L, R>(m: [L]?R, rl: [L, R]Int, rr: [R, L]Int) -> Bool:
    var mr: [R]?L = None
    for l in L:
        if let r = m[l]:
            if mr[r] != None or rl[l, r] == R.count or rr[r, l] == L.count: return false
            mr[r] = Some(l)
    for l in L:
        for r in R:
            if rl[l, r] == R.count or rr[r, l] == L.count: continue
            let l_wants = if let x = m[l]: rl[l, r] < rl[l, x] else: true
            let r_wants = if let y = mr[r]: rr[r, l] < rr[r, y] else: true
            if l_wants and r_wants: return false
    true

@test
fn stable_matchings_are_stable_and_proposer_optimal():
    domain L = 0..4
    domain R = 0..3
    for _ in 0..60:
        let pl = random_prefs<L, R>()
        let pr = random_prefs<R, L>()
        let (rl, rr) = (pref_ranks(pl), pref_ranks(pr))
        let m = stable_matching(pl, pr)
        assert is_stable(m, rl, rr)
        let base = R.count + 1
        for code in 0..base ** L.count:
            let other: [L]?R = [L] l => do:
                let d = code / base ** l.to_idx % base
                if d == R.count: None else: Some(R.nth(d))
            if not is_stable(other, rl, rr): continue
            for l in L:
                let mine = if let r = m[l]: rl[l, r] else: R.count
                let theirs = if let r = other[l]: rl[l, r] else: R.count
                assert mine <= theirs

@test
fn edge_colorings_use_max_degree_proper_colours():
    domain L = 0..5
    domain R = 0..4
    domain E = 0..14
    for _ in 0..100:
        let es: [E](l: L, r: R) = [E] _ => (
            l: L.nth(rand(0, L.count - 1)),
            r: R.nth(rand(0, R.count - 1)),
        )
        let (colors, color) = edge_coloring(es)
        var deg = 0
        for l in L: deg = max(deg, [e for e in E if es[e].l == l].len)
        for r in R: deg max= [e for e in E if es[e].r == r].len
        assert colors == deg
        for e in E:
            assert 0 <= color[e] and color[e] < colors
            for f in E:
                if e != f and (
                    es[e].l == es[f].l or es[e].r == es[f].r
                ): assert color[e] != color[f]

/// A largest set of elements independent in both matroids; with `weight`, the one of largest
/// total weight among the largest.
///
/// Augments along shortest paths of the exchange graph (weighted: shortest by weight, then by
/// length), breadth-first from the sources in increasing element order, so each augmentation
/// takes the least element on ties.
///
/// cost: \(O(r E (r + \mathit{oracle}))\) for answer size r, oracle the cost of one `can_add` or `can_swap`
/// order: keyed by `E`
/// traps: never
/// ```cp
/// domain E = 0..3
/// domain V = 0..3
/// domain C = 0..2
/// let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2), (u: 0, v: 2)]
/// let col: [E]C = [0, 0, 1]
/// let lim: [C]Int = [1, 1]
/// matroid_intersection<E>(GraphicMatroid(es), ColorMatroid(col, lim)) == [true, false, true]
/// ```
pub fn matroid_intersection<E, A: Matroid<E>, B: Matroid<E> > (
    a: A,
    b: B,
    weight: ?(E -> Int) = None,
) -> [E]Bool:
    var (ma, mb) = (a, b)
    var take: [E]Bool = false
    let w = weight ?? (_ => 0)
    loop:
        ma.rebuild(take)
        mb.rebuild(take)
        // Vertex cost: `-w(x)` to bring `x` in, `w(y)` to take `y` out; shortest by (cost, length).
        var dist: [E]?(Int, Int) = None
        var prev: [E]?E = None
        for x in E:
            if not take[x] and ma.can_add(x): dist[x] = Some((-w(x), 0))
        var changed = true
        while changed:
            changed = false
            for p in E:
                let Some((dc, dl)) = dist[p] else: continue
                for q in E:
                    if take[p] == take[q]: continue
                    let arc = if take[p]: ma.can_swap(p, q) else: mb.can_swap(q, p)
                    if not arc: continue
                    let nd = (dc + (if take[q]: w(q) else: -w(q)), dl + 1)
                    if dist[q] == None or nd < dist[q]!:
                        dist[q] = Some(nd)
                        prev[q] = Some(p)
                        changed = true
        var end: ?E = None
        for x in E:
            if take[x] or not mb.can_add(x) or dist[x] == None: continue
            if end == None or dist[x]! < dist[end!]!: end = Some(x)
        let Some(t) = end else: break
        var cur: ?E = Some(t)
        while let x = cur:
            take[x] = not take[x]
            cur = prev[x]
    take

/// An independence oracle for `matroid_intersection`. `rebuild(take)` sets the current
/// independent set; `can_add(e)` asks whether `take + e` stays independent, and
/// `can_swap(out, e)` whether `take - out + e` does. Graphic and colour (partition)
/// matroids ship as `GraphicMatroid` and `ColorMatroid`.
pub trait Matroid<E>:
    fn rebuild(mut self, take: [E]Bool)
    fn can_add(self, e: E) -> Bool
    fn can_swap(self, out: E, e: E) -> Bool

/// \(O(V + E)\) per rebuild. Independent sets are forests of the edges' endpoints.
pub struct GraphicMatroid<V, E>:
    edges: [E](u: V, v: V)
    var take: [E]Bool
    comp: [V]Int
    tin: [V]Int
    tout: [V]Int
    below: [E]?V
    /// The graphic matroid of `edges`: a set of edges is independent when it is a forest on `V`.
    ///
    /// cost: \(O(1)\); each `rebuild` is \(O(V + E)\), a DFS of the chosen forest
    /// traps: never
    /// ```cp
    /// domain E = 0..3
    /// domain V = 0..3
    /// let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2), (u: 0, v: 2)]
    /// var m = GraphicMatroid(es)
    /// m.rebuild([true, true, false])
    /// not m.can_add(2) and m.can_swap(0, 2)
    /// ```
    pub fn new(edges: [E](u: V, v: V)) -> GraphicMatroid<V, E> =
        GraphicMatroid(
            edges,
            [E] _ => false,
            [V] v => v.to_idx,
            [V] _ => 0,
            [V] _ => 0,
            [E] _ => None,
        )

impl Matroid<E> for GraphicMatroid<V, E>:
    fn rebuild(mut self, take: [E]Bool):
        self.take = take
        var adj: [V]List<E> = [V] _ => []
        for e in E:
            if take[e]:
                adj[edges[e].u].push(e)
                adj[edges[e].v].push(e)
        var seen: [V]Bool = false
        var clock = 0
        for r in V:
            if seen[r]: continue
            seen[r] = true
            comp[r] = r.to_idx
            tin[r] = clock
            clock += 1
            var stack: List<(V, Int)> = [(r, 0)]
            while let (x, i) = stack.pop():
                if i == adj[x].len:
                    tout[x] = clock
                    continue
                stack.push((x, i + 1))
                let e = adj[x][i]
                let y = if edges[e].u == x: edges[e].v else: edges[e].u
                if seen[y]: continue
                seen[y] = true
                comp[y] = r.to_idx
                below[e] = Some(y)
                tin[y] = clock
                clock += 1
                stack.push((y, 0))

    fn can_add(self, e: E) -> Bool = not take[e] and comp[edges[e].u] != comp[edges[e].v]

    fn can_swap(self, out: E, e: E) -> Bool:
        if not take[out] or take[e]: return false
        let (u, v) = (edges[e].u, edges[e].v)
        if comp[u] != comp[v]: return true
        let c = below[out]!
        let inside = (x: V) => tin[c] <= tin[x] and tin[x] < tout[c]
        inside(u) != inside(v)

/// \(O(E)\) per rebuild. Independent sets take at most `limit[c]` elements of each colour.
pub struct ColorMatroid<E, C>:
    color: [E]C
    limit: [C]Int
    var take: [E]Bool
    used: [C]Int
    /// The partition matroid of `color`: a set is independent when it holds at most `limit[c]`
    /// elements of each colour `c`.
    ///
    /// cost: \(O(1)\); each `rebuild` is \(O(E)\)
    /// empty: with no elements only the empty set exists and every query is `false`
    /// traps: when some `limit[c]` is negative
    /// ```cp
    /// domain C = 0..2
    /// let col: [0..3]C = [0, 0, 1]
    /// let lim: [C]Int = [1, 1]
    /// var m = ColorMatroid(col, lim)
    /// m.rebuild([true, false, false])
    /// not m.can_add(1) and m.can_add(2)
    /// ```
    pub fn new(color: [E]C, limit: [C]Int) -> ColorMatroid<E, C>:
        for c in C: assert limit[c] >= 0, "`ColorMatroid` needs every `limit[c] >= 0`"
        ColorMatroid(color, limit, [E] _ => false, [C] _ => 0)

impl Matroid<E> for ColorMatroid<E, C>:
    fn rebuild(mut self, take: [E]Bool):
        self.take = take
        for c in C: used[c] = 0
        for e in E:
            if take[e]: used[color[e]] += 1

    fn can_add(self, e: E) -> Bool = not take[e] and used[color[e]] < limit[color[e]]

    fn can_swap(self, out: E, e: E) -> Bool =
        take[out] and not take[e] and (color[out] == color[e] or used[color[e]] < limit[color[e]])

/// The minimum-cost circulation of arcs with lower and upper bounds and costs, negative cycles
/// included; `None` when no circulation meets the bounds.
///
/// Finds any circulation with `circulation`, then cancels negative cycles of its residual network
/// with `CostFlow.cancel_cycles`.
///
/// cost: one `circulation`, then one `cancel_cycles` on V vertices and 2E arcs
/// ties: among cheapest circulations, the one `cancel_cycles` finds
/// order: `flow` keyed by `E`
/// traps: when some `lo > hi`
/// ```cp
/// domain E = 0..2
/// domain V = 0..2
/// let es: [E](u: V, v: V, lo: Int, hi: Int, cost: Int) = [(u: 0, v: 1, lo: 0, hi: 2, cost: -3), (u: 1, v: 0, lo: 0, hi: 5, cost: 1)]
/// min_cost_circulation(es) == Some((cost: -4, flow: [2, 2]))
/// ```
/// ```cp
/// domain E = 0..1
/// domain V = 0..2
/// let es: [E](u: V, v: V, lo: Int, hi: Int, cost: Int) = [(u: 0, v: 1, lo: 1, hi: 1, cost: 0)]
/// min_cost_circulation(es) == None
/// ```
pub fn min_cost_circulation<V, E>(
    arcs: [E](u: V, v: V, lo: Int, hi: Int, cost: Int)
) -> ?(cost: Int, flow: [E]Int):
    let base = circulation(
        [E] e => (u: arcs[e].u, v: arcs[e].v, lo: arcs[e].lo, hi: arcs[e].hi)
    ) ?? return None
    var g = CostFlow<V>()
    var up: List<Arc> = []
    var down: List<Arc> = []
    for e in E:
        up.push(g.add(arcs[e].u, arcs[e].v, Some(arcs[e].hi - base[e]), arcs[e].cost))
        down.push(g.add(arcs[e].v, arcs[e].u, Some(base[e] - arcs[e].lo), -arcs[e].cost))
    g.cancel_cycles()
    let flow = [E] e => base[e] + g.flow(up[e.to_idx]) - g.flow(down[e.to_idx])
    var cost = 0
    for e in E: cost += flow[e] * arcs[e].cost
    Some((cost:, flow:))

@test
fn matroid_intersections_match_brute_force():
    domain V = 0..4
    domain E = 0..7
    domain C = 0..3
    for _ in 0..120:
        let es: [E](u: V, v: V) = [E] _ => (u: V.nth(rand(0, 3)), v: V.nth(rand(0, 3)))
        let col: [E]C = [E] _ => C.nth(rand(0, 2))
        let lim: [C]Int = [C] _ => rand(0, 2)
        let wt: [E]Int = [E] _ => rand(-3, 5)
        var best_n = -1
        var best_w = 0
        for mask in 0..1 << E.count:
            let s: [E]Bool = [E] e => mask.bit(e.to_idx)
            var d = Dsu<V>()
            var used: [C]Int = 0
            var ok = true
            var n = 0
            var tw = 0
            for e in E:
                if not s[e]: continue
                if not d.unite(es[e].u, es[e].v): ok = false
                used[col[e]] += 1
                if used[col[e]] > lim[col[e]]: ok = false
                n += 1
                tw += wt[e]
            if not ok: continue
            if n > best_n or (n == best_n and tw > best_w):
                best_n = n
                best_w = tw
        for weighted in [false, true]:
            let got = matroid_intersection(
                GraphicMatroid(es),
                ColorMatroid(col, lim),
                weight: if weighted: Some((e: E) => wt[e]) else: None,
            )
            var d = Dsu<V>()
            var used: [C]Int = 0
            var n = 0
            var tw = 0
            for e in E:
                if not got[e]: continue
                assert d.unite(es[e].u, es[e].v)
                used[col[e]] += 1
                assert used[col[e]] <= lim[col[e]]
                n += 1
                tw += wt[e]
            assert n == best_n
            if weighted: assert tw == best_w

@test
fn min_cost_circulations_match_brute_force():
    domain V = 0..4
    domain E = 0..5
    for _ in 0..150:
        let es: [E](u: V, v: V, lo: Int, hi: Int, cost: Int) = [E] _ => do:
            let lo = rand(0, 1)
            (
                u: V.nth(rand(0, 3)),
                v: V.nth(rand(0, 3)),
                lo:,
                hi: lo + rand(0, 2),
                cost: rand(-4, 4),
            )
        var best: ?Int = None
        var f: [E]Int = 0
        for code in 0..3 ** E.count:
            var ok = true
            var net: [V]Int = 0
            for e in E:
                f[e] = es[e].lo + code / 3 ** e.to_idx % 3
                if f[e] > es[e].hi: ok = false
                net[es[e].u] -= f[e]
                net[es[e].v] += f[e]
            if not ok or [v for v in V if net[v] != 0].len > 0: continue
            let c = sum(f[e] * es[e].cost for e in E)
            if best == None or c < best!: best = Some(c)
        let got = min_cost_circulation(es)
        assert (got?.cost) == best
        if let r = got:
            var net: [V]Int = 0
            for e in E:
                assert es[e].lo <= r.flow[e] and r.flow[e] <= es[e].hi
                net[es[e].u] -= r.flow[e]
                net[es[e].v] += r.flow[e]
            for v in V: assert net[v] == 0
