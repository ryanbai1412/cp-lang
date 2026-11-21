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
    /// The paired arc in the opposite direction. This is what `e ^ 1` means in C++.
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

    /// An empty network. O(V).
    pub fn new() -> FlowGraph<V> = todo

    /// Add an arc `u -> v` with capacity `cap` (and its reverse with capacity 0). O(1).
    /// A `cap` of `none` is an arc no cut can afford: its capacity is 2^60.
    pub fn add(mut self, u: V, v: V, cap: ?Int) -> Arc = todo

    /// Add an undirected edge: `cap` in both directions. O(1).
    pub fn add_both(mut self, u: V, v: V, cap: Int) -> Arc = todo

    /// The flow currently sent along `a` (negative on the reverse arc of a used arc).
    pub fn flow(self, a: Arc) -> Int = todo

    /// The capacity `a` was added with.
    pub fn cap(self, a: Arc) -> Int = todo

    /// The vertex `a` leaves.
    pub fn source(self, a: Arc) -> V = todo

    /// The vertex `a` points to.
    pub fn target(self, a: Arc) -> V = todo

    /// Push as much as possible from `s` to `t`, on top of the flow already there, but
    /// no more than `limit`. Returns the amount added. Dinic: O(V^2 E); O(E sqrt(V)) when
    /// every capacity is 1.
    pub fn max_flow(mut self, s: V, t: V, limit: ?Int = none) -> Int = todo

    /// The source side of a minimum cut, after `max_flow(s, t)` ran to completion: the
    /// vertices `s` still reaches in the residual network. Traps if no such `max_flow`
    /// came since the last `add` or `reset`. O(V + E).
    pub fn min_cut(self, s: V) -> [V]Bool = todo

    /// Clear all flow, restoring every arc to its initial capacity. O(E).
    pub fn reset(mut self) = todo

    /// O(1). Change arc `a`'s capacity to `cap` (`none`: never cut), keeping its flow
    /// when it still fits; a later `max_flow` re-solves from the current flow. Traps when
    /// the flow on `a` exceeds the new capacity (lower it with `reset` first).
    pub fn set_cap(mut self, a: Arc, cap: ?Int) = todo

    /// O(1). The capacity left on `a` in the residual network: `cap(a) - flow(a)`.
    pub fn residual(self, a: Arc) -> Int = todo

    /// O(1) to make, O(E) to walk. Every arc added, in the order `add` returned them (never a reverse arc).
    pub @get fn arcs(self) -> yields Arc = todo

    /// O(deg u). The arcs added out of `u`, in `add` order.
    pub fn out(self, u: V) -> yields Arc = todo

    /// O(V + E). The arcs from `side` to its complement after `max_flow(s, t)`: the
    /// edges of the min cut `min_cut(s)` returns. Traps like `min_cut` on a stale flow.
    pub fn cut_arcs(self, s: V) -> List<Arc> = todo

    /// O(P (V + E)) for P paths. The current s-t flow as paths, each with the amount it
    /// carries; their amounts sum to the flow out of `s`. Cycles of flow are skipped.
    pub fn paths(self, s: V, t: V) -> yields (arcs: List<Arc>, amount: Int) = todo

    /// The same maximum flow as `max_flow(s, t)` by push-relabel (highest label, gap
    /// heuristic): O(V^2 sqrt(E)), and often several times faster than Dinic on dense
    /// or unit-capacity grids. Returns the amount added; `min_cut(s)` works after it.
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

    /// An empty network. O(V).
    pub fn new() -> CostFlow<V> = todo

    /// Add an arc `u -> v` with capacity `cap` and cost `cost` per unit; its reverse has
    /// capacity 0 and cost `-cost`. `cap: none` is 2^60. Costs may be negative as long
    /// as no cycle of positive capacity has negative total cost. O(1).
    pub fn add(mut self, u: V, v: V, cap: ?Int, cost: Int) -> Arc = todo

    /// The flow currently sent along `a`.
    pub fn flow(self, a: Arc) -> Int = todo

    /// The capacity `a` was added with.
    pub fn cap(self, a: Arc) -> Int = todo

    /// The cost per unit of flow along `a`.
    pub fn cost(self, a: Arc) -> Int = todo

    /// The vertex `a` leaves.
    pub fn source(self, a: Arc) -> V = todo

    /// The vertex `a` points to.
    pub fn target(self, a: Arc) -> V = todo

    /// O(1). Change arc `a`'s capacity, as `FlowGraph.set_cap`.
    pub fn set_cap(mut self, a: Arc, cap: ?Int) = todo

    /// O(1) to make, O(E) to walk. Every arc added, in `add` order.
    pub @get fn arcs(self) -> yields Arc = todo

    /// O(V E log V) worst case by cost scaling. Saturate every negative-cost cycle of the
    /// residual network as well, so the stored flow is a min-cost circulation plus the s-t
    /// flows already sent; returns the cost added. Negative cycles are allowed here.
    pub fn cancel_cycles(mut self) -> Int = todo

    /// Send the most flow from `s` to `t`, up to `limit`, at the least cost among flows of
    /// that amount, on top of the flow already there. Returns the amount and cost added.
    /// O(F E log V) for F augmenting paths, plus O(V E) once when a residual arc has a
    /// negative cost (the first call with negative costs, and every later call).
    pub fn min_cost_flow(mut self, s: V, t: V, limit: ?Int = none) -> (flow: Int, cost: Int) = todo

    /// The breakpoints of the cheapest cost of sending each amount from `s` to `t`, up to
    /// `limit`: `(0, 0)` first, then one `(flow, cost)` per change of slope; between
    /// breakpoints the cost is linear and the slopes never decrease. Same cost as
    /// `min_cost_flow`.
    pub fn slope(mut self, s: V, t: V, limit: ?Int = none) -> List<(flow: Int, cost: Int)> = todo

    /// Potentials under which every residual arc has a nonnegative reduced cost: all zero
    /// when no residual arc is negative, else Bellman-Ford from every vertex at once.
    fn potentials(self) -> [V]Int = todo

/// A maximum-weight closure: the set `take` of vertices with the largest total `profit`
/// such that taking `needs[e].u` forces taking `needs[e].v`. Profits may be negative; the
/// empty set is a closure, so `value >= 0`. One `max_flow` on V + 2 vertices and
/// V + E arcs (project selection).
pub fn max_closure<V, E>(profit: [V]Int, needs: [E](u: V, v: V)) -> (value: Int, take: [V]Bool) = todo

/// A maximum matching between `L` and `R`, where `adj[l]` lists the partners `l` may take:
/// its `size`, and the partner of each vertex on either side (`none` when unmatched).
/// Hopcroft-Karp, O(E sqrt(V)) time, no recursion.
pub fn max_matching<L, R>(adj: [L]List<R>) -> (size: Int, mate: [L]?R, back: [R]?L) = todo

/// A minimum vertex cover of the bipartite graph `adj`: every edge has a marked end, and
/// exactly `max_matching(adj).size` vertices are marked (Konig). The unmarked vertices are
/// a maximum independent set. O(E sqrt(V)).
pub fn min_vertex_cover<L, R>(adj: [L]List<R>) -> (left: [L]Bool, right: [R]Bool) = todo

/// Breadth-first layers of the left side by alternating paths: free vertices are layer 0,
/// and a matched vertex is one deeper than a left vertex adjacent to its partner.
fn alternating_layers<L, R>(adj: [L]List<R>, mate: [L]?R, back: [R]?L) -> [L]?(0..2_147_483_648) = todo

/// The maximum-weight assignment: every `l` in `L` gets a distinct `mate[l]` in `R`
/// (needs `L.K <= R.K`), maximizing `total`, the sum of `weight(l, mate[l])`.
/// Hungarian algorithm, O(L^2 R) time, O(L R) calls of `weight`.
pub fn max_weight_matching<L, R>(weight: (L, R) -> Int) -> (total: Int, mate: [L]R) = todo

/// The minimum-weight assignment: every `l` in `L` gets a distinct `mate[l]` in `R`
/// (needs `L.K <= R.K`), minimizing `total`, the sum of `weight(l, mate[l])`.
/// Hungarian algorithm, O(L^2 R) time, O(L R) calls of `weight`.
pub fn min_weight_matching<L, R>(weight: (L, R) -> Int) -> (total: Int, mate: [L]R) = todo

/// Minimum-cost assignment of rows `L` into columns `R` with potentials `u`, `v`; row and
/// column 0 are the sentinel of the e-maxx formulation, so indices are 1-based inside.
fn hungarian<L, R>(cost: (L, R) -> Int) -> [L]R = todo

/// A maximum matching in the undirected graph `adj` (each edge listed from both ends):
/// its `size` and each vertex's partner. Edmonds' blossom algorithm, O(V^3).
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

/// A maximum-weight matching in the undirected graph `edges`: no vertex in two chosen
/// edges, maximizing `total`, the sum of chosen `w`. Edges with `w <= 0` never help and
/// are ignored; of parallel edges the heaviest counts. Edmonds' weighted blossom
/// algorithm with dual labels, O(V^3) time, O(V^2) memory.
pub fn max_weight_general_matching<V, E>(
    edges: [E](u: V, v: V, w: Int)
) -> (total: Int, mate: [V]?V) = todo

/// A flow `f` on every arc with `lo <= f[e] <= hi` that is conserved at every vertex, or
/// `none` when there is none. One `max_flow` on V + 2 vertices and E + V arcs.
pub fn circulation<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int)) -> ?[E]Int = todo

/// The largest `s -> t` flow with `lo <= flow[e] <= hi` on every arc, conserved at every
/// vertex but `s` and `t`, or `none` when no such flow exists. `value` is the net flow out
/// of `s`. Two `max_flow`s on V + 2 vertices and E + V + 1 arcs.
pub fn max_bounded_flow<V, E>(
    arcs: [E](u: V, v: V, lo: Int, hi: Int),
    s: V,
    t: V,
) -> ?(value: Int, flow: [E]Int) = todo

/// The smallest `s -> t` flow with `lo <= flow[e] <= hi` on every arc, conserved at every
/// vertex but `s` and `t`, never sending more back into `s` than leaves it (`value >= 0`),
/// or `none` when no such flow exists. Same cost as
/// `max_bounded_flow`.
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

/// A Gomory-Hu tree of the undirected graph `edges`: `V.K - 1` edges on `V` such that for
/// every `s != t` the minimum `s-t` cut of `edges` is the smallest `w` on the tree path
/// from `s` to `t`. Vertices in different components are joined by `w = 0`. Gusfield's
/// algorithm: `V.K - 1` max flows on one reused network, each O(V^2 E).
pub fn gomory_hu<V, E>(edges: [E](u: V, v: V, w: Int)) -> List<(u: V, v: V, w: Int)> = todo

/// A minimum cut of the undirected graph `edges` over all vertex pairs: its weight `w`
/// and one side of it. Stoer-Wagner on a dense V x V matrix, O(V^3) time, O(V^2) memory.
/// Needs at least two vertices; weights are non-negative.
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

    /// O(L + R). No edges, nothing matched.
    pub fn new() -> Matching<L, R> = todo

    /// O(1). Allow `l` to match `r`. Does not augment.
    pub fn add(mut self, l: L, r: R) = todo

    /// O(V + E). Look for one augmenting path from the unmatched `from`; true when it
    /// grew the matching. False also when `from` is already matched.
    pub fn augment(mut self, from: L) -> Bool = todo

    /// O(E sqrt V) Hopcroft-Karp from the current matching. Returns the size added.
    pub fn augment_all(mut self) -> Int = todo

    /// O(V + E). Unmatch `l` (and its mate), then try to rematch both. Returns the new
    /// size; the matching stays maximum when it was before.
    pub fn remove(mut self, l: L) -> Int = todo

    /// O(1). `l`'s partner, `none` when unmatched.
    pub fn mate(self, l: L) -> ?R = todo
    /// O(1). `r`'s partner, `none` when unmatched.
    pub fn back(self, r: R) -> ?L = todo
    /// O(1). The number of matched pairs.
    pub @get fn len(self) -> Int = todo
    /// O(V + E). The König vertex cover of the current matching (minimum when the
    /// matching is maximum).
    pub fn cover(self) -> (left: [L]Bool, right: [R]Bool) = todo

/// O(F E log V), F = matching size. The maximum (or, with `min`, minimum) total weight
/// matching among the listed edges, not necessarily perfect: sparse weights through
/// min-cost flow, where `max_weight_matching` needs every pair's weight.
/// `size:` forces exactly that many pairs (`none` when impossible).
pub fn best_matching<L, R, E>(edges: [E](l: L, r: R, w: Int), size: ?Int = none, min: Bool = false) -> ?(total: Int, mate: [L]?R, used: [E]Bool) = todo

/// O(L R). Gale-Shapley stable matching, proposals from `L`: `pref_l[l]` lists the `R`s
/// `l` accepts, best first, and likewise `pref_r`. Optimal for every `l` among stable
/// matchings.
pub fn stable_matching<L, R>(pref_l: [L]List<R>, pref_r: [R]List<L>) -> [L]?R = todo

/// O(E V). Colour the edges of a bipartite multigraph so no two edges at a vertex share a
/// colour, with exactly max-degree colours (König's edge colouring theorem).
pub fn edge_coloring<L, R, E>(edges: [E](l: L, r: R)) -> (colors: Int, color: [E]Int) = todo

/// O(E^2 V) is not reached; O(r E (r + oracle)) for answer size r. A largest set of
/// elements independent in both matroids; with `weight`, the largest-total among the
/// largest (weighted intersection by shortest augmenting paths).
pub fn matroid_intersection<E, A: Matroid<E>, B: Matroid<E> >(a: A, b: B, weight: ?(E -> Int) = none) -> [E]Bool = todo

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
    /// O(1). The matroid of forests over `edges`.
    pub fn new(edges: [E](u: V, v: V)) -> GraphicMatroid<V, E> = todo

/// O(E) per rebuild. Independent sets take at most `limit[c]` elements of each colour.
pub struct ColorMatroid<E, C>:
    color: [E]C
    limit: [C]Int
    /// O(1). The partition matroid of `color` with per-colour `limit`.
    pub fn new(color: [E]C, limit: [C]Int) -> ColorMatroid<E, C> = todo

/// O(V^2 E) worst case, like `max_flow`. The minimum cost circulation of arcs with lower
/// and upper bounds and costs, negative cycles included (cost scaling); `none` when no
/// circulation meets the bounds.
pub fn min_cost_circulation<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int, cost: Int)) -> ?(cost: Int, flow: [E]Int) = todo
