//! Flows, cuts and matchings: Dinic's max flow and min cut, min-cost flow, maximum
//! closure, bipartite and general matching (weighted too), assignment, bounded flows,
//! Gomory-Hu and global min cut.
//!
//! Two networks, `FlowGraph` and `CostFlow`, share `Arc` and the accessors `flow`, `cap`,
//! `source`, `target`. Everything whose input is a table or an edge list is a function
//! over the caller's domains: a matching is `[L]?R`, a bounded flow is `[E]Int`.
use std.heap.{MinHeap}
use std.seq.{next_permutation}

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

    /// An empty network. O(V).
    pub fn new() -> FlowGraph<V>:
        FlowGraph([V] v => [], [], [], [])

    /// Add an arc `u -> v` with capacity `cap` (and its reverse with capacity 0). O(1).
    /// A `cap` of `none` is an arc no cut can afford: its capacity is 2^60.
    pub fn add(mut self, u: V, v: V, cap: ?Int) -> Arc:
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
        a

    /// Add an undirected edge: `cap` in both directions. O(1).
    pub fn add_both(mut self, u: V, v: V, cap: Int) -> Arc:
        let a = self.add(u, v, cap)
        left[a.rev.id] = cap
        cap0[a.rev.id] = cap
        a

    /// The flow currently sent along `a` (negative on the reverse arc of a used arc).
    pub fn flow(self, a: Arc) -> Int = cap0[a.id] - left[a.id]

    /// The capacity `a` was added with.
    pub fn cap(self, a: Arc) -> Int = cap0[a.id]

    /// The vertex `a` leaves.
    pub fn source(self, a: Arc) -> V = to[a.id ^ 1]

    /// The vertex `a` points to.
    pub fn target(self, a: Arc) -> V = to[a.id]

    /// Push as much as possible from `s` to `t`, on top of the flow already there, but
    /// no more than `limit`. Returns the amount added. O(V^2 E); O(E sqrt(V)) when every
    /// capacity is 1.
    pub fn max_flow(mut self, s: V, t: V, limit: ?Int = none) -> Int:
        assert s != t
        let limit = limit ?? Int.MAX
        var total = 0
        while total < limit:
            let level = self.levels(s)
            if level[t] == none: break
            var next: [V]Int = 0
            while total < limit:
                let pushed = self.augment(s, t, level, mut next, limit - total)
                if pushed == 0: break
                total += pushed
        total

    /// The source side of a minimum cut, after `max_flow(s, t)` without a limit: the
    /// vertices `s` still reaches in the residual network. O(V + E).
    pub fn min_cut(self, s: V) -> [V]Bool:
        let level = self.levels(s)
        [V] v => level[v] != none

    /// Clear all flow, restoring every arc to its initial capacity. O(E).
    pub fn reset(mut self):
        for k in 0..left.len: left[k] = cap0[k]

    fn close(mut self, a: Arc):
        left[a.id] = 0
        left[a.rev.id] = 0

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
    fn augment(mut self, s: V, t: V, level: [V]?Int, mut next: [V]Int, most: Int) -> Int:
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
        let amount = min(most, min(left[a.id] for a in path))
        for a in path:
            left[a.id] -= amount
            left[a.rev.id] += amount
        amount

/// A flow network with a cost per unit on every arc, solved by successive shortest paths
/// with Johnson potentials. Arc handles and accessors are those of `FlowGraph`.
/// `add` is O(1); memory is two arcs per `add`.
pub struct CostFlow<V>:
    head: [V]List<Arc>
    to: List<V>
    left: List<Int>
    cap0: List<Int>
    price: List<Int>

    /// An empty network. O(V).
    pub fn new() -> CostFlow<V>:
        CostFlow([V] v => [], [], [], [], [])

    /// Add an arc `u -> v` with capacity `cap` and cost `cost` per unit; its reverse has
    /// capacity 0 and cost `-cost`. `cap: none` is 2^60. Costs may be negative as long
    /// as no cycle of positive capacity has negative total cost. O(1).
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

    /// The flow currently sent along `a`.
    pub fn flow(self, a: Arc) -> Int = cap0[a.id] - left[a.id]

    /// The capacity `a` was added with.
    pub fn cap(self, a: Arc) -> Int = cap0[a.id]

    /// The cost per unit of flow along `a`.
    pub fn cost(self, a: Arc) -> Int = price[a.id]

    /// The vertex `a` leaves.
    pub fn source(self, a: Arc) -> V = to[a.id ^ 1]

    /// The vertex `a` points to.
    pub fn target(self, a: Arc) -> V = to[a.id]

    /// Send the most flow from `s` to `t`, up to `limit`, at the least cost among flows of
    /// that amount, on top of the flow already there. Returns the amount and cost added.
    /// O(F E log V) for F augmenting paths, plus O(V E) once when a residual arc has a
    /// negative cost (the first call with negative costs, and every later call).
    pub fn min_cost_flow(mut self, s: V, t: V, limit: ?Int = none) -> (flow: Int, cost: Int):
        let curve = self.slope(s, t, limit)
        curve.last!

    /// The breakpoints of the cheapest cost of sending each amount from `s` to `t`, up to
    /// `limit`: `(0, 0)` first, then one `(flow, cost)` per change of slope; between
    /// breakpoints the cost is linear and the slopes never decrease. Same cost as
    /// `min_cost_flow`.
    pub fn slope(mut self, s: V, t: V, limit: ?Int = none) -> List<(flow: Int, cost: Int)>:
        assert s != t
        let limit = limit ?? Int.MAX
        var pot = self.potentials()
        var curve = [(flow: 0, cost: 0)]
        var (flow, total) = (0, 0)
        var last_unit: ?Int = none
        while flow < limit:
            var dist: [V]?Int = none
            var via: [V]?Arc = none
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
                    if dist[v] == none or nd < dist[v]!:
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
            curve.push((flow: flow, cost: total))
            last_unit = unit
        curve

    /// Potentials under which every residual arc has a nonnegative reduced cost: all zero
    /// when no residual arc is negative, else Bellman-Ford from every vertex at once.
    fn potentials(self) -> [V]Int:
        var pot: [V]Int = 0
        if not any(left[k] > 0 and price[k] < 0 for k in 0..left.len): return pot
        var rounds = 0
        var changed = true
        while changed:
            changed = false
            rounds += 1
            assert rounds <= V.K + 1
            for u in V:
                for a in head[u]:
                    if left[a.id] > 0 and pot[u] + price[a.id] < pot[to[a.id]]:
                        pot[to[a.id]] = pot[u] + price[a.id]
                        changed = true
        pot

/// A maximum-weight closure: the set `take` of vertices with the largest total `profit`
/// such that taking `needs[e].u` forces taking `needs[e].v`. Profits may be negative; the
/// empty set is a closure, so `value >= 0`. One `max_flow` on V + 2 vertices and
/// V + E arcs (project selection).
pub fn max_closure<V, E>(profit: [V]Int, needs: [E](u: V, v: V)) -> (value: Int, take: [V]Bool):
    domain N = 0..V.K + 2
    let (s, t) = (N(V.K), N(V.K + 1))
    var g = FlowGraph<N>()
    var gain = 0
    for v in V:
        if profit[v] > 0:
            g.add(s, N(v.to_idx), profit[v])
            gain += profit[v]
        elif profit[v] < 0:
            g.add(N(v.to_idx), t, -profit[v])
    for e in E: g.add(N(needs[e].u.to_idx), N(needs[e].v.to_idx), cap: none)
    let value = gain - g.max_flow(s, t)
    let side = g.min_cut(s)
    (value: value, take: [V] v => side[N(v.to_idx)])

/// A maximum matching between `L` and `R`, where `adj[l]` lists the partners `l` may take:
/// its `size`, and the partner of each vertex on either side (`none` when unmatched).
/// Hopcroft-Karp, O(E sqrt(V)) time, no recursion.
pub fn max_matching<L, R>(adj: [L]List<R>) -> (size: Int, mate: [L]?R, back: [R]?L):
    var mate: [L]?R = none
    var back: [R]?L = none
    var size = 0
    for u in L:
        for v in adj[u]:
            if back[v] == none:
                mate[u] = v
                back[v] = u
                size += 1
                break
    while true:
        var level = alternating_layers(adj, mate, back)
        if not any(level[u] != none and any(back[v] == none for v in adj[u]) for u in L): break
        var next: [L]Int = 0
        for root in L:
            if mate[root] != none: continue
            var path = [root]
            while not path.is_empty:
                let u = path.last!
                if next[u] == adj[u].len:
                    level[u] = none
                    path.pop()
                    continue
                let v = adj[u][next[u]]
                next[u] += 1
                if let w = back[v]:
                    if level[w] == level[u]! + 1: path.push(w)
                else:
                    var free = v
                    for i in (0..path.len).rev:
                        let x = path[i]
                        let old = mate[x]
                        mate[x] = free
                        back[free] = x
                        if old != none: free = old!
                    size += 1
                    path = []
    (size: size, mate: mate, back: back)

/// A minimum vertex cover of the bipartite graph `adj`: every edge has a marked end, and
/// exactly `max_matching(adj).size` vertices are marked (Konig). The unmarked vertices are
/// a maximum independent set. O(E sqrt(V)).
pub fn min_vertex_cover<L, R>(adj: [L]List<R>) -> (left: [L]Bool, right: [R]Bool):
    let m = max_matching(adj)
    let level = alternating_layers(adj, m.mate, m.back)
    var right: [R]Bool = false
    for u in L:
        if level[u] != none:
            for v in adj[u]: right[v] = true
    (left: [L] u => level[u] == none, right: right)

/// Breadth-first layers of the left side by alternating paths: free vertices are layer 0,
/// and a matched vertex is one deeper than a left vertex adjacent to its partner.
fn alternating_layers<L, R>(adj: [L]List<R>, mate: [L]?R, back: [R]?L) -> [L]?Int:
    var level: [L]?Int = none
    var queue = [u for u in L if mate[u] == none]
    for u in queue: level[u] = 0
    var at = 0
    while at < queue.len:
        let u = queue[at]
        at += 1
        for v in adj[u]:
            if let w = back[v]:
                if level[w] == none:
                    level[w] = level[u]! + 1
                    queue.push(w)
    level

/// The maximum-weight assignment: every `l` in `L` gets a distinct `mate[l]` in `R`
/// (needs `L.K <= R.K`), maximizing `total`, the sum of `weight(l, mate[l])`.
/// Hungarian algorithm, O(L^2 R) time, O(L R) calls of `weight`.
pub fn max_weight_matching<L, R>(weight: (L, R) -> Int) -> (total: Int, mate: [L]R):
    let mate = hungarian::<L, R>((l, r) => -weight(l, r))
    (total: sum(weight(l, mate[l]) for l in L), mate: mate)

/// The minimum-weight assignment: every `l` in `L` gets a distinct `mate[l]` in `R`
/// (needs `L.K <= R.K`), minimizing `total`, the sum of `weight(l, mate[l])`.
/// Hungarian algorithm, O(L^2 R) time, O(L R) calls of `weight`.
pub fn min_weight_matching<L, R>(weight: (L, R) -> Int) -> (total: Int, mate: [L]R):
    let mate = hungarian::<L, R>(weight)
    (total: sum(weight(l, mate[l]) for l in L), mate: mate)

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

/// A maximum matching in the undirected graph `adj` (each edge listed from both ends):
/// its `size` and each vertex's partner. Edmonds' blossom algorithm, O(V^3).
pub fn general_matching<V>(adj: [V]List<V>) -> (size: Int, mate: [V]?V):
    let n = V.K
    let nb = [[w.to_idx for w in adj[V.nth(i)]] for i in 0..n]
    var mate = List.filled(n, -1)
    var size = 0
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
    (size: size, mate: [V] v => if mate[v.to_idx] == -1: none else: V.nth(mate[v.to_idx]))

/// Weighted blossom state over vertices 1..=n and blossoms n+1..=2n; index 0 is "none".
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

    fn delta(self, k: Int) -> Int = lab[eu[k]] + lab[ev[k]] - 2 * ew[k]

    fn update_slack(mut self, u: Int, x: Int):
        if slack[x] == 0 or self.delta(u * m + x) < self.delta(slack[x] * m + x): slack[x] = u

    fn set_slack(mut self, x: Int):
        slack[x] = 0
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

/// A maximum-weight matching in the undirected graph `edges`: no vertex in two chosen
/// edges, maximizing `total`, the sum of chosen `w`. Edges with `w <= 0` never help and
/// are ignored; of parallel edges the heaviest counts. Edmonds' weighted blossom
/// algorithm with dual labels, O(V^3) time, O(V^2) memory.
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
        [if 1 <= i and i <= n: top else: 0 for i in 0..m],
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
    var total = 0
    for u in 1..=n:
        if g.mate[u] > u: total += g.ew[u * m + g.mate[u]]
    (
        total: total,
        mate: [V] v => if g.mate[v.to_idx + 1] == 0: none else: V.nth(g.mate[v.to_idx + 1] - 1),
    )

/// A flow `f` on every arc with `lo <= f[e] <= hi` that is conserved at every vertex, or
/// `none` when there is none. One `max_flow` on V + 2 vertices and E + V arcs.
pub fn circulation<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int)) -> ?[E]Int:
    let r = bounded_flow(arcs, none, 0) ?? return none
    r.flow

/// The largest `s -> t` flow with `lo <= flow[e] <= hi` on every arc, conserved at every
/// vertex but `s` and `t`, or `none` when no such flow exists. `value` is the net flow out
/// of `s`. Two `max_flow`s on V + 2 vertices and E + V + 1 arcs.
pub fn max_bounded_flow<V, E>(
    arcs: [E](u: V, v: V, lo: Int, hi: Int),
    s: V,
    t: V,
) -> ?(value: Int, flow: [E]Int):
    bounded_flow(arcs, (s, t), 1)

/// The smallest `s -> t` flow with `lo <= flow[e] <= hi` on every arc, conserved at every
/// vertex but `s` and `t`, never sending more back into `s` than leaves it (`value >= 0`),
/// or `none` when no such flow exists. Same cost as
/// `max_bounded_flow`.
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
    let back = if let (s, t) = ends: g.add(N(t.to_idx), N(s.to_idx), cap: none) else: Arc(-1)
    if g.max_flow(ss, tt) != demand: return none
    if let (s, t) = ends:
        let (ns, nt) = (N(s.to_idx), N(t.to_idx))
        let base = g.flow(back)
        g.close(back)
        let extra = if sense > 0: g.max_flow(ns, nt) else: -g.max_flow(nt, ns, limit: base)
        return (value: base + extra, flow: [E] e => arcs[e].lo + g.flow(ids[e]))
    (value: 0, flow: [E] e => arcs[e].lo + g.flow(ids[e]))

/// A Gomory-Hu tree of the undirected graph `edges`: `V.K - 1` edges on `V` such that for
/// every `s != t` the minimum `s-t` cut of `edges` is the smallest `w` on the tree path
/// from `s` to `t`. Vertices in different components are joined by `w = 0`. Gusfield's
/// algorithm: `V.K - 1` max flows on one reused network, each O(V^2 E).
pub fn gomory_hu<V, E>(edges: [E](u: V, v: V, w: Int)) -> List<(u: V, v: V, w: Int)>:
    var g = FlowGraph<V>()
    for e in E: g.add_both(edges[e].u, edges[e].v, edges[e].w)
    var parent: [V]V = V.first
    var tree = List<(u: V, v: V, w: Int)>()
    for i in V.after(V.first):
        g.reset()
        let w = g.max_flow(i, parent[i])
        let side = g.min_cut(i)
        for j in V.after(i):
            if side[j] and parent[j] == parent[i]: parent[j] = i
        tree.push((u: i, v: parent[i], w: w))
    tree

/// A minimum cut of the undirected graph `edges` over all vertex pairs: its weight `w`
/// and one side of it. Stoer-Wagner on a dense V x V matrix, O(V^3) time, O(V^2) memory.
/// Needs at least two vertices; weights are non-negative.
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
            var u: ?Int = none
            for i in 0..n if alive[i] and not added[i]:
                if u == none or key[i] > key[u!]: u = i
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
    (w: best, side: side)

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
        let es = [(V.nth(rand(0, 5)), V.nth(rand(0, 5)), rand(0, 8)) for _ in 0..12]
        let arcs = [g.add(u, v, c) for (u, v, c) in es]
        let f = g.max_flow(V.first, V.last)
        let side = g.min_cut(V.first)
        assert side[V.first] and not side[V.last]
        assert f == sum(c for (u, v, c) in es if side[u] and not side[v])
        // brute force: the smallest cut over all 16 source sides
        var best = Int.MAX
        for mask in 0..16:
            let inside = [V] v => v == V.first or (
                v != V.last and ((mask >> (v.to_idx - 1)) & 1) == 1
            )
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

@test
fn hungarian_matches_brute_force_on_random_matrices():
    domain L = 0..4
    domain R = 0..4
    for _ in 0..15:
        let w = [L] i => [R] j => rand(-50, 50)
        let h = max_weight_matching::<L, R>((i, j) => w[i][j])
        var p = [0, 1, 2, 3]
        var best = Int.MIN
        while true:
            best max= sum(w[L(i)][R(p[i])] for i in 0..4)
            if not next_permutation(mut p): break
        assert h.total == best
        assert sorted([h.mate[i].to_idx for i in L]) == [0, 1, 2, 3]

@test
fn an_arc_without_a_capacity_is_never_cut():
    domain V = 0..4
    var g = FlowGraph<V>()
    g.add(V(0), V(1), 5)
    g.add(V(1), V(2), none)
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
    for _ in 0..20:
        let es = [(V.nth(rand(0, 4)), V.nth(rand(0, 4)), rand(0, 6)) for _ in 0..10]
        var full = FlowGraph<V>()
        for (u, v, c) in es: full.add(u, v, c)
        let best = full.max_flow(V.first, V.last)
        let k = rand(0, 12)
        var g = FlowGraph<V>()
        let arcs = [g.add(u, v, c) for (u, v, c) in es]
        assert g.max_flow(V.first, V.last, limit: k) == min(k, best)
        assert g.max_flow(V.first, V.last) == best - min(k, best)
        for (a, (u, v, _)) in zip(arcs, es):
            assert g.source(a) == u and g.target(a) == v and g.source(a.rev) == v

@test
fn min_cost_flow_agrees_with_hungarian_and_its_slope_is_convex():
    domain L = 0..4
    domain R = 0..4
    domain Node = 0..10
    for _ in 0..15:
        let w = [L] i => [R] j => rand(-50, 50)
        let h = min_weight_matching::<L, R>((i, j) => w[i][j])
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
    net.add(V(2), V(3), none, cost: 2)
    assert net.min_cost_flow(V(0), V(3), limit: 3) == (flow: 3, cost: -4 + 4)
    assert net.min_cost_flow(V(0), V(3)) == (flow: 4, cost: 16)

@test
fn max_closure_matches_brute_force():
    domain V = 0..6
    for _ in 0..40:
        let profit = [V] _ => rand(-6, 6)
        let needs = [(u: V.nth(rand(0, 5)), v: V.nth(rand(0, 5))) for _ in 0..rand(0, 8)]
        let got = max_closure(profit, needs)
        assert all(got.take[e.v] for e in needs if got.take[e.u])
        assert got.value == sum(profit[v] for v in V if got.take[v])
        var best = 0
        for mask in 0..1 << V.K:
            let take = [V] v => ((mask >> v.to_idx) & 1) == 1
            if all(take[e.v] for e in needs if take[e.u]):
                best max= sum(profit[v] for v in V if take[v])
        assert got.value == best

@test
fn max_matching_matches_brute_force_and_the_cover_proves_it():
    domain L = 0..6
    domain R = 0..5
    for _ in 0..40:
        let p = rand(1, 4)
        let adj = [L] u => [v for v in R if rand(0, 4) < p]
        let m = max_matching(adj)
        var best = List.filled(1 << R.K, -1)
        best[0] = 0
        for u in L:
            for mask in (0..1 << R.K).rev:
                if best[mask] < 0: continue
                for v in adj[u]:
                    let bit = 1 << v.to_idx
                    if mask & bit == 0: best[mask | bit] max= best[mask] + 1
        assert m.size == max(best)
        assert m.size == sum(1 for u in L if m.mate[u] != none)
        for u in L:
            if let v = m.mate[u]: assert m.back[v] == u and adj[u].contains(v)
        let cover = min_vertex_cover(adj)
        assert all(cover.left[u] or cover.right[v] for u in L for v in adj[u])
        assert sum(1 for u in L if cover.left[u]) + sum(1 for v in R if cover.right[v]) == m.size

@test
fn rectangular_assignment_matches_brute_force_both_ways():
    domain L = 0..3
    domain R = 0..5
    for _ in 0..20:
        let w = [L] i => [R] j => rand(-20, 20)
        let lo = min_weight_matching::<L, R>((i, j) => w[i][j])
        let hi = max_weight_matching::<L, R>((i, j) => w[i][j])
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
        var adj: [V]List<V> = []
        var es = List<(V, V)>()
        for u in V:
            for v in V.after(u):
                if rand(0, 3) == 0:
                    adj[u].push(v)
                    adj[v].push(u)
                    es.push((u, v))
        let m = general_matching(adj)
        for v in V:
            if let w = m.mate[v]: assert m.mate[w] == v and adj[v].contains(w)
        assert m.size * 2 == sum(1 for v in V if m.mate[v] != none)
        var best = List.filled(1 << V.K, 0)
        for mask in 1..1 << V.K:
            var lo = 0
            while (mask >> lo) & 1 == 0: lo += 1
            best[mask] = best[mask ^ (1 << lo)]
            for (u, v) in es:
                let bits = (1 << u.to_idx) | (1 << v.to_idx)
                if u.to_idx == lo and mask & bits == bits: best[mask] max= best[mask ^ bits] + 1
        assert m.size == best[(1 << V.K) - 1]

@test
fn weighted_general_matching_matches_brute_force():
    domain V = 0..8
    for _ in 0..60:
        let cnt = rand(0, 20)
        domain E = 0..cnt
        let edges = [E] _ => (u: V.nth(rand(0, 7)), v: V.nth(rand(0, 7)), w: rand(-3, 20))
        let m = max_weight_general_matching(edges)
        var got = 0
        for v in V:
            if let w = m.mate[v]:
                assert m.mate[w] == v and w != v
                if v < w: got += max(
                    edges[e].w for e in E if (edges[e].u, edges[e].v) == (v, w) or (
                        edges[e].u,
                        edges[e].v,
                    ) == (w, v)
                )
        assert got == m.total
        var best = List.filled(1 << V.K, 0)
        for mask in 1..1 << V.K:
            var lo = 0
            while (mask >> lo) & 1 == 0: lo += 1
            best[mask] = best[mask ^ (1 << lo)]
            for e in E:
                let (u, v) = (edges[e].u.to_idx, edges[e].v.to_idx)
                let bits = (1 << u) | (1 << v)
                if u != v and (u == lo or v == lo) and mask & bits == bits:
                    best[mask] max= best[mask ^ bits] + edges[e].w
        assert m.total == best[(1 << V.K) - 1]

fn all_bounded_flows<V, E>(
    arcs: [E](u: V, v: V, lo: Int, hi: Int),
    s: ?V,
    t: ?V,
) -> List<(Int, List<Int>)>:
    var found: List<(Int, List<Int>)> = []
    var f = [arcs[e].lo for e in E]
    while true:
        var net: [V]Int = 0
        for e in E:
            net[arcs[e].u] += f[e.to_idx]
            net[arcs[e].v] -= f[e.to_idx]
        if all(net[v] == 0 for v in V if v != s and v != t):
            found.push((if s == none: 0 else: net[s!], [x for x in f]))
        var i = 0
        while i < E.K and f[i] == arcs[E.nth(i)].hi:
            f[i] = arcs[E.nth(i)].lo
            i += 1
        if i == E.K: break
        f[i] += 1
    found

@test
fn bounded_flows_match_brute_force():
    domain V = 0..4
    var (circ, feasible) = (0, 0)
    for _ in 0..80:
        let arcs = [
            (
                u: V.nth(rand(0, 3)),
                v: V.nth(rand(0, 3)),
                lo: lo,
                hi: lo + rand(0, 2),
            ) for lo in [rand(0, 5) / 4 for _ in 0..rand(1, 6)]
        ]
        let every = all_bounded_flows(arcs, none, none)
        let got = circulation(arcs)
        assert (got == none) == every.is_empty
        if let f = got:
            assert every.contains((0, [x for x in f]))
            circ += 1
        let (s, t) = (V.first, V.last)
        let st = [x for x in all_bounded_flows(arcs, s, t) if x.0 >= 0]
        let (big, small) = (max_bounded_flow(arcs, s, t), min_bounded_flow(arcs, s, t))
        assert (big == none) == st.is_empty and (small == none) == st.is_empty
        if big == none: continue
        feasible += 1
        assert big!.value == max(x.0 for x in st) and small!.value == min(x.0 for x in st)
        assert st.contains((big!.value, [x for x in big!.flow]))
        assert st.contains((small!.value, [x for x in small!.flow]))
    assert circ > 0 and feasible > 10

@test
fn gomory_hu_tree_gives_every_pairwise_min_cut():
    domain V = 0..7
    for _ in 0..20:
        let edges = [
            (u: V.nth(rand(0, 6)), v: V.nth(rand(0, 6)), w: rand(0, 9)) for _ in 0..rand(0, 12)
        ]
        let tree = gomory_hu(edges)
        assert tree.len == V.K - 1
        for s in V:
            // smallest weight on the tree path from s, by a walk over the tree
            var best: [V]?Int = none
            best[s] = Int.MAX
            var stack = [s]
            while not stack.is_empty:
                let x = stack.pop()
                for e in tree:
                    for (a, b) in [(e.u, e.v), (e.v, e.u)]:
                        if a == x and best[b] == none:
                            best[b] = min(best[x]!, e.w)
                            stack.push(b)
            for t in V.after(s):
                var g = FlowGraph<V>()
                for e in edges: g.add_both(e.u, e.v, e.w)
                assert best[t] == g.max_flow(s, t)

@test
fn global_min_cut_matches_brute_force():
    domain V = 0..6
    for _ in 0..30:
        let edges = [
            (u: V.nth(rand(0, 5)), v: V.nth(rand(0, 5)), w: rand(0, 9)) for _ in 0..rand(0, 12)
        ]
        let (w, side) = global_min_cut(edges)
        var crossing = 0
        for e in edges if side[e.u] != side[e.v]: crossing += e.w
        assert crossing == w
        let taken = [v for v in V if side[v]].len
        assert 0 < taken and taken < V.K
        var best = Int.MAX
        for mask in 1..(1 << V.K) - 1:
            var cut = 0
            for e in edges if (mask >> e.u.to_idx) % 2 != (mask >> e.v.to_idx) % 2: cut += e.w
            best = min(best, cut)
        assert w == best
