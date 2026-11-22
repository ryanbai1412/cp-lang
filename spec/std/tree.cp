//! Rooted trees: parents, subtree ranges, LCA, and path folds over one range structure.
//!
//! Each `Rooted` tree value `t` owns a fresh domain, `t.Pre`: the positions of an
//! Euler order in which every subtree and every heavy chain is one contiguous range.
//! `t.pos(v)` is a `t.Pre`, not a `V` (and never mixes with another tree `t2.Pre`), so
//! indexing a per-position structure with a vertex or another tree's position does not type-check.
//!
//! `Rooted` is the one tree abstraction. A value per vertex lives in any
//! structure indexed by `t.Pre` (a `SegTree`, with or without `act:`, a `Fenwick`), and
//! `fold_path` / `apply_path` walk a path over it, so there is no second
//! "tree with values" type and no second copy of the data.
use std.algebra.{Reversible}
use std.graph.{Neighbours, Target, Graph, Visit}
use std.dp.{Step}
use std.algebra.{Monoid, MaxSub, Action}
use std.segtree.{SegTree}
use std.algebra.{add}
use std.seq.{dedup}
use std.rand.{Rng}

/// A tree rooted at `root`, owning its own fresh heavy-first preorder domain `t.Pre`.
/// Built in O(n) without recursion; memory is seven `[V]` arrays, one `[Pre]` array and child
/// lists.
pub struct Rooted<V, Pre>:
    root: V
    /// The parent; `none` at the root.
    par: [V]?V
    dep: [V]0..=V.count
    /// The total edge length from the root; equal to the depth when the edges have no lengths.
    far: [V]Int
    /// The number of vertices in the subtree.
    sz: [V]1..=V.count
    /// Breadth-first order: the children of `v` are `bfs[k0[v]..k1[v]]`, so no vertex owns a list.
    bfs: List<V>
    k0: [V]0..=V.count
    k1: [V]0..=V.count
    /// The child with the largest subtree; `none` at a leaf.
    heavy: [V]?V
    /// The highest vertex of the heavy chain through `v`.
    top: [V]V
    tin: [V]Pre
    /// The inverse of `tin` / `pos`.
    at: [Pre]V

    /// O(n). Root the tree given by `next` at `root`: `Rooted(g, root: r)` for a `Graph`,
    /// `Rooted(lists, root: r)` for a table of neighbour lists `[V]List<V>`, and
    /// `Rooted(root: r, next: u => ...)` for anything else. `next` may list every neighbour
    /// (the parent is skipped) or only the children, as vertices or as arcs. The tree is the
    /// breadth-first spanning tree, so on a graph with extra edges `depth` is the distance
    /// from the root. Stops if some vertex is not reachable from the root.
    /// Edge lengths come from the arcs, as in every graph algorithm: arcs that are `Weighted`
    /// (a `w` field, or `g.weighted(f)`) give `dist`, `w` and `diameter` their lengths; others
    /// count 1 per edge.
    pub fn new<N>(next: N, root: V) -> Rooted<V, _> = todo

    /// O(n). Build a rooted tree from parents, keyed by vertex: `par: [V]?V` with `none` at the
    /// root, or the usual input shape "parents of 2..n" read as `read<[V.drop(1)]V>()`, whose one
    /// missing key is the root.
    pub fn from_parents<P: Keyed>(par: P) -> Rooted<V, _> = todo

    /// O(n). One tree per root: `next` must reach every vertex from exactly one of `from`. The
    /// trees sit one after another in `t.Pre`, in the order of `from`; `t.root` is the first root.
    /// Paths and lowest common ancestors exist only within one tree (`root_of` tells).
    pub fn forest<N>(next: N, from: List<V>) -> Rooted<V, _> = todo

    fn build<N, A>(next: N, roots: List<V>, len: A -> Int) = todo

    fn finish(
        roots: List<V>,
        par: [V]?V,
        dep: [V]0..=V.count,
        far: [V]Int,
        bfs: List<V>,
        k0: [V]0..=V.count,
        k1: [V]0..=V.count,
    ) = todo

    /// O(1). The position of `v` in this tree's value-owned preorder domain `t.Pre`. Inverse of
    /// `at`.
    pub fn pos(self, v: V) -> Pre = todo

    /// O(1). The parent of `v`; `none` at the root.
    pub fn parent(self, v: V) -> ?V = todo

    /// O(1). The arc `v` was entered by, with every field `next` gave it (`t.up_arc(v)!.cost`
    /// on a tree read with a `cost` field); `none` at a root. Edge data stays on the arcs, so no
    /// `[V]` side array is built from the edge list.
    pub fn up_arc<A>(self, v: V) -> ?A = todo

    /// O(n). Reorder a vertex-indexed array `[V]T` into preorder `[t.Pre]T` (`out[t.pos(v)] ==
    /// a[v]`).
    pub fn reorder<T>(self, a: [V]T) -> [Pre]T = todo

    /// O(1) to start, O(1) per child, no allocation (children are stored contiguously): the
    /// children of `v`, in the order `next` listed them.
    pub fn children(self, v: V) -> yields V = todo

    /// O(children) calls of `f`. For each child `c` of `v`, in child order, the fold with `m` of
    /// `f` over `v`'s other children: the step of a rerooting DP, with no division or inverse
    /// needed. The fold keeps child order (earlier siblings, then later ones), so `m` need not
    /// commute. O(k) calls of
    /// `f` and of `m` for `k` children.
    ///
    ///     for (c, rest) in t.excl_fold(v, mul, c => down[c] + 1):
    ///         up[c] = (up[v] * rest + 1) % m
    pub fn excl_fold<T, M>(self, v: V, op: M, f: V -> T) -> yields (c: V, rest: T) = todo

    /// O(1). The number of edges between `v` and the root.
    pub fn depth(self, v: V) -> Int = todo

    /// O(1). The number of vertices in the subtree of `v`.
    pub fn size(self, v: V) -> Int = todo

    /// O(children). The children of `v` other than `heavy[v]`, in child order. Every vertex has
    /// O(log n) light edges above it, which is what small-to-large merging pays for.
    pub fn light(self, v: V) -> yields V = todo

    /// O(log n). The root of the tree that holds `v` (always `t.root` unless built with `forest`).
    pub fn root_of(self, v: V) -> V = todo

    /// O(size(v)). The vertices of `v`'s subtree in preorder, `v` first: `t.at[p] for p in
    /// t.sub(v)`.
    pub fn subtree(self, v: V) -> yields V = todo

    /// O(n) calls of `f`. A top-down pass: `out[v] = f(out[parent], v)`, and `f(init, root)` at a
    /// root. The state on the path from the root, such as a prefix maximum or a distance. O(n)
    /// calls of `f`.
    pub fn down<T>(self, init: T, f: (T, V) -> T) -> [V]T = todo

    /// O(n) calls. A bottom-up tree DP: `out[v] = finish(fold with m of lift(out[c], c, v) over
    /// children c, v)`.
    /// `lift` sees both ends of the edge, child first, and defaults to the child's value. Children
    /// fold in child order.
    pub fn up<T, U, M>(
        self,
        op: M,
        lift: (T, V, V) -> U = (x, c, p) => x,
        finish: (U, V) -> T,
    ) -> [V]T = todo

    /// O(n) calls. The same DP as `up`, for every root at once: `out[v]` is what `up` would give at
    /// `v` if the tree were rooted at `v`. Each vertex folds its parent's side first, then its
    /// children; the sibling folds use prefixes and suffixes, so `m` needs no inverse.
    pub fn reroot<T, U, M>(
        self,
        op: M,
        lift: (T, V, V) -> U = (x, c, p) => x,
        finish: (U, V) -> T,
    ) -> [V]T = todo

    /// O(n log n). The isomorphism class of every subtree: `of[a] == of[b]` exactly when the
    /// subtrees of `a` and `b` are the same rooted tree up to reordering children. Classes are
    /// numbered by first appearance from the leaves up.
    pub fn shapes(self) -> (domain Shape, of: [V]Shape) = todo

    /// O(1). The size of every subtree, by vertex: `t.subtree_sizes[v]` is `t.size(v)`.
    pub @get fn subtree_sizes(self) -> [V]Int = todo

    /// O(depth). A centroid: a vertex whose removal leaves no piece larger than half the tree. When
    /// there are two, this is the one nearer the root.
    pub fn centroid(self) -> V = todo

    /// O(depth). Every centroid, one or two: the one nearer the root first. Two happen only when
    /// the number of vertices is even and one edge splits the tree in halves.
    pub fn centroids(self) -> List<V> = todo

    /// O(1) to make, O(n) to walk. Iterative enter/exit Euler tour, the `Visit` events of `std.graph.dfs`: yields
    /// `Enter(v, parent)` when entering `v` and `Exit(v, parent)` when leaving its subtree.
    pub @get fn tour(self) -> yields Visit<V> = todo

    /// O(1). The positions of `v`'s subtree, `v` first.
    pub fn sub(self, v: V) -> Range<Pre> = todo

    /// O(1). The positions of `v`'s subtree without `v`: its descendants.
    pub fn below(self, v: V) -> Range<Pre> = todo

    /// O(1). Is `a` an ancestor of `b`? A vertex is its own ancestor.
    pub fn is_anc(self, a: V, b: V) -> Bool = todo

    /// O(1). The child end of the tree edge between `u` and `v`: the one below.
    pub fn lower(self, u: V, v: V) -> V = todo

    /// O(log n). The lowest common ancestor.
    pub fn lca(self, a: V, b: V) -> V = todo

    /// O(log n). The length of the path between `a` and `b`: its number of edges, or their total
    /// length in a tree built with `weighted`.
    pub fn dist(self, a: V, b: V) -> Int = todo

    /// O(log n + |path|). The simple path of vertices from `a` to `b`, inclusive. O(|path|).
    pub fn path(self, a: V, b: V) -> List<V> = todo

    /// O(n). A longest simple path in the tree: `(len, u, v)` with `self.dist(u, v) == len`.
    pub fn diameter(self) -> (len: Int, u: V, v: V) = todo

    /// O(log n). The ancestor `k` edges above `v`, or `none` if that is above the root.
    pub fn ancestor(self, v: V, k: Int) -> ?V = todo

    /// O(log n). The vertex `k` edges from `a` on the path to `b`, or `none` if the path is
    /// shorter.
    pub fn jump(self, a: V, b: V, k: Int) -> ?V = todo

    /// O(log^2 n). The highest ancestor `u` of `v` such that `pred` holds for `u`, or `none` when
    /// it does not hold for `v` itself. `pred` must be true and then false on the way up.
    pub fn jump_while(self, v: V, pred: V -> Bool) -> ?V = todo

    /// O(log n). The path from `a` to `b` as O(log n) position ranges. `rising` ranges are walked
    /// from their high end to their low end. With `edges`, the highest vertex of the path is left
    /// out: for a value per edge, stored at the edge's lower end.
    pub fn chains(
        self,
        a: V,
        b: V,
        edges: Bool = false,
    ) -> yields (range: Range<Pre>, rising: Bool) = todo

    /// O(log^2 n). The fold of the values on the path from `a` to `b`, in that order, read from
    /// `ds`: any structure over `Pre` with `fold(range)` and a `monoid` (`SegTree`).
    /// A stretch of the path that climbs is folded top-down by `ds` and then turned round
    /// with `rev()`: a value that is not its own reverse implements `Reversible`. One
    /// structure, O(log^2 n).
    pub fn fold_path<S, T>(self, a: V, b: V, ds: S, edges: Bool = false) -> T = todo

    /// O(log^2 n). Every vertex on the path from `a` to `b` takes the update `tag`, through
    /// `ds.apply`.
    pub fn apply_path<S, F>(self, a: V, b: V, mut ds: S, tag: F, edges: Bool = false) = todo

    /// O(k log k + k log n). The tree squeezed down to `keys`: the keys and the lowest common
    /// ancestor of every pair, in a fresh domain `vt.K` in preorder, so `for x in vt.K.rev` visits
    /// children first.
    /// `vt.at[x]` is the vertex, `vt.parent[x]` its nearest kept ancestor, `vt.w[x]` the weighted
    /// distance to it (0 at the top) and `vt.key[x]` whether it is a key.
    pub fn virtual(self, keys: List<V>) -> VirtualTree<V, _> = todo

    /// O(1). The edge length from `v` up to its parent: 0 at a root, 1 unless built with `w:`.
    pub fn w(self, v: V) -> Int = todo

    /// O(depth(v)). `v`, its parent, and so on up to the root.
    pub fn ancestors(self, v: V) -> yields V = todo

    /// O(1). The top of the heavy chain through `v`: the highest vertex reached by heavy edges.
    pub fn top(self, v: V) -> V = todo

    /// O(1). The heavy chain through `v` as one range of positions, its top first.
    pub fn chain(self, v: V) -> Range<Pre> = todo

    /// O(log n). The subtree of `v` were the tree rooted at `root`, as at most two ranges of
    /// positions: `sub(v)` when `root` is outside it, all of `Pre` when `v == root`, and
    /// otherwise everything except the subtree of the child of `v` toward `root`.
    pub fn sub_at(self, v: V, root: V) -> yields Range<Pre> = todo

    /// O(log n). Whether `c` lies on the path from `a` to `b`.
    pub fn on_path(self, c: V, a: V, b: V) -> Bool = todo

    /// O(log n). The one vertex on all three paths between `a`, `b` and `c`: the deepest of
    /// their pairwise lowest common ancestors.
    pub fn meet(self, a: V, b: V, c: V) -> V = todo

    /// O(n) calls of `lift` and `finish`. The other half of `reroot`: for `v` that is not a
    /// root, the DP of the tree without `sub(v)`, rooted at `parent(v)`; `none` at a root.
    /// Takes the arguments of `reroot`, for answers that are per edge rather than per vertex.
    pub fn above<T, U, M>(
        self,
        op: M,
        lift: (T, V, V) -> U = (x, c, p) => x,
        finish: (U, V) -> T,
    ) -> [V]?T = todo

    /// O(log n). The farthest pair among two vertex sets' farthest pairs, which is the
    /// farthest pair of their union. An operation value with identity `none`:
    /// `SegTree(op: (x, y) => t.farthest(x, y), id: none)` keeps the diameter of any range
    /// of vertices under changes, and a running fold keeps it as vertices arrive.
    pub fn farthest(
        self,
        x: ?(len: Int, u: V, v: V),
        y: ?(len: Int, u: V, v: V),
    ) -> ?(len: Int, u: V, v: V) = todo

    /// O(n). Every vertex ordered by depth, each depth in preorder, over a fresh domain, so
    /// the descendants of `v` at one depth are one range.
    pub fn layers(self) -> Layers<V, _> = todo

    /// O(n). The 2n enter and exit events of `tour` over a fresh domain, for Mo's algorithm
    /// on paths and on subtrees.
    pub fn events(self) -> Events<V, _> = todo

    /// O(1) per item. The closed Euler walk: 2n - 1 visits from the root back to it, each pair of
    /// neighbours joined by a tree edge, children in child order. A walk that visits every vertex
    /// of a connected graph is `Rooted(g, root: r).euler_tour()`.
    pub fn euler_tour(self) -> yields V = todo

    /// O(n). The long-path decomposition, for DP indexed by depth merged in O(n) in total.
    pub fn long_paths(self) -> LongPaths<V, _> = todo

    /// O(size(v)) and less when pruned. The subtree of `v` in preorder as `std.dp` steps:
    /// `step.state` is the vertex, `step.depth` its depth below `v`, and `step.prune()` skips
    /// the rest of its subtree (decision 126).
    pub fn preorder(self, v: V) -> yields Step<V> = todo

/// A virtual tree from `Rooted.virtual`: a few vertices of a tree and their pairwise lowest
/// common ancestors, over its own domain `K` in preorder.
pub struct VirtualTree<V, K>:
    at: [K]V
    parent: [K]?K
    w: [K]Int
    key: [K]Bool

/// A centroid decomposition: the centroid of the tree, then recursively of each piece left when
/// it is removed. Every path passes through the centroid ancestor that first separates its
/// ends, and each vertex has at most log2 n + 1 centroid ancestors, so a question about all
/// paths through `v` is a loop over `ancestors(v)`.
/// Built in O(n log n) time and memory without recursion.
pub struct Centroids<V>:
    root: V
    cpar: [V]?V
    cdep: [V]0..=V.count
    /// For each vertex, every centroid whose piece holds it, nearest first, with the
    /// distance and the neighbour of the centroid it hangs off.
    anc: [V]List<(c: V, dist: Int, branch: V)>
    /// For each centroid, the vertices of its piece in breadth-first order from it.
    held: [V]List<V>

    /// O(n log n). Decompose the tree given by `next` (as for `Rooted`: a `Graph`, neighbour lists
    /// or a fn).
    pub fn new<N>(next: N) -> Centroids<V> = todo

    /// O(1). The parent of centroid `c` in the centroid tree; `none` at `cd.root`.
    pub fn parent(self, c: V) -> ?V = todo

    /// O(1). The depth of `c` in the centroid tree, at most log2 n.
    pub fn depth(self, c: V) -> Int = todo

    /// O(log n). Every centroid whose piece holds `v`, nearest first (`v` itself, with distance 0),
    /// with the tree distance from it. At most log2 n + 1 entries, O(1) each, no allocation.
    pub fn ancestors(self, v: V) -> yields (c: V, dist: Int) = todo

    /// O(size of the piece). The piece of centroid `c`: each vertex `v`, its distance from `c`, and
    /// `branch`, the neighbour of `c` on the way to `v` (`c` itself for `v == c`). Pairs in
    /// different branches are exactly the paths whose highest centroid is `c`.
    pub fn piece(self, c: V) -> yields (v: V, dist: Int, branch: V) = todo

// ---- tests

/// A random tree on `V` as adjacency lists, with arcs in both directions.
fn random_tree<V>(any: V) -> [V]List<V> = todo

/// The vertices from `a` to `b`, by climbing parents.
fn slow_path<V, Pre>(r: Rooted<V, Pre>, a: V, b: V) -> List<V> = todo

@test
fn subtrees_and_chains_are_contiguous() = todo

@test
fn a_table_of_neighbour_lists_roots_like_the_fn_that_reads_it() = todo

@test
fn lca_dist_ancestor_and_step_match_climbing() = todo

@test
fn weighted_dist_adds_the_lengths_on_the_path() = todo

@test
fn path_folds_keep_the_order_from_a_to_b() = todo

struct Word(s: Str)

impl Reversible for Word:
    fn rev(self) -> Word = todo

@test
fn path_updates_and_subtree_folds_over_one_lazy_tree() = todo

@test
fn centroids_leave_no_piece_over_half() = todo

@test
fn excl_fold_folds_the_other_children() = todo

@test
fn rerooting_counts_connected_sets_through_every_vertex() = todo

@test
fn orders_lifting_and_the_virtual_tree() = todo

@test
fn a_tree_from_a_stored_graph() = todo

@test
fn from_parents_tour_pos_and_value_owned_pre() = todo

@test
fn weighted_rooted_bfs_tree_on_cyclic_graph() = todo

@test
fn lower_is_the_child_end_of_an_edge() = todo

@test
fn path_and_diameter_match_pairwise_distances() = todo

@test
fn heavy_light_and_subtree_cover_the_children() = todo

@test
fn a_forest_keeps_each_tree_in_one_range() = todo

@test
fn down_and_up_match_depth_size_and_height() = todo

@test
fn reroot_matches_up_from_every_root() = todo

@test
fn shapes_match_the_canonical_string() = todo

@test
fn centroid_ancestors_split_every_path() = todo

@test
fn a_jump_tree_answers_what_the_rooted_tree_does() = todo

/// A rooted forest grown one leaf at a time, by skew-binary jump pointers: `add` is O(1),
/// queries are O(log n), and it keeps three arrays. Every vertex starts as its own root.
pub struct JumpTree<V>:
    par: [V]?V
    jump: [V]V
    dep: [V]0..=V.count

    /// O(n). A forest of lone vertices, each its own root.
    pub fn new() -> JumpTree<V> = todo

    /// O(1). Hang `v`, a vertex not yet in any tree, below `p`.
    pub fn add(mut self, v: V, p: V) = todo

    /// O(1). The parent of `v`, or `none` at a root.
    pub fn parent(self, v: V) -> ?V = todo

    /// O(1). The number of edges from `v` up to its root.
    pub fn depth(self, v: V) -> Int = todo

    /// O(log n). The root of `v`'s tree.
    pub fn root_of(self, v: V) -> V = todo

    /// O(log n). The ancestor `k` edges above `v`, or `none` if that is above the root.
    pub fn ancestor(self, v: V, k: Int) -> ?V = todo

    /// O(log n). The lowest common ancestor of `a` and `b`, or `none` in different trees.
    pub fn lca(self, a: V, b: V) -> ?V = todo

    /// O(log n) calls. The highest ancestor `u` of `v` such that `pred` holds for `u`, or `none`
    /// when it does not hold for `v` itself. `pred` must be true and then false on the way up.
    pub fn jump_while(self, v: V, pred: V -> Bool) -> ?V = todo

    /// O(log n). The number of edges between `a` and `b`; `none` across trees.
    pub fn dist(self, a: V, b: V) -> ?Int = todo

    /// O(log n). The `k`-th vertex on the path from `a` to `b` (`a` is the 0th); `none` past
    /// `b` or across trees.
    pub fn jump(self, a: V, b: V, k: Int) -> ?V = todo

/// Binary lifting over a fixed forest given by parent links, each carrying a value on the edge
/// up: O(n log n) build and memory, O(log n) queries. A forest that grows leaf by leaf is a
/// `JumpTree`; one that changes is a `LinkCut`.
pub struct Lift<V, T>(op: (T, T) -> T, identity: T):
    up: [V]?V
    dep: [V]0..=V.count

    /// O(n log n). `par[v]` is `v`'s parent with the value on the edge to it, `none` at a root.
    /// `op` must be associative; it need not commute, since folds run from the lower end up.
    pub fn new(par: [V]?(V, T), op: (T, T) -> T, identity: T) -> Lift<V, T> = todo

    /// O(log n). Climb from `v` while the next vertex up satisfies `ok`, which must hold on a
    /// prefix of every root path: how many edges were climbed and where it stopped. Greedy
    /// "jump while the next one still fits" counts are `Lift(next, ...).climb(v, ok).steps`.
    pub fn climb(self, v: V, ok: V -> Bool) -> (steps: Int, at: V) = todo

    /// O(1). The parent of `v`, or `none` at a root.
    pub fn parent(self, v: V) -> ?V = todo

    /// O(1). The number of edges from `v` up to its root.
    pub fn depth(self, v: V) -> Int = todo

    /// O(log n). The ancestor `k` edges above `v`, or `none` if that is above the root.
    pub fn ancestor(self, v: V, k: Int) -> ?V = todo

    /// O(log n). The lowest common ancestor of `a` and `b`, or `none` in different trees.
    pub fn lca(self, a: V, b: V) -> ?V = todo

    /// O(log n). The fold of the edge values from `v` up to its ancestor `u`, lowest first;
    /// `identity` when `u == v`. Traps if `u` is not an ancestor of `v`.
    pub fn fold(self, v: V, up_to u: V) -> T = todo

    /// O(log n) calls. The highest ancestor `u` of `v` with `ok(self.fold(v, up_to: u))`, or `v`
    /// itself; `ok` must turn false at most once on the way up.
    pub fn climb_while(self, v: V, ok: T -> Bool) -> V = todo

/// A forest that changes: `link` and `cut` edges, re-root anywhere, and fold or set vertex
/// values along paths, by a link-cut tree (splay trees over preferred paths). Values fold
/// with `op`; a value that is not its own reverse implements `Reversible`, as for
/// `Rooted.fold_path`. Every operation is amortized O(log n).
pub struct LinkCut<V, T>(m: Monoid<T>):
    lc: [V]?V
    rc: [V]?V
    pa: [V]?V
    flip: [V]Bool
    val: [V]T
    agg: [V]T
    stk: List<V>

    /// O(n). `V.K` single-vertex trees holding `a` (the identity of `op` when `a` is absent).
    pub fn new<M>(op: M, a: ?[V]T = none) -> LinkCut<V, T> = todo

    fn top(self, x: V) -> Bool = todo

    fn turn(mut self, x: V) = todo

    fn push(mut self, x: V) = todo

    fn pull(mut self, x: V) = todo

    fn rotate(mut self, x: V) = todo

    fn splay(mut self, x: V) = todo

    // Makes root -> x the preferred path with x at the root of its splay tree; returns the
    // last vertex where the walk joined the old preferred path (the lca with the previous access).
    fn access(mut self, x: V) -> V = todo

    /// Amortized O(log n). Make `v` the root of its tree.
    pub fn make_root(mut self, v: V) = todo

    /// Amortized O(log n). The root of the tree holding `v`.
    pub fn root_of(mut self, v: V) -> V = todo

    /// Amortized O(log n). Whether `u` and `v` are in the same tree.
    pub fn connected(mut self, u: V, v: V) -> Bool = todo

    /// Amortized O(log n). Add the edge `u`-`v`; they must be in different trees, which costs one
    /// extra `root_of`.
    pub fn link(mut self, u: V, v: V) = todo

    /// Amortized O(log n). Remove the edge `u`-`v`, which must exist.
    pub fn cut(mut self, u: V, v: V) = todo

    /// Amortized O(log n). The lowest common ancestor of `u` and `v` when their tree is rooted at
    /// `root`, which becomes its root; `none` unless all three are connected.
    pub fn lca(mut self, u: V, v: V, root: V) -> ?V = todo

    /// O(1). The value at `v`.
    pub fn get(self, v: V) -> T = todo

    /// Amortized O(log n). Set the value at `v` to `x`.
    pub fn set(mut self, v: V, x: T) = todo

    /// Amortized O(log n). The fold of the values on the path from `u` to `v`, in that order; `u`
    /// becomes the root.
    pub fn fold_path(mut self, u: V, v: V) -> T = todo

    /// Amortized O(log n). The number of edges between `a` and `b`; `none` across trees.
    pub fn dist(mut self, a: V, b: V) -> ?Int = todo

    /// Amortized O(log n). The `k`-th vertex on the path from `a` to `b` (`a` is the 0th);
    /// `none` past `b` or across trees.
    pub fn jump(mut self, a: V, b: V, k: Int) -> ?V = todo

/// `LinkCut` with updates along a path: `LazyLinkCut.new(+, add, a)` adds to every value on
/// a path and folds paths, under the same `Action` as `SegTree(op, a, act:)`. Reversing a path must not
/// change how a tag acts, `apply(f, x.rev(), k) == apply(f, x, k).rev()`, which holds for every
/// commutative `over`. Every operation is amortized O(log n). Memory is about twice `LinkCut`'s.
pub struct LazyLinkCut<V, T, F>(act: Action<T, F>):
    lc: [V]?V
    rc: [V]?V
    pa: [V]?V
    flip: [V]Bool
    val: [V]T
    agg: [V]T
    /// The number of vertices in the splay subtree, the `span` that `apply` sees.
    cnt: [V]0..=V.count
    lz: [V]F
    stk: List<V>

    /// O(n). `V.K` single-vertex trees holding `a` (the identity of `act.over` when `a` is absent).
    pub fn new(act: Action<T, F>, a: ?[V]T = none) -> LazyLinkCut<V, T, F> = todo

    fn top(self, x: V) -> Bool = todo

    fn turn(mut self, x: V) = todo

    fn put(mut self, x: V, f: F) = todo

    fn push(mut self, x: V) = todo

    fn pull(mut self, x: V) = todo

    fn rotate(mut self, x: V) = todo

    fn splay(mut self, x: V) = todo

    fn access(mut self, x: V) -> V = todo

    /// Amortized O(log n). Make `v` the root of its tree.
    pub fn make_root(mut self, v: V) = todo

    /// Amortized O(log n). The root of the tree holding `v`.
    pub fn root_of(mut self, v: V) -> V = todo

    /// Amortized O(log n). Whether `u` and `v` are in the same tree.
    pub fn connected(mut self, u: V, v: V) -> Bool = todo

    /// Amortized O(log n). Add the edge `u`-`v`; they must be in different trees, which costs one
    /// extra `root_of`.
    pub fn link(mut self, u: V, v: V) = todo

    /// Amortized O(log n). Remove the edge `u`-`v`, which must exist.
    pub fn cut(mut self, u: V, v: V) = todo

    /// Amortized O(log n). The lowest common ancestor of `u` and `v` when their tree is rooted at
    /// `root`, which becomes its root; `none` unless all three are connected.
    pub fn lca(mut self, u: V, v: V, root: V) -> ?V = todo

    /// Amortized O(log n). The value at `v`, with every update along paths through it applied.
    pub fn get(mut self, v: V) -> T = todo

    /// Amortized O(log n). Set the value at `v` to `x`.
    pub fn set(mut self, v: V, x: T) = todo

    /// Amortized O(log n). The fold of the values on the path from `u` to `v`, in that order; `u`
    /// becomes the root.
    pub fn fold_path(mut self, u: V, v: V) -> T = todo

    /// Amortized O(log n). Apply `f` to every value on the path from `u` to `v`; `u` becomes the
    /// root.
    pub fn apply_path(mut self, u: V, v: V, f: F) = todo

    /// Amortized O(log n). The number of edges between `a` and `b`; `none` across trees.
    pub fn dist(mut self, a: V, b: V) -> ?Int = todo

    /// Amortized O(log n). The `k`-th vertex on the path from `a` to `b` (`a` is the 0th);
    /// `none` past `b` or across trees.
    pub fn jump(mut self, a: V, b: V, k: Int) -> ?V = todo

@test
fn link_cut_follows_a_changing_forest() = todo

@test
fn link_cut_matches_a_brute_forest_with_ordered_folds() = todo

@test
fn lazy_link_cut_matches_a_brute_forest() = todo

@test
fn lazy_link_cut_keeps_path_order_under_assignment() = todo

/// How a permutation-tree node joins its two children `l` (earlier positions) and `r`.
/// `Incr`: `r`'s values sit just above `l`'s; `Decr`: just below; `Partial`: the union is
/// not an interval on its own and only closes further up; `Full`: `r` fills the gaps of a
/// `Partial` `l`, closing an interval in neither order.
pub enum PermKind: Leaf, Incr, Decr, Partial, Full

/// A permutation-tree node as `PermTree.node` shows it: `Leaf(i)` for position `i`, or
/// `Inner(l, r, kind)` with its two children and how they join.
pub enum PermNode<I, N>: Leaf(i: I), Inner(l: N, r: N, kind: PermKind)

/// The binary permutation (interval-decomposition) tree of a bijection `p: [I]V`: every node
/// covers a run of positions, leaves are single positions, a node's children split its run in
/// two, and the values of every node but a `Partial` one form an interval. Nodes have their own
/// domain `t.Node` of 2n - 1 members; `kind`, `first`, `last`, `lo` and `hi` are arrays over it.
/// Built in O(n) by one stack pass.
pub struct PermTree<I, V, Node>:
    root: Node
    kind: [Node]PermKind
    l: [Node]Node
    r: [Node]Node
    first: [Node]I
    last: [Node]I
    lo: [Node]V
    hi: [Node]V

    /// O(n). The tree of `p`, which must be a bijection between two domains of equal size.
    pub fn new(p: [I]V) -> PermTree<I, V, _> = todo

    /// O(1). The view of `v` to match on (decision 132): `Leaf(i)` or `Inner(l, r, kind)`.
    pub fn node(self, v: Node) -> PermNode<I, Node> = todo

    /// O(1). The leaf of position `i`.
    pub fn leaf(self, i: I) -> Node = todo

    /// O(1). The earlier child of `v`, an inner node.
    pub fn left(self, v: Node) -> Node = todo

    /// O(1). The later child of `v`, an inner node.
    pub fn right(self, v: Node) -> Node = todo

    /// O(1). The positions `v` covers.
    pub fn span(self, v: Node) -> Range<I> = todo

    /// O(1). From the least to the greatest value `v` covers: exactly its values unless `v` is
    /// `Partial`.
    pub fn vals(self, v: Node) -> Range<V> = todo

@test
fn perm_tree_nodes_cover_intervals_of_values() = todo

@test
fn perm_tree_of_a_small_permutation() = todo

/// The vertices of a `Rooted` tree by depth, each depth in preorder, over their own domain
/// `B`: the descendants of a vertex at one depth are a contiguous range of `B`.
pub struct Layers<V, B>:
    at: [B]V

    /// O(1). The position of `v`.
    pub fn pos(self, v: V) -> B = todo

    /// O(1). Every vertex at depth `d`; empty past the deepest.
    pub fn level(self, d: Int) -> Range<B> = todo

    /// O(log n). The descendants of `v` at depth `d`; empty when there are none.
    pub fn below(self, v: V, d: Int) -> Range<B> = todo

/// The enter and exit events of a `Rooted` tree's `tour` over their own domain `P` of 2n
/// members, for Mo's algorithm on trees.
pub struct Events<V, P>:
    at: [P]Visit<V>

    /// O(1). The event that enters `v`.
    pub fn enter(self, v: V) -> P = todo

    /// O(1). The event that leaves `v`.
    pub fn exit(self, v: V) -> P = todo

    /// O(1). The events inside `v`'s subtree, `enter(v)` through `exit(v)`.
    pub fn sub(self, v: V) -> Range<P> = todo

    /// O(log n). The vertices met an odd number of times in `range`, plus `extra` when
    /// present, are exactly the vertices of the path from `a` to `b`.
    pub fn path(self, a: V, b: V) -> (range: Range<P>, extra: ?V) = todo

/// The long-path decomposition of a `Rooted` tree: each vertex continues the path of its
/// tallest child, so a DP indexed by depth below `v` lives in one shared buffer over `L`,
/// inheriting `deep[v]`'s values is free, and merging the other children costs their heights.
pub struct LongPaths<V, L>:
    /// The child with the tallest subtree; `none` at a leaf.
    deep: [V]?V
    /// The number of edges on the longest path down from `v`.
    height: [V]Int

    /// O(1). The slots of `v`, one per depth offset `0..=height[v]`, offset 0 first;
    /// `slots(deep[v])` starts one later in the same buffer.
    pub fn slots(self, v: V) -> Range<L> = todo

    /// O(1). The slot of `v` at depth offset `d`; traps unless `0 <= d <= height[v]`.
    pub fn slot(self, v: V, d: Int) -> L = todo

/// Lowest common ancestors in O(1) after O(n log n) preprocessing, by a sparse table over the
/// Euler tour: for many queries on a fixed tree when `Rooted.lca` is the bottleneck.
pub struct LcaTable<V>:
    first: [V]Int
    far: [V]Int

    /// O(n log n) time and memory.
    pub fn new<Pre>(t: Rooted<V, Pre>) -> LcaTable<V> = todo

    /// O(1). The lowest common ancestor of `a` and `b`.
    pub fn lca(self, a: V, b: V) -> V = todo

    /// O(1). The distance between `a` and `b`, weighted as the tree was built.
    pub fn dist(self, a: V, b: V) -> Int = todo

/// A forest that changes, with folds over whole trees and subtrees, by an Euler tour tree
/// (a balanced sequence of each tree's tour). Where `LinkCut` folds paths, this folds
/// components. Values fold with `op`, which must commute. Every operation is expected
/// O(log n).
pub struct EulerTourTree<V, T>:
    val: [V]T

    /// O(n). Lone vertices with values `a` (each `op`'s identity when `a` is `none`).
    pub fn new<M>(op: M, a: ?[V]T = none) -> EulerTourTree<V, T> = todo

    /// Expected O(log n). Join the trees of `u` and `v` by the edge `(u, v)`.
    pub fn link(mut self, u: V, v: V) = todo

    /// Expected O(log n). Remove the edge `(u, v)`.
    pub fn cut(mut self, u: V, v: V) = todo

    /// Expected O(log n). Whether `a` and `b` are in one tree.
    pub fn connected(mut self, a: V, b: V) -> Bool = todo

    /// Expected O(log n). The number of vertices in `v`'s tree.
    pub fn len(mut self, v: V) -> Int = todo

    /// O(1). The value at `v`.
    pub fn get(self, v: V) -> T = todo

    /// Expected O(log n). Replace the value at `v`.
    pub fn set(mut self, v: V, x: T) = todo

    /// Expected O(log n). The fold of every value in `v`'s tree.
    pub fn fold_tree(mut self, v: V) -> T = todo

    /// Expected O(log n). The fold over `v`'s side of the edge `(v, away)`.
    pub fn fold_sub(mut self, v: V, away: V) -> T = todo

/// `EulerTourTree` with updates to whole trees and subtrees under an action.
pub struct LazyEulerTourTree<V, T, F>:
    val: [V]T

    /// O(n). Lone vertices with values `a`.
    pub fn new(act: Action<T, F>, a: ?[V]T = none) -> LazyEulerTourTree<V, T, F> = todo

    /// Expected O(log n). Join the trees of `u` and `v` by the edge `(u, v)`.
    pub fn link(mut self, u: V, v: V) = todo

    /// Expected O(log n). Remove the edge `(u, v)`.
    pub fn cut(mut self, u: V, v: V) = todo

    /// Expected O(log n). Whether `a` and `b` are in one tree.
    pub fn connected(mut self, a: V, b: V) -> Bool = todo

    /// Expected O(log n). The number of vertices in `v`'s tree.
    pub fn len(mut self, v: V) -> Int = todo

    /// Expected O(log n). The value at `v`.
    pub fn get(mut self, v: V) -> T = todo

    /// Expected O(log n). Replace the value at `v`.
    pub fn set(mut self, v: V, x: T) = todo

    /// Expected O(log n). The fold of every value in `v`'s tree.
    pub fn fold_tree(mut self, v: V) -> T = todo

    /// Expected O(log n). The fold over `v`'s side of the edge `(v, away)`.
    pub fn fold_sub(mut self, v: V, away: V) -> T = todo

    /// Expected O(log n). Apply `f` to every value in `v`'s tree.
    pub fn apply_tree(mut self, v: V, f: F) = todo

    /// Expected O(log n). Apply `f` to every value on `v`'s side of the edge `(v, away)`.
    pub fn apply_sub(mut self, v: V, away: V, f: F) = todo

/// O(n). The Cartesian tree of `a` as parent links: the root is the least element by `by`
/// (the leftmost among equals) and every subtree is a contiguous range of `D`. Root it with
/// `Rooted.from_parents`.
pub fn cartesian_tree<D, T>(a: [D]T, by: (T, T) -> Bool = (x, y) => x < y) -> [D]?D = todo

/// The Kruskal reconstruction tree: each edge that merges two components, in increasing
/// weight, becomes a node above them. The leaves under a node are the vertices connected by
/// edges of weight at most its own, so "what `v` reaches under a limit" is one subtree.
pub struct KruskalTree<V, N, W>:
    /// The leaf node of each vertex.
    leaf: [V]N
    /// The merging weight of each node; `none` at a leaf.
    w: [N]?W

    /// O(1). The nodes as a `Rooted` forest over `N`: subtree ranges, LCA and folds.
    pub @get fn tree(self) -> Rooted<N, _> = todo

    /// O(log n). The highest node above `leaf[v]` whose weight is at most `limit`; its
    /// leaves are what `v` reaches by edges of weight at most `limit`.
    pub fn reach(self, v: V, limit: W) -> N = todo

    /// O(log n). The least possible largest edge weight on a path from `a` to `b`; `none`
    /// when they are not connected, and at `a == b`.
    pub fn bottleneck(self, a: V, b: V) -> ?W = todo

/// O(m log m). The Kruskal reconstruction tree of `edges` weighted by `w`, over the vertices
/// `V`, taking the same arguments as `std.graph.kruskal`; its nodes are a fresh domain of at
/// most 2|V| - 1 members, bound by the caller: `let (domain N, kt) = kruskal_tree(es, w: f)`.
/// Merging by the largest weight first is `w: e => desc(e.alt)`, and "edges above `p`" is then
/// `kt.reach(v, limit: desc(p + 1))`: any `Ord` weight works, so no weight is ever negated.
pub fn kruskal_tree<V, E, W>(edges: [E](u: V, v: V, ..), w: E -> W) -> (domain N, KruskalTree<V, N, W>) = todo

/// O(n). The Prüfer code of a tree on `V`, which needs at least two vertices: `V.count - 2`
/// vertices, in bijection with the labelled trees.
pub fn prufer<V>(edges: Tree<V>) -> List<V> = todo

/// O(n). The tree whose Prüfer code is `code`, which must have `V.count - 2` entries.
pub fn from_prufer<V>(code: List<V>) -> Tree<V> = todo
