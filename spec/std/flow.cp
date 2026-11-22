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
pub struct Arc(id: 0..2_147_483_648):
    /// The paired arc in the opposite direction, what `e ^ 1` means in C++. Every `add` makes an
    /// arc and its reverse together, so `a.rev.rev == a`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(5)); g.source(a.rev) == 1 and a.rev.rev == a`
    pub @get fn rev(self) -> Arc = todo

/// A flow network over the vertices `V`, solved with Dinic's algorithm (`max_flow`) or push-relabel (`push_relabel`).
/// `add` is O(1). `max_flow` is O(V^2 E) in general, O(E sqrt(V)) on unit
/// networks (bipartite matching). Memory is two arcs per `add`.
pub struct FlowGraph<V>:
    head: [V]List<Arc>
    to: List<V>
    left: List<Int>
    cap0: List<Int>
    cut_from: ?V

    /// An empty flow network over the vertices `V`.
    ///
    /// cost: O(n) for n = `V.count`
    /// traps: never
    /// example: `let g = FlowGraph<0..3>(); g.arcs.collect() == []`
    pub fn new() -> FlowGraph<V> = todo

    /// Adds an arc `u -> v` with capacity `cap`, and its reverse `v -> u` with capacity 0, and
    /// returns the forward arc. A `cap` of `None` is an arc no cut can afford: capacity 2^60.
    /// Self-loops are allowed and never carry flow.
    ///
    /// cost: O(1) amortized; two arcs of memory
    /// traps: when `cap` is negative
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(5)); g.cap(a) == 5 and g.cap(a.rev) == 0`
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, None); g.cap(a) == 1 << 60`
    pub fn add(mut self, u: V, v: V, cap: ?Int) -> Arc = todo

    /// Adds an undirected edge between `u` and `v`: an arc `u -> v` whose reverse also has
    /// capacity `cap`. Returns the `u -> v` arc.
    ///
    /// cost: O(1) amortized; two arcs of memory
    /// traps: when `cap` is negative
    /// example: `var g = FlowGraph<0..2>(); let a = g.add_both(0, 1, 3); g.cap(a.rev) == 3 and g.max_flow(1, 0) == 3`
    pub fn add_both(mut self, u: V, v: V, cap: Int) -> Arc = todo

    /// The flow currently sent along `a`: negative on the reverse arc of a used arc, so
    /// `flow(a.rev) == -flow(a)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(5)); g.max_flow(0, 1); g.flow(a) == 5 and g.flow(a.rev) == -5`
    pub fn flow(self, a: Arc) -> Int = todo

    /// The capacity `a` has now: what `add` or the last `set_cap` gave it, 0 for the reverse arc of
    /// an `add`, `cap` for either arc of an `add_both`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(4)); g.cap(a) == 4`
    pub fn cap(self, a: Arc) -> Int = todo

    /// The vertex `a` leaves.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var g = FlowGraph<0..3>(); let a = g.add(2, 0, Some(1)); g.source(a) == 2 and g.source(a.rev) == 0`
    pub fn source(self, a: Arc) -> V = todo

    /// The vertex `a` points to.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var g = FlowGraph<0..3>(); let a = g.add(2, 0, Some(1)); g.target(a) == 0 and g.target(a.rev) == 2`
    pub fn target(self, a: Arc) -> V = todo

    /// Pushes as much flow as possible from `s` to `t`, on top of the flow already there but no more
    /// than `limit` in this call, and returns the amount added.
    ///
    /// Dinic's algorithm: BFS levels, then blocking flows by DFS scanning each vertex's arcs in
    /// `add` order. Calling it again with no change adds 0; after it runs to completion (returns
    /// less than `limit`, or `limit` is `None`), `min_cut(s)` and `cut_arcs(s)` are valid.
    ///
    /// cost: O(V² E); O(E sqrt V) when every capacity is 1
    /// ties: among maximum flows, the one Dinic finds scanning arcs in `add` order
    /// traps: when `s == t` or `limit` is negative
    /// example: `var g = FlowGraph<0..4>(); g.add(0, 1, Some(3)); g.add(0, 2, Some(2)); g.add(1, 2, Some(1)); g.add(1, 3, Some(2)); g.add(2, 3, Some(3)); g.max_flow(0, 3) == 5`
    /// example: `var g = FlowGraph<0..4>(); g.add(0, 1, Some(3)); g.add(0, 2, Some(2)); g.add(1, 2, Some(1)); g.add(1, 3, Some(2)); g.add(2, 3, Some(3)); g.max_flow(0, 3, limit: Some(4)) == 4 and g.max_flow(0, 3) == 1`
    pub fn max_flow(mut self, s: V, t: V, limit: ?Int = None) -> Int = todo

    /// The source side of a minimum `s`-`t` cut: the vertices `s` still reaches in the residual
    /// network, after `max_flow(s, t)` (or `push_relabel(s, t)`) ran to completion.
    ///
    /// cost: O(V + E)
    /// ties: the smallest source side among minimum cuts (exactly what `s` reaches)
    /// order: keyed by `V`
    /// traps: when no completed `max_flow` or `push_relabel` from `s` came since the last `add`, `set_cap` or `reset`
    /// example: `var g = FlowGraph<0..3>(); g.add(0, 1, Some(5)); g.add(1, 2, Some(1)); g.max_flow(0, 2); g.min_cut(0) == [true, true, false]`
    pub fn min_cut(self, s: V) -> [V]Bool = todo

    /// Clears all flow, so every arc is back at its current capacity with nothing sent.
    ///
    /// cost: O(E)
    /// traps: never
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(5)); g.max_flow(0, 1); g.reset(); g.flow(a) == 0 and g.max_flow(0, 1) == 5`
    pub fn reset(mut self) = todo

    /// Changes arc `a`'s capacity to `cap` (`None`: 2^60), keeping its flow; a later `max_flow`
    /// continues from the current flow. The reverse arc keeps its own capacity.
    ///
    /// cost: O(1)
    /// traps: when the flow on `a` exceeds the new capacity (lower it with `reset` first), or `cap` is negative
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(2)); g.max_flow(0, 1); g.set_cap(a, Some(5)); g.max_flow(0, 1) == 3`
    pub fn set_cap(mut self, a: Arc, cap: ?Int) = todo

    /// The capacity left on `a` in the residual network: `cap(a) - flow(a)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var g = FlowGraph<0..2>(); let a = g.add(0, 1, Some(5)); g.max_flow(0, 1, limit: Some(2)); g.residual(a) == 3 and g.residual(a.rev) == 2`
    pub fn residual(self, a: Arc) -> Int = todo

    /// Every arc `add` or `add_both` returned, lazily; never a reverse arc. O(1) to make.
    ///
    /// cost: O(E) for the whole walk
    /// empty: yields nothing before the first `add`
    /// order: the order `add` returned them
    /// traps: never
    /// example: `var g = FlowGraph<0..3>(); let a = g.add(0, 1, Some(1)); let b = g.add(1, 2, Some(1)); g.arcs.collect() == [a, b]`
    pub @get fn arcs(self) -> yields Arc = todo

    /// The arcs `add` or `add_both` returned whose source is `u`, lazily; never a reverse arc.
    ///
    /// cost: O(deg u) for the whole walk
    /// empty: yields nothing when no added arc leaves `u`
    /// order: the order `add` returned them
    /// traps: never
    /// example: `var g = FlowGraph<0..3>(); let a = g.add(0, 1, Some(1)); g.add(1, 2, Some(1)); let c = g.add(0, 2, Some(1)); g.out(0).collect() == [a, c]`
    pub fn out(self, u: V) -> yields Arc = todo

    /// The added arcs crossing the minimum cut `min_cut(s)` from the source side to the other, after
    /// `max_flow(s, t)` ran to completion. Every one is saturated and their capacities sum to the
    /// flow value.
    ///
    /// cost: O(V + E)
    /// order: the order `add` returned them
    /// traps: like `min_cut`, when the flow from `s` is missing or stale
    /// example: `var g = FlowGraph<0..3>(); g.add(0, 1, Some(5)); let b = g.add(1, 2, Some(1)); g.max_flow(0, 2); g.cut_arcs(0) == [b]`
    pub fn cut_arcs(self, s: V) -> List<Arc> = todo

    /// The current `s`-`t` flow split into paths, each with the amount it carries; the amounts sum
    /// to the net flow out of `s`. Cycles of flow are skipped. Does not change the flow.
    ///
    /// Each path walks from `s` along the first arc, in `add` order, that still carries unclaimed
    /// flow, until it reaches `t`; its amount is the least unclaimed flow on the way.
    ///
    /// cost: O(P (V + E)) for P paths
    /// empty: yields nothing when no flow goes from `s` to `t`
    /// order: discovery order, as above
    /// traps: when `s == t`
    /// example: `var g = FlowGraph<0..4>(); let a = g.add(0, 1, Some(1)); let b = g.add(1, 3, Some(1)); let c = g.add(0, 2, Some(1)); let d = g.add(2, 3, Some(1)); g.max_flow(0, 3); g.paths(0, 3).collect() == [(arcs: [a, b], amount: 1), (arcs: [c, d], amount: 1)]`
    pub fn paths(self, s: V, t: V) -> yields (arcs: List<Arc>, amount: Int) = todo

    /// The same maximum flow amount as `max_flow(s, t)` by push-relabel (highest label, gap
    /// heuristic), often several times faster than Dinic on dense or unit-capacity grids. Adds to
    /// the flow already there and returns the amount added; `min_cut(s)` works after it.
    ///
    /// cost: O(V² sqrt E)
    /// traps: when `s == t`
    /// example: `var g = FlowGraph<0..4>(); g.add(0, 1, Some(3)); g.add(0, 2, Some(2)); g.add(1, 2, Some(1)); g.add(1, 3, Some(2)); g.add(2, 3, Some(3)); g.push_relabel(0, 3) == 5`
    pub fn push_relabel(mut self, s: V, t: V) -> Int = todo

    fn push(
        mut self,
        a: Arc,
        f: Int,
        mut excess: [V]Int,
        mut active: List<List<V>>,
        height: [V]Int,
    ) = todo

    fn close(mut self, a: Arc) = todo

    fn levels(self, s: V) -> [V]?Int = todo

    /// One augmenting path in the level graph, found without recursion.
    fn augment(mut self, s: V, t: V, level: [V]?Int, mut next: [V]Int, most: Int) -> Int = todo

/// A flow network with a cost per unit on every arc, solved by successive shortest paths
/// with Johnson potentials. Arc handles and accessors are those of `FlowGraph`.
/// `add` is O(1); memory is two arcs per `add`.
pub struct CostFlow<V>:
    head: [V]List<Arc>
    to: List<V>
    left: List<Int>
    cap0: List<Int>
    price: List<Int>

    /// An empty min-cost flow network over the vertices `V`.
    ///
    /// cost: O(n) for n = `V.count`
    /// traps: never
    /// example: `let g = CostFlow<0..3>(); g.arcs.collect() == []`
    pub fn new() -> CostFlow<V> = todo

    /// Adds an arc `u -> v` with capacity `cap` and cost `cost` per unit; its reverse has capacity 0
    /// and cost `-cost`. `cap: None` is 2^60. Costs may be negative; `min_cost_flow` needs no
    /// residual cycle of negative total cost (run `cancel_cycles` first when there may be one).
    ///
    /// cost: O(1) amortized; two arcs of memory
    /// traps: when `cap` is negative
    /// example: `var g = CostFlow<0..2>(); let a = g.add(0, 1, Some(2), 7); g.cost(a) == 7 and g.cost(a.rev) == -7`
    pub fn add(mut self, u: V, v: V, cap: ?Int, cost: Int) -> Arc = todo

    /// The flow currently sent along `a`; `flow(a.rev) == -flow(a)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var g = CostFlow<0..2>(); let a = g.add(0, 1, Some(2), 1); g.min_cost_flow(0, 1); g.flow(a) == 2`
    pub fn flow(self, a: Arc) -> Int = todo

    /// The capacity `a` has now: what `add` or the last `set_cap` gave it, 0 for a reverse arc.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var g = CostFlow<0..2>(); let a = g.add(0, 1, Some(2), 1); g.cap(a) == 2 and g.cap(a.rev) == 0`
    pub fn cap(self, a: Arc) -> Int = todo

    /// The cost per unit of flow along `a`; `cost(a.rev) == -cost(a)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var g = CostFlow<0..2>(); let a = g.add(0, 1, Some(2), -3); g.cost(a) == -3`
    pub fn cost(self, a: Arc) -> Int = todo

    /// The vertex `a` leaves.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var g = CostFlow<0..3>(); let a = g.add(2, 0, Some(1), 0); g.source(a) == 2`
    pub fn source(self, a: Arc) -> V = todo

    /// The vertex `a` points to.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var g = CostFlow<0..3>(); let a = g.add(2, 0, Some(1), 0); g.target(a) == 0`
    pub fn target(self, a: Arc) -> V = todo

    /// Changes arc `a`'s capacity, keeping its flow, as [`FlowGraph.set_cap`](FlowGraph.set_cap).
    ///
    /// cost: O(1)
    /// traps: when the flow on `a` exceeds the new capacity, or `cap` is negative
    /// example: `var g = CostFlow<0..2>(); let a = g.add(0, 1, Some(1), 2); g.min_cost_flow(0, 1); g.set_cap(a, Some(3)); g.min_cost_flow(0, 1) == (flow: 2, cost: 4)`
    pub fn set_cap(mut self, a: Arc, cap: ?Int) = todo

    /// Every arc `add` returned, lazily; never a reverse arc. O(1) to make.
    ///
    /// cost: O(E) for the whole walk
    /// empty: yields nothing before the first `add`
    /// order: the order `add` returned them
    /// traps: never
    /// example: `var g = CostFlow<0..2>(); let a = g.add(0, 1, Some(1), 0); let b = g.add(1, 0, Some(1), 0); g.arcs.collect() == [a, b]`
    pub @get fn arcs(self) -> yields Arc = todo

    /// Saturates every negative-cost cycle of the residual network, so the stored flow becomes a
    /// min-cost circulation plus the `s`-`t` flows already sent, and returns the cost added (zero or
    /// negative). Negative cycles are allowed here; afterwards `min_cost_flow` is safe.
    ///
    /// cost: O(V E log V) worst case, by cost scaling
    /// traps: never
    /// example: `var g = CostFlow<0..2>(); g.add(0, 1, Some(1), -2); g.add(1, 0, Some(1), 1); g.cancel_cycles() == -1`
    /// example: `var g = CostFlow<0..2>(); g.add(0, 1, Some(1), 2); g.cancel_cycles() == 0`
    pub fn cancel_cycles(mut self) -> Int = todo

    /// Sends the most flow from `s` to `t`, up to `limit` in this call, at the least cost among
    /// flows of that amount, on top of the flow already there. Returns the amount and cost added.
    ///
    /// Successive shortest paths with Johnson potentials: each augmentation takes a cheapest
    /// residual path, so the cost added is convex in the amount (see `slope`). Costs may be
    /// negative; the first call with a negative residual arc runs Bellman-Ford once.
    ///
    /// cost: O(F E log V) for F augmenting paths, plus O(V E) once when a residual arc has a negative cost
    /// ties: among cheapest flows, the one the shortest paths find (Dijkstra breaking ties by least vertex)
    /// traps: when `s == t`, `limit` is negative, or the residual network has a negative-cost cycle
    /// example: `var g = CostFlow<0..3>(); g.add(0, 1, Some(2), 1); g.add(1, 2, Some(2), 1); g.add(0, 2, Some(1), 5); g.min_cost_flow(0, 2) == (flow: 3, cost: 9)`
    /// example: `var g = CostFlow<0..3>(); g.add(0, 1, Some(2), 1); g.add(1, 2, Some(2), 1); g.add(0, 2, Some(1), 5); g.min_cost_flow(0, 2, limit: Some(2)) == (flow: 2, cost: 4)`
    pub fn min_cost_flow(mut self, s: V, t: V, limit: ?Int = None) -> (flow: Int, cost: Int) = todo

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
    pub fn slope(mut self, s: V, t: V, limit: ?Int = None) -> List<(flow: Int, cost: Int)> = todo

    /// Potentials under which every residual arc has a nonnegative reduced cost: all zero
    /// when no residual arc is negative, else Bellman-Ford from every vertex at once.
    fn potentials(self) -> [V]Int = todo

/// A maximum-weight closure (project selection): the set `take` of vertices with the largest
/// total `profit` such that an arc `u -> v` of `before` means taking `v` forces taking `u`
/// (prerequisites point at what they unlock, the direction `peel` and `topo` read).
///
/// Profits may be negative; the empty set is a closure, so `value >= 0`. One `max_flow` on
/// V + 2 vertices and V + E arcs; `take` is the source side of its minimum cut.
///
/// cost: one `max_flow` on V + 2 vertices and V + E arcs, O(V + E) calls of `profit`
/// ties: the smallest optimal closure (contained in every other)
/// order: `take` keyed by `V`
/// traps: never
/// example: `let p = [3, -1, -5]; max_closure([[], [0], []], profit: v => p[v]) == (value: 2, take: [true, true, false])`
/// example: `max_closure([[]], profit: _ => 0) == (value: 0, take: [false])`
pub fn max_closure<V, G: Next<V>>(before: G, profit: V -> Int) -> (value: Int, take: [V]Bool) = todo

/// A maximum matching between `L` and `R`, where `adj[l]` lists the partners `l` may take: its
/// `size`, and the partner of each vertex on either side (`None` when unmatched).
///
/// Hopcroft-Karp without recursion, scanning `L` in increasing order and each `adj[l]` in list
/// order; repeated entries are harmless.
///
/// cost: O(E sqrt V) time, O(V + E) memory
/// empty: `(size: 0, ...)` with every entry `None` when `L` or every list is empty
/// ties: the maximum matching Hopcroft-Karp finds in the scan order above
/// traps: never
/// example: `let adj: [0..2]List<0..2> = [[0, 1], [0]]; max_matching(adj) == (size: 2, mate: [Some(1), Some(0)], back: [Some(1), Some(0)])`
pub fn max_matching<L, R>(adj: [L]List<R>) -> (size: Int, mate: [L]?R, back: [R]?L) = todo

/// A minimum vertex cover of the bipartite graph `adj`: every edge has a marked end, and exactly
/// `max_matching(adj).size` vertices are marked (König). The unmarked vertices are a maximum
/// independent set.
///
/// With `Z` the vertices reachable from free left vertices by alternating paths (unmatched edge
/// left to right, matched edge right to left), the cover is the left vertices outside `Z` and the
/// right vertices inside it; this cover does not depend on which maximum matching is used.
///
/// cost: O(E sqrt V)
/// empty: nothing marked when there are no edges
/// ties: the König cover above
/// order: `left` keyed by `L`, `right` keyed by `R`
/// traps: never
/// example: `let adj: [0..2]List<0..1> = [[0], [0]]; min_vertex_cover(adj) == (left: [false, false], right: [true])`
pub fn min_vertex_cover<L, R>(adj: [L]List<R>) -> (left: [L]Bool, right: [R]Bool) = todo

/// Breadth-first layers of the left side by alternating paths: free vertices are layer 0,
/// and a matched vertex is one deeper than a left vertex adjacent to its partner.
fn alternating_layers<L, R>(adj: [L]List<R>, mate: [L]?R, back: [R]?L) -> [L]?(0..2_147_483_648) = todo

/// The maximum-weight assignment: every `l` in `L` gets a distinct `mate[l]` in `R`, maximizing
/// `total`, the sum of `weight(l, mate[l])`. Needs `L.count <= R.count`; with fewer left vertices
/// some of `R` stay unused.
///
/// cost: O(L² R) time, O(L R) calls of `weight`, by the Hungarian algorithm
/// ties: the assignment the Hungarian algorithm finds adding rows in increasing order and scanning columns in increasing order
/// order: `mate` keyed by `L`
/// traps: when `L.count > R.count`
/// example: `let w = [[1, 5], [4, 3]]; max_weight_matching<0..2, 0..2>(weight: (l, r) => w[l][r]) == (total: 9, mate: [1, 0])`
/// example: `max_weight_matching<0..1, 0..2>(weight: (_, r) => r) == (total: 1, mate: [1])`
pub fn max_weight_matching<L, R>(weight: (L, R) -> Int) -> (total: Int, mate: [L]R) = todo

/// The minimum-weight assignment: every `l` in `L` gets a distinct `mate[l]` in `R`, minimizing
/// `total`, the sum of `weight(l, mate[l])`. Needs `L.count <= R.count`.
///
/// cost: O(L² R) time, O(L R) calls of `weight`, by the Hungarian algorithm
/// ties: the assignment the Hungarian algorithm finds adding rows in increasing order and scanning columns in increasing order
/// order: `mate` keyed by `L`
/// traps: when `L.count > R.count`
/// example: `let w = [[1, 5], [4, 3]]; min_weight_matching<0..2, 0..2>(weight: (l, r) => w[l][r]) == (total: 4, mate: [0, 1])`
pub fn min_weight_matching<L, R>(weight: (L, R) -> Int) -> (total: Int, mate: [L]R) = todo

/// Minimum-cost assignment of rows `L` into columns `R` with potentials `u`, `v`; row and
/// column 0 are the sentinel of the e-maxx formulation, so indices are 1-based inside.
fn hungarian<L, R>(cost: (L, R) -> Int) -> [L]R = todo

/// A maximum matching in the undirected graph `adj` (each edge listed from both ends): its
/// `size` and each vertex's partner, `None` when unmatched.
///
/// Edmonds' blossom algorithm, growing a search from each free vertex in increasing order and
/// scanning `adj[v]` in list order; that fixes which maximum matching comes back.
///
/// cost: O(V³)
/// empty: `size == 0` and every `mate` `None` when there are no edges
/// traps: never
/// example: `let adj: [0..4]List<0..4> = [[1], [0, 2], [1, 3], [2]]; general_matching(adj) == (size: 2, mate: [Some(1), Some(0), Some(3), Some(2)])`
/// example: `let adj: [0..3]List<0..3> = [[1, 2], [0, 2], [0, 1]]; general_matching(adj).size == 1`
pub fn general_matching<V>(adj: [V]List<V>) -> (size: Int, mate: [V]?V) = todo

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

    fn delta(self, k: Int) -> Int = todo

    fn update_slack(mut self, u: Int, x: Int) = todo

    fn set_slack(mut self, x: Int) = todo

    fn push(mut self, x: Int) = todo

    fn set_st(mut self, x: Int, b: Int) = todo

    fn get_pr(mut self, b: Int, xr: Int) -> Int = todo

    fn set_match(mut self, u: Int, v: Int) = todo

    fn augment(mut self, u: Int, v: Int) = todo

    fn get_lca(mut self, u: Int, v: Int) -> Int = todo

    fn add_blossom(mut self, u: Int, lca: Int, v: Int) = todo

    fn expand_blossom(mut self, b: Int) = todo

    fn on_found(mut self, k: Int) -> Bool = todo

    /// One augmentation with dual adjustments; false when no augmenting path pays.
    fn phase(mut self) -> Bool = todo

/// A maximum-weight matching in the undirected graph `edges`: no vertex in two chosen edges,
/// maximizing `total`, the sum of chosen `w`, not necessarily of largest size. Edges with
/// `w <= 0` never help and are ignored; of parallel edges the heaviest counts.
///
/// cost: O(V³) time, O(V²) memory, by Edmonds' weighted blossom algorithm with dual labels
/// ties: the matching the algorithm finds; `total` is exact
/// traps: never
/// example: `let es: [0..3](u: 0..4, v: 0..4, w: Int) = [(u: 0, v: 1, w: 2), (u: 1, v: 2, w: 3), (u: 2, v: 3, w: 2)]; max_weight_general_matching(es) == (total: 4, mate: [Some(1), Some(0), Some(3), Some(2)])`
pub fn max_weight_general_matching<V, E>(
    edges: [E](u: V, v: V, w: Int)
) -> (total: Int, mate: [V]?V) = todo

/// A flow `f` with `lo <= f[e] <= hi` on every arc that is conserved at every vertex, or `None`
/// when there is none.
///
/// Sends `lo` on every arc, then routes each vertex's excess from a super source to a super sink
/// with one `max_flow`; the result is `lo` plus that flow.
///
/// cost: one `max_flow` on V + 2 vertices and E + V arcs
/// order: keyed by `E`
/// traps: when some `lo > hi`
/// example: `let es: [0..2](u: 0..2, v: 0..2, lo: Int, hi: Int) = [(u: 0, v: 1, lo: 2, hi: 2), (u: 1, v: 0, lo: 0, hi: 5)]; circulation(es) == Some([2, 2])`
/// example: `let es: [0..1](u: 0..2, v: 0..2, lo: Int, hi: Int) = [(u: 0, v: 1, lo: 1, hi: 1)]; circulation(es) == None`
pub fn circulation<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int)) -> ?[E]Int = todo

/// The largest `s -> t` flow with `lo <= flow[e] <= hi` on every arc, conserved at every vertex
/// but `s` and `t`, or `None` when no such flow exists. `value` is the net flow out of `s`.
///
/// cost: two `max_flow`s on V + 2 vertices and E + V + 1 arcs
/// ties: among maximum flows, the one the two `max_flow`s find
/// order: `flow` keyed by `E`
/// traps: when some `lo > hi`, or `s == t`
/// example: `let es: [0..2](u: 0..3, v: 0..3, lo: Int, hi: Int) = [(u: 0, v: 1, lo: 1, hi: 3), (u: 1, v: 2, lo: 0, hi: 2)]; max_bounded_flow(es, 0, 2) == Some((value: 2, flow: [2, 2]))`
/// example: `let es: [0..2](u: 0..3, v: 0..3, lo: Int, hi: Int) = [(u: 0, v: 1, lo: 3, hi: 3), (u: 1, v: 2, lo: 0, hi: 2)]; max_bounded_flow(es, 0, 2) == None`
pub fn max_bounded_flow<V, E>(
    arcs: [E](u: V, v: V, lo: Int, hi: Int),
    s: V,
    t: V,
) -> ?(value: Int, flow: [E]Int) = todo

/// The smallest `s -> t` flow with `lo <= flow[e] <= hi` on every arc, conserved at every vertex
/// but `s` and `t`, never sending more back into `s` than leaves it (`value >= 0`), or `None`
/// when no such flow exists.
///
/// cost: as `max_bounded_flow`
/// ties: among minimum flows, the one the two `max_flow`s find
/// order: `flow` keyed by `E`
/// traps: when some `lo > hi`, or `s == t`
/// example: `let es: [0..2](u: 0..3, v: 0..3, lo: Int, hi: Int) = [(u: 0, v: 1, lo: 1, hi: 3), (u: 1, v: 2, lo: 0, hi: 2)]; min_bounded_flow(es, 0, 2) == Some((value: 1, flow: [1, 1]))`
pub fn min_bounded_flow<V, E>(
    arcs: [E](u: V, v: V, lo: Int, hi: Int),
    s: V,
    t: V,
) -> ?(value: Int, flow: [E]Int) = todo

/// Lower bounds by demands: send `lo` on every arc up front, then route each vertex's
/// excess from a super source to a super sink. With `ends`, an unbounded `t -> s` arc
/// makes the problem a circulation; `sense` then pushes more `s -> t` (1) or back (-1).
fn bounded_flow<V, E>(
    arcs: [E](u: V, v: V, lo: Int, hi: Int),
    ends: ?(V, V),
    sense: Int,
) -> ?(value: Int, flow: [E]Int) = todo

/// A Gomory-Hu tree of the undirected graph `edges`: `V.count - 1` edges on `V` such that for
/// every `s != t` the minimum `s`-`t` cut of `edges` is the smallest `w` on the tree path from
/// `s` to `t`. Vertices in different components are joined by `w = 0`.
///
/// Gusfield's algorithm: every vertex starts with parent `V.first`; for each `u` after it in
/// order, `w` is the minimum cut between `u` and its parent `p`, the tree gains `(u, p, w)`, and
/// every later vertex with parent `p` on `u`'s side (what `u` reaches in the residual) takes
/// parent `u`.
///
/// cost: `V.count - 1` max flows on one reused network, each O(V² E)
/// empty: `[]` when `V` has at most one position
/// order: one edge per vertex but `V.first`, by increasing `u`; `v` is its parent
/// traps: when some `w` is negative
/// example: `let es: [0..2](u: 0..3, v: 0..3, w: Int) = [(u: 0, v: 1, w: 2), (u: 1, v: 2, w: 3)]; gomory_hu(es) == [(u: 1, v: 0, w: 2), (u: 2, v: 1, w: 3)]`
pub fn gomory_hu<V, E>(edges: [E](u: V, v: V, w: Int)) -> List<(u: V, v: V, w: Int)> = todo

/// A minimum cut of the undirected graph `edges` over all vertex pairs: its weight `w` and one
/// side of it, the side without `V.first`. A disconnected graph has `w == 0`.
///
/// Stoer-Wagner on a dense V x V matrix. Each phase starts from `V.first` (or the merged vertex
/// holding it) and adds the most tightly connected vertex, the least on ties; the first phase
/// whose cut is minimum gives `side`.
///
/// cost: O(V³) time, O(V²) memory
/// order: `side` keyed by `V`
/// traps: when `V` has fewer than two positions, or some `w` is negative
/// example: `let es: [0..2](u: 0..3, v: 0..3, w: Int) = [(u: 0, v: 1, w: 2), (u: 1, v: 2, w: 3)]; global_min_cut(es) == (w: 2, side: [false, true, true])`
pub fn global_min_cut<V, E>(edges: [E](u: V, v: V, w: Int)) -> (w: Int, side: [V]Bool) = todo

@test
fn textbook_network() = todo

@test
fn min_cut_equals_max_flow_on_random_networks() = todo

@test
fn bipartite_matching_with_undirected_and_unit_arcs() = todo

@test
fn hungarian_matches_brute_force_on_random_matrices() = todo

@test
fn push_relabel_agrees_with_dinic_and_leaves_a_valid_flow() = todo

@test
fn flow_over_a_product_domain_matches_flat_numbering() = todo

@test
fn an_arc_without_a_capacity_is_never_cut() = todo

@test
fn reset_restores_initial_capacities() = todo

@test
fn a_limit_stops_the_flow_and_arcs_know_their_ends() = todo

@test
fn min_cost_flow_agrees_with_hungarian_and_its_slope_is_convex() = todo

@test
fn min_cost_flow_respects_the_limit_and_negative_costs() = todo

@test
fn max_closure_matches_brute_force() = todo

@test
fn max_matching_matches_brute_force_and_the_cover_proves_it() = todo

@test
fn rectangular_assignment_matches_brute_force_both_ways() = todo

@test
fn general_matching_matches_brute_force() = todo

@test
fn weighted_general_matching_matches_brute_force() = todo

fn all_bounded_flows<V, E>(
    arcs: [E](u: V, v: V, lo: Int, hi: Int),
    s: ?V,
    t: ?V,
) -> List<(Int, List<Int>)> = todo

@test
fn bounded_flows_match_brute_force() = todo

@test
fn gomory_hu_tree_gives_every_pairwise_min_cut() = todo

@test
fn global_min_cut_matches_brute_force() = todo


/// O(E sqrt V) for the first `augment_all`, O(V + E) per later `add` + `augment`.
/// A bipartite matching that grows with the graph: edges arrive online and each
/// `augment(l)` tries one augmenting path from `l` (Kuhn), so a contest that adds
/// vertices or edges between queries keeps a maximum matching without re-solving.
pub struct Matching<L, R>:
    adj: [L]List<R>
    mate: [L]?R
    back: [R]?L
    size: Int

    /// An online bipartite matching between `L` and `R` with no edges and nothing matched.
    ///
    /// cost: O(L + R)
    /// traps: never
    /// example: `let m = Matching<0..2, 0..2>(); m.len == 0 and m.mate(0) == None`
    pub fn new() -> Matching<L, R> = todo

    /// Allows `l` to match `r`. Does not augment; a repeated edge is harmless.
    ///
    /// cost: O(1) amortized
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(0, 1); m.len == 0 and m.augment(0) and m.mate(0) == Some(1)`
    pub fn add(mut self, l: L, r: R) = todo

    /// Looks for one augmenting path from the unmatched `from` (Kuhn's DFS, trying edges in `add`
    /// order); `true` when it grew the matching. `false` also when `from` is already matched.
    ///
    /// cost: O(V + E)
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(0, 0); m.add(0, 1); m.add(1, 0); m.augment(0); m.augment(1) and m.mate(0) == Some(1) and m.mate(1) == Some(0)`
    pub fn augment(mut self, from: L) -> Bool = todo

    /// Grows the current matching to a maximum one by Hopcroft-Karp and returns the number of pairs
    /// added.
    ///
    /// cost: O(E sqrt V)
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(0, 0); m.add(1, 0); m.add(1, 1); m.augment_all() == 2 and m.len == 2`
    pub fn augment_all(mut self) -> Int = todo

    /// Unmatches `l` and its mate `r`, then tries to rematch both: `augment(l)` first, then one
    /// augmenting path ending at `r` if it is still free. Returns the new size; the matching stays
    /// maximum when it was before. No edge is removed.
    ///
    /// cost: O(V + E)
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(0, 0); m.add(1, 0); m.augment(0); m.remove(0) == 1 and m.mate(0) == Some(0)`
    pub fn remove(mut self, l: L) -> Int = todo

    /// `l`'s partner, `None` when unmatched.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(1, 0); m.augment(1); m.mate(1) == Some(0) and m.mate(0) == None`
    pub fn mate(self, l: L) -> ?R = todo
    /// `r`'s partner, `None` when unmatched.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(1, 0); m.augment(1); m.back(0) == Some(1) and m.back(1) == None`
    pub fn back(self, r: R) -> ?L = todo
    /// The number of matched pairs.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var m = Matching<0..2, 0..2>(); m.add(1, 0); m.augment(1); m.len == 1`
    pub @get fn len(self) -> Int = todo
    /// The König vertex cover of the current matching, built as in
    /// [`min_vertex_cover`](min_vertex_cover); a minimum cover when the matching is maximum.
    ///
    /// cost: O(V + E)
    /// order: `left` keyed by `L`, `right` keyed by `R`
    /// traps: never
    /// example: `var m = Matching<0..2, 0..1>(); m.add(0, 0); m.add(1, 0); m.augment_all(); m.cover() == (left: [false, false], right: [true])`
    pub fn cover(self) -> (left: [L]Bool, right: [R]Bool) = todo

/// The maximum (or, with `min: true`, minimum) total-weight matching among the listed edges, not
/// necessarily perfect: sparse weights through min-cost flow, where `max_weight_matching` needs
/// every pair's weight. `size:` forces exactly that many pairs, `None` when impossible.
///
/// Without `size`, augmentation stops at the first path that would not strictly improve the
/// total, so of equal totals the one with fewer pairs comes back. `used[e]` marks the chosen
/// edges; of parallel edges at most one is used.
///
/// cost: O(F E log V), F = matching size
/// ties: the matching successive shortest paths finds, fewer pairs first
/// order: `mate` keyed by `L`, `used` keyed by `E`
/// traps: when `size` is negative
/// example: `let es: [0..3](l: 0..2, r: 0..2, w: Int) = [(l: 0, r: 0, w: 5), (l: 0, r: 1, w: 3), (l: 1, r: 0, w: 4)]; best_matching(es) == Some((total: 7, mate: [Some(1), Some(0)], used: [false, true, true]))`
/// example: `let es: [0..3](l: 0..2, r: 0..2, w: Int) = [(l: 0, r: 0, w: 5), (l: 0, r: 1, w: 3), (l: 1, r: 0, w: 4)]; best_matching(es, size: Some(1)) == Some((total: 5, mate: [Some(0), None], used: [true, false, false]))`
/// example: `let es: [0..3](l: 0..2, r: 0..2, w: Int) = [(l: 0, r: 0, w: 5), (l: 0, r: 1, w: 3), (l: 1, r: 0, w: 4)]; best_matching(es, size: Some(3)) == None and best_matching(es, min: true)!.total == 0`
pub fn best_matching<L, R, E>(edges: [E](l: L, r: R, w: Int), size: ?Int = None, min: Bool = false) -> ?(total: Int, mate: [L]?R, used: [E]Bool) = todo

/// Gale-Shapley stable matching with proposals from `L`: `pref_l[l]` lists the `R`s `l` accepts,
/// best first, and likewise `pref_r`. A pair can match only when each lists the other. The result
/// is the stable matching best for every `l`, which is unique.
///
/// cost: O(L R)
/// empty: `[]` when `L` is empty; `None` for every `l` whose list is empty
/// traps: when a preference list names the same vertex twice
/// example: `let pl: [0..2]List<0..2> = [[0, 1], [0, 1]]; let pr: [0..2]List<0..2> = [[1, 0], [1, 0]]; stable_matching(pl, pr) == [Some(1), Some(0)]`
/// example: `let pl: [0..1]List<0..1> = [[0]]; let pr: [0..1]List<0..1> = [[]]; stable_matching(pl, pr) == [None]`
pub fn stable_matching<L, R>(pref_l: [L]List<R>, pref_r: [R]List<L>) -> [L]?R = todo

/// Colours the edges of a bipartite multigraph so no two edges at a vertex share a colour, with
/// exactly max-degree colours `0..colors` (König's edge colouring theorem).
///
/// Edges are coloured in `E` order. For edge `(l, r)`, `a` is the least colour free at `l` and
/// `b` the least free at `r`; when they differ, the path from `r` alternating colours `a` and `b`
/// has them swapped, which frees `a` at `r`; then the edge takes `a`.
///
/// cost: O(E V)
/// order: `color` keyed by `E`
/// traps: never
/// example: `let es: [0..3](l: 0..2, r: 0..2) = [(l: 0, r: 0), (l: 0, r: 1), (l: 1, r: 0)]; edge_coloring(es) == (colors: 2, color: [1, 0, 0])`
pub fn edge_coloring<L, R, E>(edges: [E](l: L, r: R)) -> (colors: Int, color: [E]Int) = todo

/// A largest set of elements independent in both matroids; with `weight`, the one of largest
/// total weight among the largest.
///
/// Augments along shortest paths of the exchange graph (weighted: shortest by weight, then by
/// length), breadth-first from the sources in increasing element order, so each augmentation
/// takes the least element on ties.
///
/// cost: O(r E (r + oracle)) for answer size r, oracle the cost of one `can_add` or `can_swap`
/// order: keyed by `E`
/// traps: never
/// example: `let es: [0..3](u: 0..3, v: 0..3) = [(u: 0, v: 1), (u: 1, v: 2), (u: 0, v: 2)]; let col: [0..3]0..2 = [0, 0, 1]; let lim: [0..2]Int = [1, 1]; matroid_intersection(GraphicMatroid(es), ColorMatroid(col, lim)) == [true, false, true]`
pub fn matroid_intersection<E, A: Matroid<E>, B: Matroid<E> >(a: A, b: B, weight: ?(E -> Int) = None) -> [E]Bool = todo

/// An independence oracle for `matroid_intersection`. `rebuild(take)` sets the current
/// independent set; `can_add(e)` asks whether `take + e` stays independent, and
/// `can_swap(out, e)` whether `take - out + e` does. Graphic and colour (partition)
/// matroids ship as `GraphicMatroid` and `ColorMatroid`.
pub trait Matroid<E>:
    fn rebuild(mut self, take: [E]Bool)
    fn can_add(self, e: E) -> Bool
    fn can_swap(self, out: E, e: E) -> Bool

/// O(E α) per rebuild. Independent sets are forests of the edges' endpoints.
pub struct GraphicMatroid<V, E>:
    edges: [E](u: V, v: V)
    /// The graphic matroid of `edges`: a set of edges is independent when it is a forest on `V`.
    ///
    /// cost: O(1); each `rebuild` is O(E alpha(V))
    /// traps: never
    /// example: `let es: [0..3](u: 0..3, v: 0..3) = [(u: 0, v: 1), (u: 1, v: 2), (u: 0, v: 2)]; var m = GraphicMatroid(es); m.rebuild([true, true, false]); not m.can_add(2) and m.can_swap(0, 2)`
    pub fn new(edges: [E](u: V, v: V)) -> GraphicMatroid<V, E> = todo

/// O(E) per rebuild. Independent sets take at most `limit[c]` elements of each colour.
pub struct ColorMatroid<E, C>:
    color: [E]C
    limit: [C]Int
    /// The partition matroid of `color`: a set is independent when it holds at most `limit[c]`
    /// elements of each colour `c`.
    ///
    /// cost: O(1); each `rebuild` is O(E)
    /// empty: with no elements only the empty set exists and every query is `false`
    /// traps: when some `limit[c]` is negative
    /// example: `let col: [0..3]0..2 = [0, 0, 1]; let lim: [0..2]Int = [1, 1]; var m = ColorMatroid(col, lim); m.rebuild([true, false, false]); not m.can_add(1) and m.can_add(2)`
    pub fn new(color: [E]C, limit: [C]Int) -> ColorMatroid<E, C> = todo

/// The minimum-cost circulation of arcs with lower and upper bounds and costs, negative cycles
/// included (cost scaling); `None` when no circulation meets the bounds.
///
/// cost: O(V² E) worst case, like `max_flow`
/// ties: among cheapest circulations, the one cost scaling finds
/// order: `flow` keyed by `E`
/// traps: when some `lo > hi`
/// example: `let es: [0..2](u: 0..2, v: 0..2, lo: Int, hi: Int, cost: Int) = [(u: 0, v: 1, lo: 0, hi: 2, cost: -3), (u: 1, v: 0, lo: 0, hi: 5, cost: 1)]; min_cost_circulation(es) == Some((cost: -4, flow: [2, 2]))`
/// example: `let es: [0..1](u: 0..2, v: 0..2, lo: Int, hi: Int, cost: Int) = [(u: 0, v: 1, lo: 1, hi: 1, cost: 0)]; min_cost_circulation(es) == None`
pub fn min_cost_circulation<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int, cost: Int)) -> ?(cost: Int, flow: [E]Int) = todo
