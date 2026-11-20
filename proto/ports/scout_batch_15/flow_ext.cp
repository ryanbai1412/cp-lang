//! Min-cost max-flow and weighted bipartite matching.
//! Proposed additions to `std.flow` (or `std.matching`).

/// Handle to a directed arc inside a `MinCostFlow`.
pub struct CostArc(id: Int):
    pub @get fn rev(self) -> CostArc = CostArc(id ^ 1)

/// Directed network with integer capacities and signed integer costs per unit.
/// Successive shortest paths with Johnson potentials: SPFA initializes potentials
/// when negative edges are present, then each augmentation runs Dijkstra.
pub struct MinCostFlow<V>:
    head: [V]List<CostArc>
    to: List<V>
    left: List<Int>
    cost: List<Int>

    pub fn new() -> MinCostFlow<V>:
        MinCostFlow([V] v => [], [], [], [])

    pub fn add(mut self, u: V, v: V, cap: Int, w: Int) -> CostArc:
        let id = to.len
        to.push(v)
        left.push(cap)
        cost.push(w)
        head[u].push(CostArc(id))
        to.push(u)
        left.push(0)
        cost.push(-w)
        head[v].push(CostArc(id + 1))
        CostArc(id)

    /// Push up to `limit` units of flow from `s` to `t` at minimum total cost.
    /// Pass `limit = Int.MAX` for a full minimum-cost maximum flow.
    pub fn min_cost_flow(mut self, s: V, t: V, limit: Int) -> (flow: Int, cost: Int):
        var pot: [V]?Int = none
        // Initialize potentials with SPFA if any positive-capacity arc has negative cost.
        var has_neg = false
        for i in 0..left.len:
            if left[i] > 0 and cost[i] < 0: has_neg = true
        if has_neg:
            pot[s] = 0
            var in_q: [V]Bool = false
            var q = [s]
            var qi = 0
            in_q[s] = true
            while qi < q.len:
                let u = q[qi]
                qi += 1
                in_q[u] = false
                let du = pot[u]!
                for a in head[u]:
                    if left[a.id] <= 0: continue
                    let v = to[a.id]
                    let nd = du + cost[a.id]
                    if pot[v] == none or nd < pot[v]!:
                        pot[v] = nd
                        if not in_q[v]:
                            in_q[v] = true
                            q.push(v)
        else:
            for v in V: pot[v] = 0

        var (pushed, total_cost) = (0, 0)
        while pushed < limit:
            var dist: [V]?Int = none
            var prev_v: [V]?V = none
            var prev_a: [V]?CostArc = none
            dist[s] = 0
            var heap = MinHeap<(Int, V)>()
            heap.push((0, s))
            while let (d, u) = heap.pop():
                if d > dist[u]!: continue
                let pu = pot[u] ?? 0
                for a in head[u]:
                    if left[a.id] <= 0: continue
                    let v = to[a.id]
                    let pv = pot[v] ?? 0
                    let nd = d + cost[a.id] + pu - pv
                    if dist[v] == none or nd < dist[v]!:
                        dist[v] = nd
                        prev_v[v] = u
                        prev_a[v] = a
                        heap.push((nd, v))
            if dist[t] == none: break
            for v in V:
                if let dv = dist[v]:
                    pot[v] = (pot[v] ?? 0) + dv
            var add_f = limit - pushed
            var cur = t
            while cur != s:
                let a = prev_a[cur]!
                add_f min= left[a.id]
                cur = prev_v[cur]!
            cur = t
            while cur != s:
                let a = prev_a[cur]!
                left[a.id] -= add_f
                left[a.rev.id] += add_f
                cur = prev_v[cur]!
            pushed += add_f
            total_cost += add_f * (pot[t]! - (pot[s] ?? 0))
        (flow: pushed, cost: total_cost)

/// Maximum-weight perfect bipartite matching for equal-sized domains `L` and `R`.
/// Classic O(n^3) Hungarian algorithm with integer weights; returns `(total, mate)`
/// where `mate[i]` is the partner of `i` in `R`.
pub fn max_weight_matching<L, R>(weight: (L, R) -> Int) -> (total: Int, mate: [L]R):
    let n = L.K
    // 1-based sentinel row/col 0 for augmenting-path bookkeeping.
    var u = List.filled(n + 1, 0)
    var v = List.filled(n + 1, 0)
    var p = List.filled(n + 1, 0)
    var way = List.filled(n + 1, 0)
    // Maximize weight by minimizing -weight.
    for i in 1..=n:
        p[0] = i
        var j0 = 0
        var minv = List.filled(n + 1, Int.MAX)
        var used = List.filled(n + 1, false)
        while true:
            used[j0] = true
            let i0 = p[j0]
            var delta = Int.MAX
            var j1 = 0
            for j in 1..=n:
                if used[j]: continue
                let cur = -weight(L.nth(i0 - 1), R.nth(j - 1)) - u[i0] - v[j]
                if cur < minv[j]:
                    minv[j] = cur
                    way[j] = j0
                if minv[j] < delta:
                    delta = minv[j]
                    j1 = j
            for j in 0..=n:
                if used[j]:
                    u[p[j]] += delta
                    v[j] -= delta
                else:
                    minv[j] -= delta
            j0 = j1
            if p[j0] == 0: break
        while true:
            let j1 = way[j0]
            p[j0] = p[j1]
            j0 = j1
            if j0 == 0: break
    var row_to_col = List.filled(n, 0)
    for j in 1..=n: row_to_col[p[j] - 1] = j - 1
    let mate = [L] i => R.nth(row_to_col[i.to_idx])
    let total = sum(weight(i, mate[i]) for i in L)
    (total: total, mate: mate)

@test
fn mcmf_and_hungarian_agree_on_random_matrices():
    domain L = 0..4
    domain R = 0..4
    for _ in 0..10:
        let w = [L] i => [R] j => rand(-50, 50)
        let h = max_weight_matching::<L, R>((i, j) => w[i][j])
        // Solve the same instance as min-cost max-flow with negated weights.
        domain Node = 0..10
        let (src, snk) = (Node(8), Node(9))
        var net = MinCostFlow<Node>.new()
        for i in L: net.add(src, Node(i.to_idx), 1, 0)
        for j in R: net.add(Node(4 + j.to_idx), snk, 1, 0)
        for i in L:
            for j in R:
                net.add(Node(i.to_idx), Node(4 + j.to_idx), 1, -w[i][j])
        let res = net.min_cost_flow(src, snk, 4)
        assert res.flow == 4 and -res.cost == h.total
