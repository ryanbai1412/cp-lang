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
    /// The parent; `None` at the root.
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
    /// The child with the largest subtree; `None` at a leaf.
    heavy: [V]?V
    /// The highest vertex of the heavy chain through `v`.
    top: [V]V
    tin: [V]Pre
    /// The inverse of `tin` / `pos`.
    at: [Pre]V

    /// Roots the tree given by `next` at `root`: `Rooted(g, root: r)` for a `Graph`,
    /// `Rooted(lists, root: r)` for a table of neighbour lists `[V]List<V>`, and
    /// `Rooted(root: r, next: u => ...)` for anything else.
    ///
    /// `next` may list every neighbour (the parent is skipped) or only the children, as vertices or
    /// as arcs. The tree is the breadth-first spanning tree from `root`, children in the order `next`
    /// lists them, so on a graph with extra edges `depth` is the distance from the root. Edge
    /// lengths come from the arcs, as in every graph algorithm: `Weighted` arcs (a `w` field, or
    /// `g.weighted(f)`) give `dist`, `w` and `diameter` their lengths; others count 1 per edge.
    /// The heavy child of a vertex is the child with the largest subtree, the first in child order
    /// on ties; `t.Pre` is the preorder that visits the heavy child first and then the other
    /// children in child order.
    ///
    /// cost: O(n + m) for m listed arcs, without recursion
    /// traps: when some vertex is not reachable from `root`
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.children(0).collect() == [1, 2] and t.children(1).collect() == [3, 4] and t.depth(4) == 2`
    pub fn new<N>(next: N, root: V) -> Rooted<V, _> = todo

    /// Builds a rooted tree from parents keyed by vertex: `par: [V]?V` with `None` at the root, or
    /// the usual input shape "parents of 2..n" read as `read<[V.drop(1)]V>()`, whose one missing key
    /// is the root.
    ///
    /// Several roots give a forest, as `forest` does, with the roots in key order. Children are in
    /// increasing key order.
    ///
    /// cost: O(n)
    /// empty: a `par` with no root, empty or cyclic, traps
    /// traps: when the parents have a cycle; `try_from_parents` is for input that may
    /// example: `domain V = 0..3; let par: [V]?V = [None, Some(0), Some(1)]; let t = Rooted.from_parents(par); t.depth(2) == 2 and t.parent(1) == Some(0)`
    pub fn from_parents<P: Keyed>(par: P) -> Rooted<V, _> = todo

    /// `from_parents` for parents that may not form a forest: `None` when they have a cycle.
    ///
    ///     let t = Rooted.try_from_parents(up) else: print("NO"); return
    ///
    /// cost: O(n)
    /// empty: `None` when `par` has no root
    /// traps: never
    /// example: `domain V = 0..2; let par: [V]?V = [Some(1), Some(0)]; Rooted.try_from_parents(par) == None`
    pub fn try_from_parents<P: Keyed>(par: P) -> ?Rooted<V, _> = todo

    /// One tree per root: `next` must reach every vertex from exactly one of `from`.
    ///
    /// Each tree is built as `new` builds it. The trees sit one after another in `t.Pre`, in the
    /// order of `from`; `t.root` is the first root. Paths and lowest common ancestors exist only
    /// within one tree (`root_of` tells).
    ///
    /// cost: O(n + m)
    /// empty: `from` must be nonempty
    /// traps: when `from` is empty, or some vertex is reached from no root or from two
    /// example: `domain V = 0..4; let adj: [V]List<V> = [[1], [0], [3], [2]]; let t = Rooted.forest(adj, from: [2, 0]); t.roots.collect() == [2, 0] and t.root_of(1) == 0`
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

    /// The roots in the order given: one for `new`, one per tree for `forest`, key order for
    /// `from_parents`.
    ///
    /// cost: O(1) to make, O(1) per root
    /// order: as given at construction
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.roots.collect() == [0]`
    pub @get fn roots(self) -> yields V = todo

    /// The position of `v` in this tree's own preorder domain `t.Pre`; the inverse of `at`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.at[t.pos(3)] == 3 and t.pos(1) < t.pos(3) and t.pos(4) < t.pos(2)`
    pub fn pos(self, v: V) -> Pre = todo

    /// The parent of `v`; `None` at a root.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.parent(3) == Some(1) and t.parent(0) == None`
    pub fn parent(self, v: V) -> ?V = todo

    /// The arc `v` was entered by, with every field `next` gave it (`t.up_arc(v)!.cost` on a tree
    /// read with a `cost` field); `None` at a root. Edge data stays on the arcs, so no `[V]` side
    /// array is built from the edge list.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..3; let es: [0..2](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]; let t = Rooted(Graph.undirected(es), root: 0); t.up_arc(2)!.to == 2 and t.up_arc(0) == None`
    pub fn up_arc<A>(self, v: V) -> ?A = todo

    /// Reorders a vertex-indexed array into preorder: `out[t.pos(v)] == a[v]`, ready to build a
    /// `SegTree` or `Fenwick` over `t.Pre`.
    ///
    /// cost: O(n)
    /// empty: never empty; a tree has a vertex
    /// order: by `t.Pre`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.reorder([10, 11, 12, 13, 14]).collect() == [10, 11, 13, 14, 12]`
    pub fn reorder<T>(self, a: [V]T) -> [Pre]T = todo

    /// The children of `v`, in the order `next` listed them. Children are stored contiguously, so
    /// nothing is allocated.
    ///
    /// cost: O(1) to start, O(1) per child
    /// empty: nothing at a leaf
    /// order: as `next` listed them
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.children(1).collect() == [3, 4] and t.children(2).count() == 0`
    pub fn children(self, v: V) -> yields V = todo

    /// The arcs from `v` down to its children, with every field `next` gave them
    /// (`for a in t.child_arcs(v): up(a.to) + a.w`), so edge data needs no side array.
    ///
    /// cost: O(1) per child
    /// empty: nothing at a leaf
    /// order: child order
    /// traps: never
    /// example: `domain V = 0..3; let es: [0..2](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]; let t = Rooted(Graph.undirected(es), root: 0); t.child_arcs(1).map(a => a.to).collect() == [2]`
    pub fn child_arcs<A>(self, v: V) -> yields A = todo

    /// For each child `c` of `v`, in child order, the fold with `op` of `f` over `v`'s other
    /// children: the step of a rerooting DP, with no division or inverse.
    ///
    /// The fold keeps child order (earlier siblings, then later ones), so `op` need not commute;
    /// with one child, `rest` is the identity. Prefix and suffix folds make it O(k) for `k`
    /// children.
    ///
    ///     for (c, rest) in t.excl_fold(v, mul, c => down[c] + 1):
    ///         up[c] = (up[v] * rest + 1) % m
    ///
    /// cost: O(k) calls of `f` and of `op`, O(k) memory
    /// empty: nothing at a leaf
    /// order: child order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.excl_fold(0, add, c => t.size(c)).collect() == [(c: 1, rest: 1), (c: 2, rest: 3)]`
    pub fn excl_fold<T, M>(self, v: V, op: M, f: V -> T) -> yields (c: V, rest: T) = todo

    /// The number of edges between `v` and its root.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.depth(0) == 0 and t.depth(4) == 2`
    pub fn depth(self, v: V) -> Int = todo

    /// The number of vertices in the subtree of `v`, `v` included.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.size(0) == 5 and t.size(1) == 3 and t.size(2) == 1`
    pub fn size(self, v: V) -> Int = todo

    /// The children of `v` other than its heavy child, in child order. Every vertex has O(log n)
    /// light edges above it, which is what small-to-large merging pays for.
    ///
    /// cost: O(children)
    /// empty: nothing at a leaf
    /// order: child order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.light(0).collect() == [2] and t.light(1).collect() == [4]`
    pub fn light(self, v: V) -> yields V = todo

    /// The root of the tree that holds `v`: always `t.root` unless built as a forest.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.root_of(4) == 0`
    pub fn root_of(self, v: V) -> V = todo

    /// The vertices of `v`'s subtree in preorder, `v` first: `t.at[p] for p in t.sub(v)`.
    ///
    /// cost: O(size(v))
    /// empty: never empty; it holds `v`
    /// order: `t.Pre` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.subtree(1).collect() == [1, 3, 4] and t.subtree(0).collect() == [0, 1, 3, 4, 2]`
    pub fn subtree(self, v: V) -> yields V = todo

    /// A top-down pass: `out[v] = f(out[parent], v)`, and `f(init, r)` at a root `r`. The state on
    /// the path from the root, such as a prefix maximum or a distance.
    ///
    /// cost: O(n) calls of `f`
    /// empty: never empty
    /// order: keyed by `V`; `f` runs in `t.Pre` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.down(0, (d, v) => d + 1) == [1, 2, 2, 3, 3]`
    pub fn down<T>(self, init: T, f: (T, V) -> T) -> [V]T = todo

    /// A bottom-up tree DP: `out[v] = finish(fold with op of lift(out[c], c, v) over children c, v)`.
    ///
    /// `lift` sees both ends of the edge, child first, and defaults to the child's value. Children
    /// fold in child order, so `op` need not commute; a leaf folds nothing and gets the identity.
    ///
    /// cost: O(n) calls of `lift`, `op` and `finish`
    /// empty: never empty
    /// order: keyed by `V`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.up(add, finish: (s, v) => s + 1) == [5, 3, 1, 1, 1]`
    pub fn up<T, U, M>(
        self,
        op: M,
        lift: (T, V, V) -> U = (x, c, p) => x,
        finish: (U, V) -> T,
    ) -> [V]T = todo

    /// The same DP as `up`, for every root at once: `out[v]` is what `up` would give at `v` if the
    /// tree were rooted at `v`.
    ///
    /// Each vertex folds its parent's side first, then its children in child order; the sibling
    /// folds use prefixes and suffixes, so `op` needs no inverse.
    ///
    /// cost: O(n) calls of `lift` and `finish`, O(n) calls of `op`
    /// empty: never empty
    /// order: keyed by `V`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.reroot(add, lift: (x, c, p) => 1, finish: (s, v) => s) == [2, 3, 1, 1, 1]`
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.reroot(add, finish: (s, v) => s + 1) == [5, 5, 5, 5, 5]`
    pub fn reroot<T, U, M>(
        self,
        op: M,
        lift: (T, V, V) -> U = (x, c, p) => x,
        finish: (U, V) -> T,
    ) -> [V]T = todo

    /// The isomorphism class of every subtree: `of[a] == of[b]` exactly when the subtrees of `a` and
    /// `b` are the same rooted tree up to reordering children.
    ///
    /// Classes are numbered in reverse preorder (children before parents) by first appearance, so
    /// a leaf, if any, has class `Shape.first`.
    ///
    /// cost: O(n log n)
    /// empty: never empty
    /// order: `of` keyed by `V`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let (domain S, of) = t.shapes(); of[3] == of[2] and of[1] != of[0] and S.count == 3`
    pub fn shapes(self) -> (domain Shape, of: [V]Shape) = todo

    /// The size of every subtree, by vertex: `t.subtree_sizes[v] == t.size(v)`.
    ///
    /// cost: O(1); a view of the stored sizes
    /// empty: never empty
    /// order: keyed by `V`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.subtree_sizes == [5, 3, 1, 1, 1]`
    pub @get fn subtree_sizes(self) -> [V]Int = todo

    /// A centroid of the tree holding `t.root`: a vertex whose removal leaves no piece of more than
    /// half its vertices. Found by walking heavy children down from the root while the heavy
    /// child's subtree is more than half.
    ///
    /// cost: O(depth)
    /// ties: of two centroids, the one nearer the root
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.centroid() == 1`
    pub fn centroid(self) -> V = todo

    /// Every centroid of the tree holding `t.root`, one or two. Two happen only when the number of
    /// vertices is even and one edge splits the tree in halves.
    ///
    /// cost: O(depth)
    /// empty: never empty
    /// order: the one nearer the root first
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.centroids() == [1]`
    /// example: `domain P = 0..4; let adj: [P]List<P> = [[1], [0, 2], [1, 3], [2]]; Rooted(adj, root: 0).centroids() == [1, 2]`
    pub fn centroids(self) -> List<V> = todo

    /// The enter/exit Euler tour, the `Visit` events of `std.graph.dfs`: `Enter(v)` when entering
    /// `v` and `Exit(v)` when leaving its subtree, children in `t.Pre` order, each tree of a forest
    /// in turn. Iterative, so deep trees are safe.
    ///
    /// cost: O(1) to make, O(n) to walk
    /// empty: never empty
    /// order: `Enter` events in `t.Pre` order; `Exit(v)` right after the last event of `v`'s subtree
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.tour.collect() == [Enter(0), Enter(1), Enter(3), Exit(3), Enter(4), Exit(4), Exit(1), Enter(2), Exit(2), Exit(0)]`
    pub @get fn tour(self) -> yields Visit<V> = todo

    /// The positions of `v`'s subtree, `v` first: one contiguous range of `t.Pre`.
    ///
    /// cost: O(1)
    /// empty: never empty; it holds `t.pos(v)`
    /// order: increasing positions
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.sub(1).map(p => t.at[p]).collect() == [1, 3, 4]`
    pub fn sub(self, v: V) -> Range<Pre> = todo

    /// The positions of `v`'s descendants: `sub(v)` without `v`.
    ///
    /// cost: O(1)
    /// empty: empty at a leaf
    /// order: increasing positions
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.below(1).map(p => t.at[p]).collect() == [3, 4] and t.below(2).count() == 0`
    pub fn below(self, v: V) -> Range<Pre> = todo

    /// Whether `a` is an ancestor of `b`; a vertex is its own ancestor.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.is_anc(1, 3) and t.is_anc(3, 3) and not t.is_anc(2, 3)`
    pub fn is_anc(self, a: V, b: V) -> Bool = todo

    /// The child end of the tree edge between `u` and `v`, in either order.
    ///
    /// cost: O(1)
    /// ties: none; exactly one end is the child
    /// traps: when `u` and `v` are not joined by a tree edge
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.lower(0, 1) == 1 and t.lower(3, 1) == 3`
    pub fn lower(self, u: V, v: V) -> V = todo

    /// The lowest common ancestor of `a` and `b`, by heavy-path climbing.
    ///
    /// cost: O(log n)
    /// traps: when `a` and `b` are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.lca(3, 4) == 1 and t.lca(3, 2) == 0 and t.lca(1, 3) == 1`
    pub fn lca(self, a: V, b: V) -> V = todo

    /// The length of the path between `a` and `b`: its number of edges, or the total of its edge
    /// lengths in a tree built from `Weighted` arcs.
    ///
    /// cost: O(log n)
    /// traps: when `a` and `b` are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.dist(3, 2) == 3 and t.dist(4, 4) == 0`
    pub fn dist(self, a: V, b: V) -> Int = todo

    /// The simple path of vertices from `a` to `b`, both included.
    ///
    /// cost: O(log n + |path|)
    /// empty: never empty; `[a]` when `a == b`
    /// order: from `a` to `b`
    /// traps: when `a` and `b` are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.path(3, 2) == [3, 1, 0, 2] and t.path(4, 4) == [4]`
    pub fn path(self, a: V, b: V) -> List<V> = todo

    /// A longest simple path in the tree holding `t.root`: `(len, u, v)` with
    /// `t.dist(u, v) == len`, by two sweeps: `u` is farthest from the root, `v` farthest from `u`.
    ///
    /// cost: O(n)
    /// ties: in each sweep the farthest vertex first in `t.Pre` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.diameter() == (len: 3, u: 3, v: 2)`
    pub fn diameter(self) -> (len: Int, u: V, v: V) = todo

    /// The ancestor `k` edges above `v` (`v` itself for `k == 0`), or `None` if that is above the
    /// root.
    ///
    /// cost: O(log n)
    /// traps: when `k` is negative
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.ancestor(3, 2) == Some(0) and t.ancestor(3, 3) == None and t.ancestor(3, 0) == Some(3)`
    pub fn ancestor(self, v: V, k: Int) -> ?V = todo

    /// The vertex `k` edges from `a` on the path to `b` (`a` for `k == 0`), or `None` if the path
    /// is shorter.
    ///
    /// cost: O(log n)
    /// traps: when `k` is negative, or `a` and `b` are in different trees
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.jump(3, 2, 2) == Some(0) and t.jump(3, 2, 3) == Some(2) and t.jump(3, 2, 4) == None`
    pub fn jump(self, a: V, b: V, k: Int) -> ?V = todo

    /// The highest ancestor `u` of `v` such that `pred(u)` holds, or `None` when `pred(v)` is false.
    /// `pred` must be true and then false on the way up, and is checked only where the search
    /// probes it.
    ///
    /// cost: O(log² n) calls of `pred`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.jump_while(3, v => v != 0) == Some(1) and t.jump_while(3, v => v == 0) == None`
    pub fn jump_while(self, v: V, pred: V -> Bool) -> ?V = todo

    /// The path from `a` to `b` as O(log n) heavy-chain ranges of `t.Pre`, in path order. A
    /// `rising` range is walked from its high end to its low end. With `edges`, the highest vertex
    /// of the path is left out: for a value per edge, stored at the edge's lower end.
    ///
    /// cost: O(log n) ranges
    /// empty: with `edges` and `a == b`, nothing; otherwise never empty
    /// order: from `a` to `b`
    /// traps: when `a` and `b` are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.chains(3, 2).map(c => c.rising).collect() == [true, false] and t.chains(3, 2, edges: true).count() == 2`
    pub fn chains(
        self,
        a: V,
        b: V,
        edges: Bool = false,
    ) -> yields (range: Range<Pre>, rising: Bool) = todo

    /// The fold of the values on the path from `a` to `b`, in that order, read from `ds`: any
    /// structure over `t.Pre` with `fold(range)` and a `monoid` (`SegTree`).
    ///
    /// A stretch of the path that climbs is folded top-down by `ds` and then turned round with
    /// `rev()`: a value that is not its own reverse implements `Reversible`. With `edges`, the
    /// value at the lowest common ancestor is left out.
    ///
    /// cost: O(log² n) with one structure
    /// empty: the identity for `a == b` with `edges`
    /// traps: when `a` and `b` are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let seg = SegTree(+, t.reorder([1, 2, 4, 8, 16])); t.fold_path(3, 2, seg) == 15 and t.fold_path(3, 2, seg, edges: true) == 14`
    pub fn fold_path<S, T>(self, a: V, b: V, ds: S, edges: Bool = false) -> T = todo

    /// Applies the update `tag` to every vertex on the path from `a` to `b`, through
    /// `ds.apply(range, tag)` once per chain range. With `edges`, the lowest common ancestor is
    /// left out.
    ///
    /// cost: O(log² n)
    /// empty: nothing changes for `a == b` with `edges`
    /// traps: when `a` and `b` are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); var seg = SegTree(+, t.reorder([0, 0, 0, 0, 0]), act: add); t.apply_path(3, 2, seg, 5); t.fold_path(3, 2, seg) == 20 and seg.fold(t.sub(1)) == 10`
    pub fn apply_path<S, F>(self, a: V, b: V, mut ds: S, tag: F, edges: Bool = false) = todo

    /// The tree squeezed down to `keys`: the keys and the lowest common ancestor of every pair, in
    /// a fresh domain `vt.K` in preorder, so `for x in vt.K.rev` visits children first.
    ///
    /// `vt.at[x]` is the vertex, `vt.parent[x]` its nearest kept ancestor, `vt.w[x]` the
    /// weighted distance to it (0 at the top) and `vt.key[x]` whether it is a key. Repeated keys
    /// count once.
    ///
    /// cost: O(k log k + k log n) for k keys
    /// empty: `keys` empty gives an empty `vt.K`
    /// order: `vt.K` in `t.Pre` order
    /// traps: when the keys are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let vt = t.virtual([4, 3]); vt.at == [1, 3, 4] and vt.key == [false, true, true] and vt.w == [0, 1, 1]`
    pub fn virtual(self, keys: List<V>) -> VirtualTree<V, _> = todo

    /// The edge length from `v` up to its parent: 0 at a root, 1 unless the arcs are `Weighted`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.w(3) == 1 and t.w(0) == 0`
    pub fn w(self, v: V) -> Int = todo

    /// `v`, its parent, and so on up to its root.
    ///
    /// cost: O(1) per vertex, O(depth(v)) in all
    /// empty: never empty; it starts with `v`
    /// order: from `v` up
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.ancestors(3).collect() == [3, 1, 0]`
    pub fn ancestors(self, v: V) -> yields V = todo

    /// The top of the heavy chain through `v`: the highest vertex reached by heavy edges.
    ///
    /// cost: O(1)
    /// ties: none; the heavy children fix the chain
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.top(3) == 0 and t.top(4) == 4 and t.top(2) == 2`
    pub fn top(self, v: V) -> V = todo

    /// The heavy chain through `v` as one range of positions, its top first.
    ///
    /// cost: O(1)
    /// empty: never empty
    /// order: top first, down the chain
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.chain(3).map(p => t.at[p]).collect() == [0, 1, 3] and t.chain(2).count() == 1`
    pub fn chain(self, v: V) -> Range<Pre> = todo

    /// The subtree of `v` were the tree rooted at `root`, as at most two ranges of positions:
    /// `sub(v)` when `root` is outside it, all of `t.Pre` when `v == root`, and otherwise
    /// everything except the subtree of the child of `v` toward `root`.
    ///
    /// cost: O(log n)
    /// empty: empty ranges are skipped, so one or two ranges come out
    /// order: increasing positions
    /// traps: when `v` and `root` are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.sub_at(1, 3).flat_map(r => r.map(p => t.at[p])).collect() == [0, 1, 4, 2] and t.sub_at(1, 2).count() == 1`
    pub fn sub_at(self, v: V, root: V) -> yields Range<Pre> = todo

    /// Whether `c` lies on the path from `a` to `b`, ends included.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.on_path(1, 3, 2) and t.on_path(3, 3, 2) and not t.on_path(4, 3, 2)`
    pub fn on_path(self, c: V, a: V, b: V) -> Bool = todo

    /// The one vertex on all three paths between `a`, `b` and `c`: the deepest of their pairwise
    /// lowest common ancestors.
    ///
    /// cost: O(log n)
    /// traps: when the three are not in one tree
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.meet(3, 4, 2) == 1 and t.meet(3, 2, 0) == 0`
    pub fn meet(self, a: V, b: V, c: V) -> V = todo

    /// The other half of `reroot`: for `v` that is not a root, the DP of the tree without `sub(v)`,
    /// rooted at `parent(v)`; `None` at a root. Takes the arguments of `reroot`, for answers that
    /// are per edge rather than per vertex.
    ///
    /// cost: O(n) calls of `lift` and `finish`, O(n) calls of `op`
    /// empty: never empty
    /// order: keyed by `V`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.above(add, finish: (s, v) => s + 1) == [None, Some(2), Some(4), Some(4), Some(4)]`
    pub fn above<T, U, M>(
        self,
        op: M,
        lift: (T, V, V) -> U = (x, c, p) => x,
        finish: (U, V) -> T,
    ) -> [V]?T = todo

    /// The farthest pair among two vertex sets' farthest pairs, which is the farthest pair of
    /// their union; `None` is the identity. As an operation value,
    /// `SegTree(op: (x, y) => t.farthest(x, y), id: None)` keeps the diameter of any range of
    /// vertices under changes.
    ///
    /// cost: O(log n): at most four `dist` calls
    /// ties: the first longest of `x`, `y`, `(x.u, y.u)`, `(x.u, y.v)`, `(x.v, y.u)`, `(x.v, y.v)`
    /// traps: when the vertices are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.farthest(Some((len: 0, u: 3, v: 3)), Some((len: 0, u: 2, v: 2))) == Some((len: 3, u: 3, v: 2)) and t.farthest(None, None) == None`
    pub fn farthest(
        self,
        x: ?(len: Int, u: V, v: V),
        y: ?(len: Int, u: V, v: V),
    ) -> ?(len: Int, u: V, v: V) = todo

    /// Every vertex ordered by depth, each depth in `t.Pre` order, over a fresh domain, so the
    /// descendants of `v` at one depth are one range.
    ///
    /// cost: O(n)
    /// empty: never empty
    /// order: by depth, then `t.Pre`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ly = t.layers(); ly.level(1).map(b => ly.at[b]).collect() == [1, 2]`
    pub fn layers(self) -> Layers<V, _> = todo

    /// The 2n enter and exit events of `tour` over a fresh domain, for Mo's algorithm on paths and
    /// on subtrees.
    ///
    /// cost: O(n)
    /// empty: never empty
    /// order: `tour` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ev = t.events(); ev.sub(1).count() == 6`
    pub fn events(self) -> Events<V, _> = todo

    /// The closed Euler walk: 2n - 1 visits from the root back to it, each pair of neighbours
    /// joined by a tree edge, children in `t.Pre` order. A walk that visits every vertex of a
    /// connected graph is `Rooted(g, root: r).euler_tour()`. A forest walks only the first tree.
    ///
    /// cost: O(1) per item
    /// empty: never empty; `[r]` for a single vertex
    /// order: walk order from the root
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.euler_tour().collect() == [0, 1, 3, 1, 4, 1, 0, 2, 0]`
    pub fn euler_tour(self) -> yields V = todo

    /// The long-path decomposition, for DP indexed by depth merged in O(n) in total. The tallest
    /// child is the first in child order on ties.
    ///
    /// cost: O(n)
    /// empty: never empty
    /// ties: the first tallest child in child order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let lp = t.long_paths(); lp.height == [2, 1, 0, 0, 0] and lp.deep[1] == Some(3)`
    pub fn long_paths(self) -> LongPaths<V, _> = todo

    /// The subtree of `v` in `t.Pre` order as `std.dp` steps: `step.state` is the vertex,
    /// `step.depth` its depth below `v`, and `step.prune()` skips the rest of its subtree
    /// (decision 126).
    ///
    /// cost: O(size(v)), less when pruned
    /// empty: never empty; it starts with `v`
    /// order: `t.Pre` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.preorder(1).map(s => (s.state, s.depth)).collect() == [(1, 0), (3, 1), (4, 1)]`
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

    /// Decomposes the tree given by `next` (as for `Rooted`: a `Graph`, neighbour lists or a fn).
    ///
    /// The first centroid is `Rooted(next, root: V.first).centroid()`. Each piece left by removing
    /// a centroid `c` is rooted at its neighbour of `c` and gets the centroid nearer that root, the
    /// pieces taken in the order `next` lists `c`'s neighbours. Built without recursion.
    ///
    /// cost: O(n log n) time and memory
    /// ties: of two centroids in a piece, the one nearer the piece's root
    /// traps: when `next` is not a connected tree
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let cd = Centroids(adj); cd.root == 1 and cd.parent(2) == Some(0)`
    pub fn new<N>(next: N) -> Centroids<V> = todo

    /// The parent of centroid `c` in the centroid tree; `None` at `cd.root`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let cd = Centroids(adj); cd.parent(0) == Some(1) and cd.parent(1) == None`
    pub fn parent(self, c: V) -> ?V = todo

    /// The depth of `c` in the centroid tree, at most log2 n.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let cd = Centroids(adj); cd.depth(1) == 0 and cd.depth(3) == 1 and cd.depth(2) == 2`
    pub fn depth(self, c: V) -> Int = todo

    /// Every centroid whose piece holds `v`, with the tree distance from it: `v` itself at distance
    /// 0, then up the centroid tree. At most log2 n + 1 entries, no allocation.
    ///
    /// cost: O(1) per entry, O(log n) in all
    /// empty: never empty; it starts with `(c: v, dist: 0)`
    /// order: nearest centroid first, up to `cd.root`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let cd = Centroids(adj); cd.ancestors(2).collect() == [(c: 2, dist: 0), (c: 0, dist: 1), (c: 1, dist: 2)]`
    pub fn ancestors(self, v: V) -> yields (c: V, dist: Int) = todo

    /// The piece of centroid `c`: each vertex `v`, its distance from `c`, and `branch`, the
    /// neighbour of `c` on the way to `v` (`c` itself for `v == c`). Pairs in different branches are
    /// exactly the paths whose highest centroid is `c`.
    ///
    /// cost: O(size of the piece)
    /// empty: never empty; it starts with `(v: c, dist: 0, branch: c)`
    /// order: breadth-first from `c`, neighbours in the order `next` lists them
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let cd = Centroids(adj); cd.piece(0).collect() == [(v: 0, dist: 0, branch: 0), (v: 2, dist: 1, branch: 2)] and cd.piece(1).count() == 5`
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

    /// A forest of lone vertices, each its own root.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `domain V = 0..4; let j = JumpTree<V>(); j.parent(2) == None and j.root_of(2) == 2`
    pub fn new() -> JumpTree<V> = todo

    /// Hangs `v`, a lone vertex not yet in any tree, below `p`, setting its skew-binary jump
    /// pointer from `p`'s.
    ///
    /// cost: O(1)
    /// traps: when `v` already has a parent, or `v == p`
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.parent(2) == Some(1) and j.depth(3) == 2`
    pub fn add(mut self, v: V, p: V) = todo

    /// The parent of `v`, or `None` at a root.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.parent(1) == Some(0) and j.parent(0) == None`
    pub fn parent(self, v: V) -> ?V = todo

    /// The number of edges from `v` up to its root.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.depth(0) == 0 and j.depth(2) == 2`
    pub fn depth(self, v: V) -> Int = todo

    /// The root of `v`'s tree.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.root_of(3) == 0`
    pub fn root_of(self, v: V) -> V = todo

    /// The ancestor `k` edges above `v` (`v` for `k == 0`), or `None` if that is above the root.
    ///
    /// cost: O(log n)
    /// traps: when `k` is negative
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.ancestor(2, 2) == Some(0) and j.ancestor(2, 3) == None`
    pub fn ancestor(self, v: V, k: Int) -> ?V = todo

    /// The lowest common ancestor of `a` and `b`, or `None` in different trees.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.lca(2, 3) == Some(1) and j.lca(2, 0) == Some(0)`
    /// example: `domain V = 0..3; var j = JumpTree<V>(); j.add(1, 0); j.lca(1, 2) == None`
    pub fn lca(self, a: V, b: V) -> ?V = todo

    /// The highest ancestor `u` of `v` such that `pred(u)` holds, or `None` when `pred(v)` is false.
    /// `pred` must be true and then false on the way up.
    ///
    /// cost: O(log n) calls of `pred`
    /// traps: never
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.jump_while(2, v => v != 0) == Some(1) and j.jump_while(2, v => v == 0) == None`
    pub fn jump_while(self, v: V, pred: V -> Bool) -> ?V = todo

    /// The number of edges between `a` and `b`; `None` across trees.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.dist(2, 3) == Some(2) and j.dist(0, 0) == Some(0)`
    pub fn dist(self, a: V, b: V) -> ?Int = todo

    /// The `k`-th vertex on the path from `a` to `b` (`a` is the 0th); `None` past `b` or across
    /// trees.
    ///
    /// cost: O(log n)
    /// traps: when `k` is negative
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.jump(2, 3, 1) == Some(1) and j.jump(2, 3, 2) == Some(3) and j.jump(2, 3, 3) == None`
    pub fn jump(self, a: V, b: V, k: Int) -> ?V = todo

/// Binary lifting over a fixed forest given by parent links, each carrying a value on the edge
/// up: O(n log n) build and memory, O(log n) queries. A forest that grows leaf by leaf is a
/// `JumpTree`; one that changes is a `LinkCut`.
pub struct Lift<V, T>(op: (T, T) -> T, identity: T):
    up: [V]?V
    dep: [V]0..=V.count

    /// Binary lifting over the forest `par`: `par[v]` is `v`'s parent with the value on the edge
    /// to it, `None` at a root. `op` must be associative with identity `identity`; it need not
    /// commute, since folds run from the lower end up.
    ///
    /// cost: O(n log n) time and memory
    /// traps: when the parent links have a cycle
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.fold(2, up_to: 0) == 7 and l.depth(3) == 2`
    pub fn new(par: [V]?(V, T), op: (T, T) -> T, identity: T) -> Lift<V, T> = todo

    /// Lifting over any partial successor function, such as "the next interval a greedy picks":
    /// `Lift.of(i => first_from(iv[i].r))`. `None` ends a chain; the links must form a forest.
    /// Edge values are `()`, so `climb` and `ancestor` are what it is for.
    ///
    /// cost: O(n log n) time and memory, n calls of `next`
    /// traps: when the links have a cycle
    /// example: `domain I = 0..4; let l = Lift<I, ()>.of(i => if i < 3: Some(I(i + 1)) else: None); l.ancestor(0, 3) == Some(3) and l.depth(0) == 3`
    pub fn of(next: V -> ?V) -> Lift<V, ()> = todo

    /// Climbs from `v` while the next vertex up satisfies `ok`, which must hold on a prefix of
    /// every root path: how many edges were climbed and where it stopped. Greedy "jump while the
    /// next one still fits" counts are `Lift.of(next).climb(v, ok).steps`.
    ///
    /// cost: O(log n) calls of `ok`
    /// traps: never
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.climb(2, u => u != 0) == (steps: 1, at: 1) and l.climb(2, u => false) == (steps: 0, at: 2)`
    pub fn climb(self, v: V, ok: V -> Bool) -> (steps: Int, at: V) = todo

    /// The parent of `v`, or `None` at a root.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.parent(2) == Some(1) and l.parent(0) == None`
    pub fn parent(self, v: V) -> ?V = todo

    /// The number of edges from `v` up to its root.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.depth(0) == 0 and l.depth(3) == 2`
    pub fn depth(self, v: V) -> Int = todo

    /// The ancestor `k` edges above `v` (`v` for `k == 0`), or `None` if that is above the root.
    ///
    /// cost: O(log n)
    /// traps: when `k` is negative
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.ancestor(3, 2) == Some(0) and l.ancestor(3, 3) == None`
    pub fn ancestor(self, v: V, k: Int) -> ?V = todo

    /// The lowest common ancestor of `a` and `b`, or `None` in different trees.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.lca(2, 3) == Some(1)`
    pub fn lca(self, a: V, b: V) -> ?V = todo

    /// The fold with `op` of the edge values from `v` up to its ancestor `u`, lowest first;
    /// `identity` when `u == v`.
    ///
    /// cost: O(log n) calls of `op`
    /// traps: when `u` is not an ancestor of `v`
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.fold(2, up_to: 1) == 2 and l.fold(2, up_to: 0) == 7 and l.fold(2, up_to: 2) == 0`
    pub fn fold(self, v: V, up_to u: V) -> T = todo

    /// The highest ancestor `u` of `v` with `ok(self.fold(v, up_to: u))`, or `v` itself; `ok` must
    /// turn false at most once on the way up, and holds for `identity`.
    ///
    /// cost: O(log n) calls of `ok` and `op`
    /// traps: never
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.climb_while(2, s => s <= 5) == 1 and l.climb_while(2, s => s <= 7) == 0`
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

    /// `V.count` single-vertex trees holding `a`, or the identity of `op` everywhere when `a` is
    /// absent.
    ///
    /// cost: O(n)
    /// empty: `a: None` fills every vertex with the identity
    /// traps: never
    /// example: `domain V = 0..3; var lc = LinkCut<V, Int>(+); lc.get(2) == 0 and not lc.connected(0, 1)`
    pub fn new<M>(op: M, a: ?[V]T = None) -> LinkCut<V, T> = todo

    fn top(self, x: V) -> Bool = todo

    fn turn(mut self, x: V) = todo

    fn push(mut self, x: V) = todo

    fn pull(mut self, x: V) = todo

    fn rotate(mut self, x: V) = todo

    fn splay(mut self, x: V) = todo

    // Makes root -> x the preferred path with x at the root of its splay tree; returns the
    // last vertex where the walk joined the old preferred path (the lca with the previous access).
    fn access(mut self, x: V) -> V = todo

    /// Makes `v` the root of its tree.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.make_root(0); lc.root_of(2) == 0`
    pub fn make_root(mut self, v: V) = todo

    /// The root of the tree holding `v`: the last vertex made root there, by `make_root`, `link`,
    /// `lca` or `fold_path`.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.root_of(0) == 2 and lc.root_of(3) == 3`
    pub fn root_of(mut self, v: V) -> V = todo

    /// Whether `u` and `v` are in the same tree.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.connected(0, 2) and not lc.connected(0, 3)`
    pub fn connected(mut self, u: V, v: V) -> Bool = todo

    /// Adds the edge `u`-`v`: `u` becomes the root of its tree, which hangs below `v`, so the
    /// joined tree keeps `v`'s root.
    ///
    /// cost: amortized O(log n), with one `root_of` to check the trees differ
    /// traps: when `u` and `v` are already in one tree
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.link(3, 0); lc.connected(3, 2) and lc.root_of(3) == 2`
    pub fn link(mut self, u: V, v: V) = todo

    /// Removes the edge `u`-`v`.
    ///
    /// cost: amortized O(log n)
    /// traps: when there is no edge `u`-`v`
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.cut(1, 2); not lc.connected(0, 2) and lc.connected(0, 1)`
    pub fn cut(mut self, u: V, v: V) = todo

    /// The lowest common ancestor of `u` and `v` when their tree is rooted at `root`, which
    /// becomes its root; `None` unless all three are connected.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.lca(0, 2, 1) == Some(1) and lc.root_of(0) == 1 and lc.lca(0, 3, 1) == None`
    pub fn lca(mut self, u: V, v: V, root: V) -> ?V = todo

    /// The value at `v`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.get(2) == 3`
    pub fn get(self, v: V) -> T = todo

    /// Sets the value at `v` to `x`.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.set(2, 10); lc.get(2) == 10 and lc.fold_path(0, 2) == 13`
    pub fn set(mut self, v: V, x: T) = todo

    /// The fold of the values on the path from `u` to `v`, in that order; `u` becomes the root.
    ///
    /// cost: amortized O(log n)
    /// traps: when `u` and `v` are in different trees
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.fold_path(0, 2) == 6 and lc.root_of(2) == 0 and lc.fold_path(1, 1) == 2`
    pub fn fold_path(mut self, u: V, v: V) -> T = todo

    /// The number of edges between `a` and `b`; `None` across trees.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.dist(0, 2) == Some(2) and lc.dist(0, 3) == None`
    pub fn dist(mut self, a: V, b: V) -> ?Int = todo

    /// The `k`-th vertex on the path from `a` to `b` (`a` is the 0th); `None` past `b` or across
    /// trees.
    ///
    /// cost: amortized O(log n)
    /// traps: when `k` is negative
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.jump(0, 2, 1) == Some(1) and lc.jump(0, 2, 0) == Some(0) and lc.jump(0, 2, 3) == None`
    pub fn jump(mut self, a: V, b: V, k: Int) -> ?V = todo

/// `LinkCut` with updates along a path: `LazyLinkCut.new(+, add, a)` adds to every value on
/// a path and folds paths, under the same `Action` as `SegTree(op, a, act:)`. Reversing a path must not
/// change how a tag acts, `apply(f, x.rev(), k) == apply(f, x, k).rev()`, which holds for every
/// commutative `over`. Every operation is amortized O(log n). Memory is about twice `LinkCut`'s.
/// The preferred-path half of a link-cut tree over a fixed rooted tree: every node starts as its
/// own path. Counting "how many paths lie between v and the root" (colours painted along root
/// paths, access counts) reads off the switches each `access` makes.
pub struct PreferredPaths<V>:
    /// The preferred paths of `t` with every vertex its own path.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); var pp = PreferredPaths(t); pp.access(0).count() == 0`
    pub fn new<P>(t: Rooted<V, P>) -> PreferredPaths<V> = todo

    /// Makes the path from the root to `v` one preferred path, yielding each switch bottom-up:
    /// `gained` is the top vertex of the path just joined to its parent, `lost` the top vertex of
    /// the path that parent stopped preferring, if any. Counting "how many paths lie between `v`
    /// and the root" reads off the switches.
    ///
    /// cost: amortized O(log n) switches
    /// empty: nothing when `v` is already on the root's preferred path
    /// order: bottom-up
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); var pp = PreferredPaths(t); pp.access(3).collect() == [(lost: None, gained: 3), (lost: None, gained: 1)] and pp.access(4).collect() == [(lost: Some(3), gained: 4)]`
    pub fn access(mut self, v: V) -> yields (lost: ?V, gained: V) = todo

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

    /// `V.count` single-vertex trees holding `a`, or the identity of `act.over` everywhere when `a`
    /// is absent.
    ///
    /// cost: O(n)
    /// empty: `a: None` fills every vertex with the identity
    /// traps: never
    /// example: `domain V = 0..3; var lc = LazyLinkCut<V, Int, Int>(Action<Int, Int>(apply: (d, x, n) => x + d * n)); lc.get(1) == 0`
    pub fn new(act: Action<T, F>, a: ?[V]T = None) -> LazyLinkCut<V, T, F> = todo

    fn top(self, x: V) -> Bool = todo

    fn turn(mut self, x: V) = todo

    fn put(mut self, x: V, f: F) = todo

    fn push(mut self, x: V) = todo

    fn pull(mut self, x: V) = todo

    fn rotate(mut self, x: V) = todo

    fn splay(mut self, x: V) = todo

    fn access(mut self, x: V) -> V = todo

    /// Makes `v` the root of its tree.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.make_root(0); lc.root_of(2) == 0`
    pub fn make_root(mut self, v: V) = todo

    /// The root of the tree holding `v`, as for `LinkCut.root_of`.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.root_of(0) == 2`
    pub fn root_of(mut self, v: V) -> V = todo

    /// Whether `u` and `v` are in the same tree.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.connected(0, 2)`
    pub fn connected(mut self, u: V, v: V) -> Bool = todo

    /// Adds the edge `u`-`v` as `LinkCut.link` does: `u`'s tree hangs below `v`.
    ///
    /// cost: amortized O(log n), with one `root_of` to check the trees differ
    /// traps: when `u` and `v` are already in one tree
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.connected(0, 2) and lc.dist(0, 2) == Some(2)`
    pub fn link(mut self, u: V, v: V) = todo

    /// Removes the edge `u`-`v`.
    ///
    /// cost: amortized O(log n)
    /// traps: when there is no edge `u`-`v`
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.cut(0, 1); not lc.connected(0, 2) and lc.connected(1, 2)`
    pub fn cut(mut self, u: V, v: V) = todo

    /// The lowest common ancestor of `u` and `v` when their tree is rooted at `root`, which
    /// becomes its root; `None` unless all three are connected.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.lca(0, 2, 1) == Some(1)`
    pub fn lca(mut self, u: V, v: V, root: V) -> ?V = todo

    /// The value at `v`, with every update along paths through it applied.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.apply_path(0, 1, 10); lc.get(1) == 12 and lc.get(2) == 3`
    pub fn get(mut self, v: V) -> T = todo

    /// Sets the value at `v` to `x`, replacing every earlier update there.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.set(1, 7); lc.fold_path(0, 2) == 11`
    pub fn set(mut self, v: V, x: T) = todo

    /// The fold of the values on the path from `u` to `v`, in that order; `u` becomes the root.
    ///
    /// cost: amortized O(log n)
    /// traps: when `u` and `v` are in different trees
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.fold_path(0, 2) == 6`
    pub fn fold_path(mut self, u: V, v: V) -> T = todo

    /// Applies `f` to every value on the path from `u` to `v`; `u` becomes the root.
    ///
    /// cost: amortized O(log n)
    /// traps: when `u` and `v` are in different trees
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.apply_path(0, 1, 10); lc.fold_path(0, 2) == 26`
    pub fn apply_path(mut self, u: V, v: V, f: F) = todo

    /// The number of edges between `a` and `b`; `None` across trees.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.dist(0, 2) == Some(2) and lc.dist(1, 1) == Some(0)`
    pub fn dist(mut self, a: V, b: V) -> ?Int = todo

    /// The `k`-th vertex on the path from `a` to `b` (`a` is the 0th); `None` past `b` or across
    /// trees.
    ///
    /// cost: amortized O(log n)
    /// traps: when `k` is negative
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.jump(2, 0, 1) == Some(1) and lc.jump(2, 0, 3) == None`
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

    /// The binary permutation tree of `p`, built by one stack pass from left to right: a run that
    /// keeps extending in one direction builds left-deep, so `[0, 1, 2]` is `((0, 1), 2)`.
    ///
    /// cost: O(n)
    /// empty: `I` must be nonempty
    /// traps: when `I` is empty, `I.count != V.count`, or `p` is not a bijection
    /// example: `domain I = 0..3; let p: [I]I = [1, 0, 2]; let pt = PermTree(p); pt.kind[pt.root] == PermKind.Incr and pt.kind[pt.left(pt.root)] == PermKind.Decr`
    pub fn new(p: [I]V) -> PermTree<I, V, _> = todo

    /// The view of `v` to match on (decision 132): `Leaf(i)` or `Inner(l, r, kind)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain I = 0..3; let p: [I]I = [1, 0, 2]; let pt = PermTree(p); pt.node(pt.leaf(1)) == PermNode.Leaf(1) and pt.node(pt.root) == PermNode.Inner(pt.left(pt.root), pt.leaf(2), PermKind.Incr)`
    pub fn node(self, v: Node) -> PermNode<I, Node> = todo

    /// The leaf of position `i`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain I = 0..3; let p: [I]I = [1, 0, 2]; let pt = PermTree(p); pt.span(pt.leaf(2)) == 2..3 and pt.kind[pt.leaf(2)] == PermKind.Leaf`
    pub fn leaf(self, i: I) -> Node = todo

    /// The earlier child of the inner node `v`.
    ///
    /// cost: O(1)
    /// traps: when `v` is a leaf
    /// example: `domain I = 0..3; let p: [I]I = [1, 0, 2]; let pt = PermTree(p); pt.span(pt.left(pt.root)) == 0..2`
    pub fn left(self, v: Node) -> Node = todo

    /// The later child of the inner node `v`.
    ///
    /// cost: O(1)
    /// traps: when `v` is a leaf
    /// example: `domain I = 0..3; let p: [I]I = [1, 0, 2]; let pt = PermTree(p); pt.right(pt.root) == pt.leaf(2)`
    pub fn right(self, v: Node) -> Node = todo

    /// The positions `v` covers, a nonempty range of `I`.
    ///
    /// cost: O(1)
    /// empty: never empty
    /// order: increasing positions
    /// traps: never
    /// example: `domain I = 0..3; let p: [I]I = [1, 0, 2]; let pt = PermTree(p); pt.span(pt.root) == 0..3`
    pub fn span(self, v: Node) -> Range<I> = todo

    /// From the least to the greatest value `v` covers: exactly its values unless `v` is `Partial`.
    ///
    /// cost: O(1)
    /// empty: never empty
    /// order: increasing values
    /// traps: never
    /// example: `domain I = 0..3; let p: [I]I = [1, 0, 2]; let pt = PermTree(p); pt.vals(pt.left(pt.root)) == 0..2 and pt.vals(pt.leaf(0)) == 1..2`
    pub fn vals(self, v: Node) -> Range<V> = todo

@test
fn perm_tree_nodes_cover_intervals_of_values() = todo

@test
fn perm_tree_of_a_small_permutation() = todo

/// The vertices of a `Rooted` tree by depth, each depth in preorder, over their own domain
/// `B`: the descendants of a vertex at one depth are a contiguous range of `B`.
pub struct Layers<V, B>:
    at: [B]V

    /// The position of `v` among the layers.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ly = t.layers(); ly.at[ly.pos(2)] == 2`
    pub fn pos(self, v: V) -> B = todo

    /// Every vertex at depth `d`, in `t.Pre` order; empty past the deepest or below 0.
    ///
    /// cost: O(1)
    /// empty: for `d` outside `0..=` the greatest depth
    /// order: `t.Pre` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ly = t.layers(); ly.level(2).map(b => ly.at[b]).collect() == [3, 4] and ly.level(3).count() == 0`
    pub fn level(self, d: Int) -> Range<B> = todo

    /// The descendants of `v` at depth `d`, counted from the root as in `level`, by binary search
    /// in `level(d)`; `v` itself when `d == depth(v)`.
    ///
    /// cost: O(log n)
    /// empty: when there are none, including `d < depth(v)`
    /// order: `t.Pre` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ly = t.layers(); ly.below(1, 2).map(b => ly.at[b]).collect() == [3, 4] and ly.below(2, 2).count() == 0`
    pub fn below(self, v: V, d: Int) -> Range<B> = todo

/// The enter and exit events of a `Rooted` tree's `tour` over their own domain `P` of 2n
/// members, for Mo's algorithm on trees.
pub struct Events<V, P>:
    at: [P]Visit<V>

    /// The event that enters `v`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ev = t.events(); ev.at[ev.enter(1)] == Enter(1) and ev.enter(1) < ev.enter(3)`
    pub fn enter(self, v: V) -> P = todo

    /// The event that leaves `v`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ev = t.events(); ev.at[ev.exit(3)] == Exit(3) and ev.exit(3) < ev.exit(1)`
    pub fn exit(self, v: V) -> P = todo

    /// The events inside `v`'s subtree, `enter(v)` through `exit(v)`.
    ///
    /// cost: O(1)
    /// empty: never empty; 2 * size(v) events
    /// order: `tour` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ev = t.events(); ev.sub(1).map(p => ev.at[p]).collect() == [Enter(1), Enter(3), Exit(3), Enter(4), Exit(4), Exit(1)]`
    pub fn sub(self, v: V) -> Range<P> = todo

    /// A range of events whose vertices met an odd number of times, plus `extra` when present, are
    /// exactly the vertices of the path from `a` to `b`.
    ///
    /// With `a` the end entered first: when `a` is an ancestor of `b`, the range is `enter(a)`
    /// through `enter(b)` and `extra` is `None`; otherwise it is `exit(a)` through `enter(b)` and
    /// `extra` is their lowest common ancestor.
    ///
    /// cost: O(log n)
    /// empty: never empty
    /// order: `tour` order
    /// traps: when `a` and `b` are in different trees
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ev = t.events(); ev.path(3, 2).range.count() == 5 and ev.path(3, 2).extra == Some(0) and ev.path(1, 4).extra == None`
    pub fn path(self, a: V, b: V) -> (range: Range<P>, extra: ?V) = todo

/// The long-path decomposition of a `Rooted` tree: each vertex continues the path of its
/// tallest child, so a DP indexed by depth below `v` lives in one shared buffer over `L`,
/// inheriting `deep[v]`'s values is free, and merging the other children costs their heights.
pub struct LongPaths<V, L>:
    /// The child with the tallest subtree; `None` at a leaf.
    deep: [V]?V
    /// The number of edges on the longest path down from `v`.
    height: [V]Int

    /// The slots of `v`, one per depth offset `0..=height[v]`, offset 0 first; `slots(deep[v])`
    /// starts one later in the same buffer.
    ///
    /// cost: O(1)
    /// empty: never empty
    /// order: offset 0 first
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let lp = t.long_paths(); lp.slots(0).count() == 3 and lp.slots(2).count() == 1`
    pub fn slots(self, v: V) -> Range<L> = todo

    /// The slot of `v` at depth offset `d`.
    ///
    /// cost: O(1)
    /// traps: unless `0 <= d <= height[v]`
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let lp = t.long_paths(); lp.slot(1, 1) == lp.slot(3, 0) and lp.slot(0, 1) == lp.slot(1, 0)`
    pub fn slot(self, v: V, d: Int) -> L = todo

/// Lowest common ancestors in O(1) after O(n log n) preprocessing, by a sparse table over the
/// Euler tour: for many queries on a fixed tree when `Rooted.lca` is the bottleneck.
pub struct LcaTable<V>:
    first: [V]Int
    far: [V]Int

    /// A sparse table over the Euler walk of `t`, for O(1) lowest common ancestors on a fixed tree.
    ///
    /// cost: O(n log n) time and memory
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let lt = LcaTable(t); lt.lca(3, 4) == 1`
    pub fn new<Pre>(t: Rooted<V, Pre>) -> LcaTable<V> = todo

    /// The lowest common ancestor of `a` and `b`.
    ///
    /// cost: O(1)
    /// traps: when `a` and `b` are in different trees
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let lt = LcaTable(t); lt.lca(3, 2) == 0 and lt.lca(4, 4) == 4`
    pub fn lca(self, a: V, b: V) -> V = todo

    /// The distance between `a` and `b`, weighted as the tree was built.
    ///
    /// cost: O(1)
    /// traps: when `a` and `b` are in different trees
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let lt = LcaTable(t); lt.dist(3, 2) == 3`
    pub fn dist(self, a: V, b: V) -> Int = todo

/// A forest that changes, with folds over whole trees and subtrees, by an Euler tour tree
/// (a balanced sequence of each tree's tour). Where `LinkCut` folds paths, this folds
/// components. Values fold with `op`, which must commute. Every operation is expected
/// O(log n).
pub struct EulerTourTree<V, T>:
    val: [V]T

    /// Lone vertices with values `a`, or the identity of `op` everywhere when `a` is `None`.
    ///
    /// cost: O(n)
    /// empty: `a: None` fills every vertex with the identity
    /// traps: never
    /// example: `domain V = 0..3; var et = EulerTourTree<V, Int>(+); et.get(1) == 0 and et.len(1) == 1`
    pub fn new<M>(op: M, a: ?[V]T = None) -> EulerTourTree<V, T> = todo

    /// Joins the trees of `u` and `v` by the edge `u`-`v`.
    ///
    /// cost: expected O(log n)
    /// traps: when `u` and `v` are already in one tree
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.connected(0, 2) and et.len(2) == 3`
    pub fn link(mut self, u: V, v: V) = todo

    /// Removes the edge `u`-`v`.
    ///
    /// cost: expected O(log n)
    /// traps: when there is no edge `u`-`v`
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.cut(1, 2); not et.connected(0, 2) and et.len(0) == 2`
    pub fn cut(mut self, u: V, v: V) = todo

    /// Whether `a` and `b` are in one tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.connected(0, 2) and not et.connected(0, 3)`
    pub fn connected(mut self, a: V, b: V) -> Bool = todo

    /// The number of vertices in `v`'s tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.len(0) == 3 and et.len(3) == 1`
    pub fn len(mut self, v: V) -> Int = todo

    /// The value at `v`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.get(3) == 4`
    pub fn get(self, v: V) -> T = todo

    /// Replaces the value at `v`.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.set(0, 10); et.fold_tree(2) == 15`
    pub fn set(mut self, v: V, x: T) = todo

    /// The fold of every value in `v`'s tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.fold_tree(0) == 6 and et.fold_tree(3) == 4`
    pub fn fold_tree(mut self, v: V) -> T = todo

    /// The fold over `v`'s side of the edge `v`-`away`: what `v`'s tree would hold were the edge
    /// cut.
    ///
    /// cost: expected O(log n)
    /// traps: when there is no edge `v`-`away`
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.fold_sub(1, 2) == 3 and et.fold_sub(2, 1) == 3`
    pub fn fold_sub(mut self, v: V, away: V) -> T = todo

/// `EulerTourTree` with updates to whole trees and subtrees under an action.
pub struct LazyEulerTourTree<V, T, F>:
    val: [V]T

    /// Lone vertices with values `a`, or the identity of `act.over` everywhere when `a` is `None`.
    ///
    /// cost: O(n)
    /// empty: `a: None` fills every vertex with the identity
    /// traps: never
    /// example: `domain V = 0..3; var et = LazyEulerTourTree<V, Int, Int>(Action<Int, Int>(apply: (d, x, n) => x + d * n)); et.get(1) == 0`
    pub fn new(act: Action<T, F>, a: ?[V]T = None) -> LazyEulerTourTree<V, T, F> = todo

    /// Joins the trees of `u` and `v` by the edge `u`-`v`.
    ///
    /// cost: expected O(log n)
    /// traps: when `u` and `v` are already in one tree
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.connected(0, 2) and et.len(2) == 3`
    pub fn link(mut self, u: V, v: V) = todo

    /// Removes the edge `u`-`v`.
    ///
    /// cost: expected O(log n)
    /// traps: when there is no edge `u`-`v`
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.cut(1, 2); not et.connected(0, 2)`
    pub fn cut(mut self, u: V, v: V) = todo

    /// Whether `a` and `b` are in one tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); not et.connected(0, 3)`
    pub fn connected(mut self, a: V, b: V) -> Bool = todo

    /// The number of vertices in `v`'s tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.len(1) == 3`
    pub fn len(mut self, v: V) -> Int = todo

    /// The value at `v`, with every update to a tree or side holding it applied.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.apply_tree(0, 10); et.get(2) == 13 and et.get(3) == 4`
    pub fn get(mut self, v: V) -> T = todo

    /// Replaces the value at `v`, discarding earlier updates there.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.set(0, 10); et.fold_tree(2) == 15`
    pub fn set(mut self, v: V, x: T) = todo

    /// The fold of every value in `v`'s tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.fold_tree(0) == 6`
    pub fn fold_tree(mut self, v: V) -> T = todo

    /// The fold over `v`'s side of the edge `v`-`away`.
    ///
    /// cost: expected O(log n)
    /// traps: when there is no edge `v`-`away`
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.fold_sub(1, 2) == 3`
    pub fn fold_sub(mut self, v: V, away: V) -> T = todo

    /// Applies `f` to every value in `v`'s tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.apply_tree(0, 10); et.fold_tree(2) == 36 and et.get(3) == 4`
    pub fn apply_tree(mut self, v: V, f: F) = todo

    /// Applies `f` to every value on `v`'s side of the edge `v`-`away`.
    ///
    /// cost: expected O(log n)
    /// traps: when there is no edge `v`-`away`
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.apply_sub(1, 2, 10); et.get(0) == 11 and et.get(2) == 3 and et.fold_tree(0) == 26`
    pub fn apply_sub(mut self, v: V, away: V, f: F) = todo

/// The Cartesian tree of `a` as parent links: the root is the least element by `by` (the
/// leftmost among equals) and every subtree is a contiguous range of `D`. Root it with
/// `Rooted.from_parents`.
///
/// cost: O(n), one stack pass
/// empty: `[]` for an empty `a`
/// ties: the leftmost of equal elements is higher
/// order: keyed by `D`
/// traps: never
/// example: `cartesian_tree([3, 1, 2]) == [Some(1), None, Some(1)] and cartesian_tree([1, 1]) == [None, Some(0)]`
/// example: `cartesian_tree([3, 1, 2], by: (x, y) => x > y) == [None, Some(2), Some(0)]`
pub fn cartesian_tree<D, T>(a: [D]T, by: (T, T) -> Bool = (x, y) => x < y) -> [D]?D = todo

/// The Kruskal reconstruction tree: each edge that merges two components, in increasing
/// weight, becomes a node above them. The leaves under a node are the vertices connected by
/// edges of weight at most its own, so "what `v` reaches under a limit" is one subtree.
pub struct KruskalTree<V, N, W>:
    /// The leaf node of each vertex.
    leaf: [V]N
    /// The merging weight of each node; `None` at a leaf.
    w: [N]?W

    /// The nodes as a `Rooted` forest over `N`: subtree ranges, LCA and folds. The roots, one per
    /// component, are in increasing node order.
    ///
    /// cost: O(1); built with the tree
    /// traps: never
    /// example: `domain V = 0..3; let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3)]; let (domain N, kt) = kruskal_tree(es, w: e => es[e].w); kt.tree.parent(kt.leaf[1]) == kt.tree.parent(kt.leaf[2]) and kt.tree.size(kt.leaf[0]) == 1`
    pub @get fn tree(self) -> Rooted<N, _> = todo

    /// The highest node above `leaf[v]` whose weight is at most `limit` (`leaf[v]` itself when
    /// none is); its leaves are what `v` reaches by edges of weight at most `limit`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3)]; let (domain N, kt) = kruskal_tree(es, w: e => es[e].w); kt.reach(0, 4) == kt.leaf[0] and kt.w[kt.reach(1, 4)] == Some(3) and kt.w[kt.reach(1, 5)] == Some(5)`
    pub fn reach(self, v: V, limit: W) -> N = todo

    /// The highest node above `leaf[v]` whose merging edge, and every one below it, satisfies `ok`,
    /// which gets the edge's key in the list given to `kruskal_tree`
    /// (`kt.top(v, ok: e => roads[e].alt > p)`). `ok` must hold on a prefix of the merge order.
    ///
    /// cost: O(log n) calls of `ok`
    /// ties: none; the highest such node is unique
    /// traps: never
    /// example: `domain V = 0..3; let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3)]; let (domain N, kt) = kruskal_tree(es, w: e => es[e].w); kt.w[kt.top(1, e => es[e].w < 5)] == Some(3) and kt.top(0, e => false) == kt.leaf[0]`
    pub fn top<E>(self, v: V, ok: E -> Bool) -> N = todo

    /// For every node, `op` folded over the `leaf` values of the leaves under it, in vertex order:
    /// the least walk home from any vertex a subtree reaches is `kt.up(min, leaf: v => home[v]!)`.
    ///
    /// cost: O(n) calls of `op` and `leaf`
    /// empty: never empty; every leaf has a value
    /// order: keyed by `N`
    /// traps: never
    /// example: `domain V = 0..3; let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3)]; let (domain N, kt) = kruskal_tree(es, w: e => es[e].w); let c = kt.up(add, leaf: v => 1); c[kt.reach(2, 3)] == 2 and c[kt.leaf[0]] == 1`
    pub fn up<T, M>(self, op: M, leaf: V -> T) -> [N]T = todo

    /// The least possible largest edge weight on a path from `a` to `b`: the weight of their
    /// lowest common ancestor. `None` when they are not connected, and at `a == b`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3)]; let (domain N, kt) = kruskal_tree(es, w: e => es[e].w); kt.bottleneck(0, 2) == Some(5) and kt.bottleneck(1, 2) == Some(3) and kt.bottleneck(0, 0) == None`
    pub fn bottleneck(self, a: V, b: V) -> ?W = todo

/// The Kruskal reconstruction tree of `edges` weighted by `w`, over the vertices `V`, taking the
/// same arguments as `std.graph.kruskal`; its nodes are a fresh domain bound by the caller:
/// `let (domain N, kt) = kruskal_tree(es, w: f)`.
///
/// `N` holds one leaf per vertex, in vertex order, then one node per merging edge, in merge
/// order. Edges whose ends are already joined (loops included) add nothing. Merging by the
/// largest weight first is `w: e => desc(e.alt)`, and "edges above `p`" is then
/// `kt.reach(v, desc(p + 1))`: any `Ord` weight works, so no weight is ever negated.
///
/// cost: O(m log m) for m edges
/// empty: no edges gives `V.count` lone leaves
/// ties: equal weights merge in edge key order, as `kruskal`
/// traps: never
/// example: `domain V = 0..3; let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3)]; let (domain N, kt) = kruskal_tree(es, w: e => es[e].w); N.count == 5 and kt.bottleneck(0, 2) == Some(5)`
pub fn kruskal_tree<V, E, W>(edges: [E](u: V, v: V, ..), w: E -> W) -> (domain N, KruskalTree<V, N, W>) = todo

/// The Prüfer code of a tree on `V`: repeatedly remove the least leaf and record its neighbour,
/// until two vertices remain. `V.count - 2` vertices, in bijection with the labelled trees.
///
/// cost: O(n)
/// empty: `[]` when `V` has two vertices
/// order: removal order
/// traps: when `V` has fewer than two vertices, or `edges` is not a tree
/// example: `domain V = 0..4; let tr: Tree<V> = [(u: 0, v: 1), (u: 1, v: 2), (u: 1, v: 3)]; prufer(tr) == [1, 1]`
/// example: `domain V = 0..4; let tr: Tree<V> = [(u: 0, v: 1), (u: 1, v: 2), (u: 2, v: 3)]; prufer(tr) == [1, 2]`
pub fn prufer<V>(edges: Tree<V>) -> List<V> = todo

/// The tree whose Prüfer code is `code`. Edges come in removal order as `(u: leaf, v:
/// neighbour)`, the last joining the two vertices left, smaller first.
///
/// cost: O(n)
/// empty: `[]` with two vertices gives the one edge `(u: 0, v: 1)` in `V` order
/// traps: when `code` does not have `V.count - 2` entries, or `V` has fewer than two vertices
/// example: `domain V = 0..4; from_prufer<V>([1, 1]) == [(u: 0, v: 1), (u: 2, v: 1), (u: 1, v: 3)]`
pub fn from_prufer<V>(code: List<V>) -> Tree<V> = todo
