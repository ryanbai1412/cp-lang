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
use std.algebra.{Monoid, MaxSub, Action}
use std.algebra.{Reversible}
use std.algebra.{add}
use std.dp.{Step}
use std.dsu.{Dsu}
use std.graph.{Next, Target, Graph, Visit, Edged, arc_len}
use std.rand.{Rng}
use std.segtree.{SegTree}
use std.seq.{dedup, first_true, get_or_insert, span}

/// A BFS spanning tree rooted at `V.first`, as a `Rooted`. Each vertex's parent is the
/// vertex whose arc first reached it, scanning vertices in BFS order and arcs in input
/// order. A forest is `spanning_forest`.
///
/// cost: O(n + m)
/// traps: when the graph is not connected from `V.first`, or `V` is empty
/// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]; let t = Graph.undirected(es).spanning_tree(); t.parent(2) == Some(1) and t.parent(0) == None`
pub fn spanning_tree<V, A>(self: Graph<V, A>) -> Rooted<V, _> = Rooted.new(self, V.first!)

// blocked: `_` inside a tuple return type is E0005 (compiler worklog)
/// The DFS tree from `root` of the component holding it, as a `Rooted` tree and a mask of the
/// edges it uses. The search takes arcs in input order and descends at once (recursive order,
/// run without recursion). On an undirected graph every edge with `tree_edge[e] == false`
/// then joins a vertex to one of its ancestors in `tree`, which is what bridges, cut vertices
/// and back-edge DPs need. `tree` is a forest over all of `V`: each vertex outside the
/// component is a root of its own.
///
/// cost: O(n + m)
/// order: children in the order the search reached them
/// traps: never
/// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2), (u: 2, v: 0)]; let (tree, tree_edge) = Graph.undirected(es).dfs_tree(0); tree.parent(2) == Some(1) and tree.depth(2) == 2 and tree_edge == [true, true, false]`
pub fn dfs_tree<V, A, E>(self: Graph<V, A>, root: V) -> (
    tree: Rooted<V,
    _>,
    tree_edge: [E]Bool,
) where A: Edged<E> = todo

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

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
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
    pub fn new(next: Next<V>, root: V) -> Rooted<V, _> = Rooted.build(next, [root])

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
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
    pub fn from_parents(par: [V]?V) -> Rooted<V, _> = Rooted.try_from_parents(par)!

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// `from_parents` for parents that may not form a forest: `None` when they have a cycle.
    ///
    ///     let t = Rooted.try_from_parents(up) else: print("NO"); return
    ///
    /// cost: O(n)
    /// empty: `None` when `par` has no root
    /// traps: never
    /// example: `domain V = 0..2; let par: [V]?V = [Some(1), Some(0)]; Rooted.try_from_parents(par) == None`
    pub fn try_from_parents(par: [V]?V) -> ?Rooted<V, _>:
        var kids: [V]List<V> = []
        var roots: List<V> = []
        for v in V:
            if let p = par[v]: kids[p].push(v)
            else: roots.push(v)
        var queue = roots.copy()
        var i = 0
        while i < queue.len:
            for c in kids[queue[i]]: queue.push(c)
            i += 1
        if roots.len > 0 and queue.len == V.count: Some(Rooted.build(kids, roots)) else: None

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
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
    pub fn forest(next: Next<V>, from: List<V>) -> Rooted<V, _> = Rooted.build(next, from)

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    // Breadth-first from each root in turn, then sizes, heavy children and the heavy-first
    // preorder. Traps when a root is reached twice or some vertex is never reached.
    fn build(next: Next<V>, roots: List<V>) -> Rooted<V, _>:
        var seen: [V]Bool = false
        var par: [V]?V = None
        var dep: [V]Int = 0
        var far: [V]Int = 0
        var bfs: List<V> = []
        var k0: [V]Int = 0
        var k1: [V]Int = 0
        var i = 0
        for r in roots:
            assert not seen[r]
            seen[r] = true
            bfs.push(r)
            while i < bfs.len:
                let u = bfs[i]
                i += 1
                k0[u] = bfs.len
                for a in next.out(u):
                    let v = a.to
                    if not seen[v]:
                        seen[v] = true
                        par[v] = Some(u)
                        dep[v] = dep[u] + 1
                        far[v] = far[u] + arc_len(a)
                        bfs.push(v)
                k1[u] = bfs.len
        assert roots.len > 0 and bfs.len == V.count
        var sz: [V]Int = 1
        for j in (0..bfs.len).rev:
            if let p = par[bfs[j]]: sz[p] += sz[bfs[j]]
        var heavy: [V]?V = None
        for u in V:
            var best = 0
            for j in k0[u]..k1[u]:
                if sz[bfs[j]] > best:
                    best = sz[bfs[j]]
                    heavy[u] = Some(bfs[j])
        var top: [V]V = [V] v => v
        var tin: [V]Int = 0
        var order: List<V> = []
        var stack: List<V> = []
        for r in roots:
            stack.push(r)
            while stack.len > 0:
                let v = stack[stack.len - 1]
                stack.pop()
                tin[v] = order.len
                order.push(v)
                if let p = par[v]:
                    if heavy[p] == Some(v): top[v] = top[p]
                for j in (k0[v]..k1[v]).rev:
                    if heavy[v] != Some(bfs[j]): stack.push(bfs[j])
                if let h = heavy[v]: stack.push(h)
        domain Order = 0..V.count
        Rooted::<V, Order>(
            roots[0],
            par,
            [V] v => dep[v],
            far,
            [V] v => sz[v],
            bfs,
            [V] v => k0[v],
            [V] v => k1[v],
            heavy,
            top,
            [V] v => Order.nth(tin[v]),
            [Order] p => order[p.to_idx],
        )

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The roots in the order given: one for `new`, one per tree for `forest`, key order for
    /// `from_parents`.
    ///
    /// cost: O(1) to make, O(1) per root
    /// order: as given at construction
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.roots.collect() == [0]`
    @get pub fn roots(self) -> yields V:
        var p = 0
        while p < V.count:
            yield at[Pre.nth(p)]
            p += sz[at[Pre.nth(p)]]

    /// Every vertex, parents before children: breadth-first from the roots, each vertex's
    /// children in child order. `t.order.rev` lists children before parents, the order a tree DP
    /// without recursion walks.
    ///
    /// cost: O(1); the list is the tree's own breadth-first order
    /// order: breadth-first; the roots first
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.order == [0, 1, 2, 3, 4]`
    @get pub fn order(self) -> List<V> = bfs

    /// The position of `v` in this tree's own preorder domain `t.Pre`; the inverse of `at`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.at[t.pos(3)] == 3 and t.pos(1) < t.pos(3) and t.pos(4) < t.pos(2)`
    pub fn pos(self, v: V) -> Pre = tin[v]

    /// The parent of `v`; `None` at a root.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.parent(3) == Some(1) and t.parent(0) == None`
    pub fn parent(self, v: V) -> ?V = par[v]

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
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
    pub fn reorder<T>(self, a: [V]T) -> [Pre]T = [Pre] p => a[at[p]]

    /// The children of `v`, in the order `next` listed them. Children are stored contiguously, so
    /// nothing is allocated.
    ///
    /// cost: O(1) to start, O(1) per child
    /// empty: nothing at a leaf
    /// order: as `next` listed them
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.children(1).collect() == [3, 4] and t.children(2).count() == 0`
    pub fn children(self, v: V) -> yields V:
        for i in k0[v]..k1[v]: yield bfs[i]

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The arcs from `v` down to its children, with every field `next` gave them
    /// (`for a in t.child_arcs(v): up(a.to) + a.w`), so edge data needs no side array.
    ///
    /// cost: O(1) per child
    /// empty: nothing at a leaf
    /// order: child order
    /// traps: never
    /// example: `domain V = 0..3; let es: [0..2](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]; let t = Rooted(Graph.undirected(es), root: 0); t.child_arcs(1).map(a => a.to).collect() == [2]`
    pub fn child_arcs<A>(self, v: V) -> yields A = todo

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// For each child `c` of `v`, in child order, the fold with `op` of `f` over `v`'s other
    /// children: the step of a rerooting DP, with no division or inverse.
    ///
    /// The fold keeps child order (earlier siblings, then later ones), so `op` need not commute;
    /// with one child, `rest` is the identity. Prefix and suffix folds make it O(k) for `k`
    /// children.
    ///
    ///     for (c, rest) in t.excl_fold(v, *, c => down[c] + 1):
    ///         up[c] = (up[v] * rest + 1) % m
    ///
    /// cost: O(k) calls of `f` and of `op`, O(k) memory
    /// empty: nothing at a leaf
    /// order: child order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.excl_fold(0, +, c => t.size(c)).collect() == [(c: 1, rest: 1), (c: 2, rest: 3)]`
    pub fn excl_fold<T, M>(self, v: V, op: M, f: V -> T) -> yields (c: V, rest: T):
        let mon = Monoid<T>.of(op)
        let g = mon.op
        let k = k1[v] - k0[v]
        let vals = [f(bfs[i]) for i in k0[v]..k1[v]]
        var suf = List.filled(k + 1, mon.id)
        for i in (0..k).rev: suf[i] = g(vals[i], suf[i + 1])
        var pre = mon.id
        for i in 0..k:
            yield (c: bfs[k0[v] + i], rest: g(pre, suf[i + 1]))
            pre = g(pre, vals[i])

    /// The number of edges between `v` and its root.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.depth(0) == 0 and t.depth(4) == 2`
    pub fn depth(self, v: V) -> Int = dep[v]

    /// The number of vertices in the subtree of `v`, `v` included.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.size(0) == 5 and t.size(1) == 3 and t.size(2) == 1`
    pub fn size(self, v: V) -> Int = sz[v]

    /// The children of `v` other than its heavy child, in child order. Every vertex has O(log n)
    /// light edges above it, which is what small-to-large merging pays for.
    ///
    /// cost: O(children)
    /// empty: nothing at a leaf
    /// order: child order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.light(0).collect() == [2] and t.light(1).collect() == [4]`
    pub fn light(self, v: V) -> yields V:
        for i in k0[v]..k1[v]:
            if Some(bfs[i]) != heavy[v]: yield bfs[i]

    /// The root of the tree that holds `v`: always `t.root` unless built as a forest.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.root_of(4) == 0`
    pub fn root_of(self, v: V) -> V:
        var x = top[v]
        while let p = par[x]: x = top[p]
        x

    /// The vertices of `v`'s subtree in preorder, `v` first: `t.at[p] for p in t.sub(v)`.
    ///
    /// cost: O(size(v))
    /// empty: never empty; it holds `v`
    /// order: `t.Pre` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.subtree(1).collect() == [1, 3, 4] and t.subtree(0).collect() == [0, 1, 3, 4, 2]`
    pub fn subtree(self, v: V) -> yields V:
        for p in self.sub(v): yield at[p]

    /// A top-down pass: `out[v] = f(out[parent], v)`, and `f(init, r)` at a root `r`. The state on
    /// the path from the root, such as a prefix maximum or a distance.
    ///
    /// cost: O(n) calls of `f`
    /// empty: never empty
    /// order: keyed by `V`; `f` runs in `t.Pre` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.down(0, (d, v) => d + 1) == [1, 2, 2, 3, 3]`
    pub fn down<T>(self, init: T, f: (T, V) -> T) -> [V]T:
        var out = [V] v => init
        for p in Pre:
            let v = at[p]
            if let q = par[v]: out[v] = f(out[q], v)
            else: out[v] = f(init, v)
        out

    /// A bottom-up tree DP: `out[v] = finish(fold with op of lift(out[c], c, v) over children c, v)`.
    ///
    /// `lift` sees both ends of the edge, child first, and defaults to the child's value. Children
    /// fold in child order, so `op` need not commute; a leaf folds nothing and gets the identity.
    ///
    /// cost: O(n) calls of `lift`, `op` and `finish`
    /// empty: never empty
    /// order: keyed by `V`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.up(+, finish: (s, v) => s + 1) == [5, 3, 1, 1, 1]`
    pub fn up<T, U, M>(
        self,
        op: M,
        lift: (T, V, V) -> U = (x, c, p) => x,
        finish: (U, V) -> T,
    ) -> [V]T:
        let mon = Monoid<U>.of(op)
        let f = mon.op
        var dn: [V]?T = None
        for p in Pre.rev:
            let v = at[p]
            var acc = mon.id
            for i in k0[v]..k1[v]:
                let c = bfs[i]
                acc = f(acc, lift(dn[c]!, c, v))
            dn[v] = Some(finish(acc, v))
        [V] v => dn[v]!

    /// [`up`] for every root at once: `out[v] = finish(fold with op of lift(out[c], c, v) over
    /// neighbours c, v)`, where `out[c]` is `c`'s answer with the tree rooted at `v`.
    ///
    /// For a tree DP asked at every vertex, like the sum of distances from each vertex. A
    /// vertex's answer is built from its neighbours' answers, each seen from the far side of the
    /// edge: `lift(x, c, p)` turns neighbour `c`'s answer `x` into what it contributes to `p`
    /// (default: `x` itself), `op` (a [`Monoid`], or an operator such as `+`) combines the
    /// contributions, and `finish(s, v)` turns the combined value into `v`'s answer.
    ///
    /// ```cp
    /// domain V = 1..=5
    /// let adj: [V]List<V> = [[2, 3], [1, 4, 5], [1], [2], [2]]
    /// let t = Rooted(adj, root: 1)
    /// let sums = t.reroot(
    ///     Monoid<(size: Int, dist: Int)>.of(each(size: +, dist: +)),
    ///     lift: (x, _, _) => (size: x.size, dist: x.dist + x.size),
    ///     finish: (s, _) => (size: s.size + 1, dist: s.dist),
    /// )
    /// sums.map(s => s.dist) == [6, 5, 9, 8, 8]
    /// ```
    ///
    /// Each vertex folds its parent's side first, then its children in child order; the sibling
    /// folds use prefixes and suffixes, so `op` needs no inverse.
    ///
    /// cost: O(n) calls of `lift` and `finish`, O(n) calls of `op`
    /// empty: never empty
    /// order: keyed by `V`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.reroot(+, finish: (s, v) => s + 1) == [5, 5, 5, 5, 5]`
    pub fn reroot<T, U, M>(
        self,
        op: M,
        lift: (T, V, V) -> U = (x, c, p) => x,
        finish: (U, V) -> T,
    ) -> [V]T:
        let mon = Monoid<U>.of(op)
        let f = mon.op
        var dn: [V]?T = None
        for p in Pre.rev:
            let v = at[p]
            var acc = mon.id
            for i in k0[v]..k1[v]:
                let c = bfs[i]
                acc = f(acc, lift(dn[c]!, c, v))
            dn[v] = Some(finish(acc, v))
        var ab: [V]?T = None
        var ans: [V]?T = None
        for p in Pre:
            let v = at[p]
            let m = k1[v] - k0[v]
            var pre = List.filled(m + 1, mon.id)
            if let pv = par[v]:
                pre[0] = lift(ab[v]!, pv, v)
            for j in 0..m:
                let c = bfs[k0[v] + j]
                pre[j + 1] = f(pre[j], lift(dn[c]!, c, v))
            ans[v] = Some(finish(pre[m], v))
            var suf = mon.id
            var j = m - 1
            while j >= 0:
                let c = bfs[k0[v] + j]
                ab[c] = Some(finish(f(pre[j], suf), v))
                suf = f(lift(dn[c]!, c, v), suf)
                j -= 1
        [V] v => ans[v]!

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
    pub fn shapes(self) -> (domain Shape, of: [V]Shape):
        var known = Map<List<Int>, Int>()
        var of: [V]Int = 0
        for p in Pre.rev:
            let v = at[p]
            var key = [of[bfs[i]] for i in k0[v]..k1[v]]
            key.sort()
            of[v] = known.get_or_insert(key, known.len)
        domain Shape = 0..known.len
        (Shape:, of: [V] v => Shape(of[v]))

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The size of every subtree, by vertex: `t.subtree_sizes[v] == t.size(v)`.
    ///
    /// cost: O(1); a view of the stored sizes
    /// empty: never empty
    /// order: keyed by `V`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.subtree_sizes == [5, 3, 1, 1, 1]`
    @get pub fn subtree_sizes(self) -> [V]Int = [V] v => sz[v]

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// A centroid of the tree holding `t.root`: a vertex whose removal leaves no piece of more than
    /// half its vertices. Found by walking heavy children down from the root while the heavy
    /// child's subtree is more than half.
    ///
    /// cost: O(depth)
    /// ties: of two centroids, the one nearer the root
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.centroid() == 1`
    pub fn centroid(self) -> V:
        var c = root
        var down = true
        while down:
            down = false
            if let h = heavy[c]:
                if 2 * sz[h] > sz[root]:
                    c = h
                    down = true
        c

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// Every centroid of the tree holding `t.root`, one or two. Two happen only when the number of
    /// vertices is even and one edge splits the tree in halves.
    ///
    /// cost: O(depth)
    /// empty: never empty
    /// order: the one nearer the root first
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.centroids() == [1]`
    /// example: `domain P = 0..4; let adj: [P]List<P> = [[1], [0, 2], [1, 3], [2]]; Rooted(adj, root: 0).centroids() == [1, 2]`
    pub fn centroids(self) -> List<V>:
        let c = self.centroid()
        var out = [c]
        if let h = heavy[c]:
            if 2 * sz[h] == sz[root]: out.push(h)
        out

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The enter/exit Euler tour, the `Visit` events of `std.graph.dfs`: `Enter(v, via:)` when
    /// entering `v` and `Exit(v, via:)` when leaving its subtree, `via` the parent (`None` at a
    /// root, and a tree has no `Again`), children in `t.Pre` order, each tree of a forest
    /// in turn. Iterative, so deep trees are safe.
    ///
    /// cost: O(1) to make, O(n) to walk
    /// empty: never empty
    /// order: `Enter` events in `t.Pre` order; `Exit(v, _)` right after the last event of `v`'s subtree
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let want: List<Visit<V, V>> = [Visit.Enter(0, None), Visit.Enter(1, Some(0)), Visit.Enter(3, Some(1)), Visit.Exit(3, Some(1)), Visit.Enter(4, Some(1)), Visit.Exit(4, Some(1)), Visit.Exit(1, Some(0)), Visit.Enter(2, Some(0)), Visit.Exit(2, Some(0)), Visit.Exit(0, None)]; t.tour.collect() == want`
    @get pub fn tour(self) -> yields Visit<V, V>:
        var open: List<V> = []
        for p in Pre:
            let v = at[p]
            while open.len > 0 and not self.is_anc(open[open.len - 1], v):
                let u = open[open.len - 1]
                open.pop()
                yield Visit.Exit(u, par[u])
            yield Visit.Enter(v, par[v])
            open.push(v)
        while open.len > 0:
            let u = open[open.len - 1]
            open.pop()
            yield Visit.Exit(u, par[u])

    /// The positions of `v`'s subtree, `v` first: one contiguous range of `t.Pre`.
    ///
    /// cost: O(1)
    /// empty: never empty; it holds `t.pos(v)`
    /// order: increasing positions
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.sub(1).map(p => t.at[p]).collect() == [1, 3, 4]`
    pub fn sub(self, v: V) -> Range<Pre> = tin[v].l..tin[v].r(sz[v])

    /// The positions of `v`'s descendants: `sub(v)` without `v`.
    ///
    /// cost: O(1)
    /// empty: empty at a leaf
    /// order: increasing positions
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.below(1).map(p => t.at[p]).collect() == [3, 4] and t.below(2).count() == 0`
    pub fn below(self, v: V) -> Range<Pre> = tin[v].r..tin[v].r(sz[v])

    /// Whether `a` is an ancestor of `b`; a vertex is its own ancestor.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.is_anc(1, 3) and t.is_anc(3, 3) and not t.is_anc(2, 3)`
    pub fn is_anc(self, a: V, b: V) -> Bool = tin[b] in self.sub(a)

    /// The child end of the tree edge between `u` and `v`, in either order.
    ///
    /// cost: O(1)
    /// ties: None; exactly one end is the child
    /// traps: when `u` and `v` are not joined by a tree edge
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.lower(0, 1) == 1 and t.lower(3, 1) == 3`
    pub fn lower(self, u: V, v: V) -> V:
        let (c, p) = if dep[u] > dep[v]: (u, v) else: (v, u)
        if let q = par[c]: assert q == p
        else: assert false
        c

    /// The lowest common ancestor of `a` and `b`, by heavy-path climbing.
    ///
    /// cost: O(log n)
    /// traps: when `a` and `b` are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.lca(3, 4) == 1 and t.lca(3, 2) == 0 and t.lca(1, 3) == 1`
    pub fn lca(self, a: V, b: V) -> V:
        var (x, y) = (a, b)
        while top[x] != top[y]:
            if dep[top[x]] < dep[top[y]]: swap(mut x, mut y)
            x = par[top[x]]!
        if dep[x] < dep[y]: x else: y

    /// The length of the path between `a` and `b`: its number of edges, or the total of its edge
    /// lengths in a tree built from `Weighted` arcs.
    ///
    /// cost: O(log n)
    /// traps: when `a` and `b` are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.dist(3, 2) == 3 and t.dist(4, 4) == 0`
    pub fn dist(self, a: V, b: V) -> Int = far[a] + far[b] - 2 * far[self.lca(a, b)]

    /// The simple path of vertices from `a` to `b`, both included.
    ///
    /// cost: O(log n + |path|)
    /// empty: never empty; `[a]` when `a == b`
    /// order: from `a` to `b`
    /// traps: when `a` and `b` are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.path(3, 2) == [3, 1, 0, 2] and t.path(4, 4) == [4]`
    pub fn path(self, a: V, b: V) -> List<V>:
        let c = self.lca(a, b)
        var up: List<V> = []
        var x = a
        while x != c:
            up.push(x)
            x = par[x]!
        up.push(c)
        var down: List<V> = []
        var y = b
        while y != c:
            down.push(y)
            y = par[y]!
        for v in down.rev: up.push(v)
        up

    /// A longest simple path in the tree holding `t.root`: `(len, u, v)` with
    /// `t.dist(u, v) == len` as large as it gets, found in one bottom-up sweep that joins, at each
    /// vertex, its two deepest child branches.
    ///
    /// cost: O(n)
    /// ties: the first longest path the sweep (children before parents, `t.Pre` reversed) meets
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.diameter() == (len: 3, u: 2, v: 4)`
    pub fn diameter(self) -> (len: Int, u: V, v: V):
        var best_len = 0
        var (du, dv) = (root, root)
        var down: [V]Int = 0
        var leaf: [V]V = [V] v => v
        for p in Pre.rev:
            let v = at[p]
            if let par_v = par[v]:
                let cand = down[v] + (far[v] - far[par_v])
                if down[par_v] + cand > best_len:
                    best_len = down[par_v] + cand
                    (du, dv) = (leaf[par_v], leaf[v])
                if cand > down[par_v]:
                    down[par_v] = cand
                    leaf[par_v] = leaf[v]
        (len: best_len, u: du, v: dv)

    /// The ancestor `k` edges above `v` (`v` itself for `k == 0`), or `None` if that is above the
    /// root.
    ///
    /// cost: O(log n)
    /// traps: when `k` is negative
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.ancestor(3, 2) == Some(0) and t.ancestor(3, 3) == None and t.ancestor(3, 0) == Some(3)`
    pub fn ancestor(self, v: V, k: Int) -> ?V:
        if k < 0 or k > dep[v]: return None
        var (x, left) = (v, k)
        while dep[x] - dep[top[x]] < left:
            left -= dep[x] - dep[top[x]] + 1
            x = par[top[x]]!
        at[tin[x].offset(-left)!]

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The vertex `k` edges from `a` on the path to `b` (`a` for `k == 0`), or `None` if the path
    /// is shorter.
    ///
    /// cost: O(log n)
    /// traps: when `k` is negative, or `a` and `b` are in different trees
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.jump(3, 2, 2) == Some(0) and t.jump(3, 2, 3) == Some(2) and t.jump(3, 2, 4) == None`
    pub fn jump(self, a: V, b: V, k: Int) -> ?V:
        assert k >= 0
        let c = self.lca(a, b)
        let (da, db) = (dep[a] - dep[c], dep[b] - dep[c])
        if k <= da: self.ancestor(a, k)
        elif k <= da + db: self.ancestor(b, da + db - k)
        else: None

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The highest ancestor `u` of `v` such that `pred(u)` holds, or `None` when `pred(v)` is false.
    /// `pred` must be true and then false on the way up, and is checked only where the search
    /// probes it.
    ///
    /// cost: O(log² n) calls of `pred`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.jump_while(3, v => v != 0) == Some(1) and t.jump_while(3, v => v == 0) == None`
    pub fn jump_while(self, v: V, pred: V -> Bool) -> ?V:
        if not pred(v): return None
        var cur = v
        while true:
            let h = top[cur]
            if pred(h):
                if let p = par[h]:
                    if pred(p):
                        cur = p
                        continue
                return Some(h)
            let i = (tin[h].to_idx + 1..=tin[cur].to_idx).first_true(i => pred(at[Pre.nth(i)]))!
            return Some(at[Pre.nth(i)])
        None

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
    ) -> yields (range: Range<Pre>, rising: Bool):
        var (x, y) = (a, b)
        var falling = List<Range<Pre>>()
        while top[x] != top[y]:
            if dep[top[x]] >= dep[top[y]]:
                yield (range: tin[top[x]]..=tin[x], rising: true)
                x = par[top[x]]!
            else:
                falling.push(tin[top[y]]..=tin[y])
                y = par[top[y]]!
        let skip = if edges: 1 else: 0
        if dep[x] >= dep[y]:
            yield (range: tin[y].l(1 - skip)..tin[x].r, rising: true)
        else:
            yield (range: tin[x].l(1 - skip)..tin[y].r, rising: false)
        for r in falling.rev: yield (range: r, rising: false)

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
    pub fn fold_path<S, T>(self, a: V, b: V, ds: S, edges: Bool = false) -> T:
        let m = ds.monoid
        var acc = m.id
        for c in self.chains(a, b, edges):
            let part = ds.fold(c.range)
            acc = m.op(acc, if c.rising: part.rev() else: part)
        acc

    /// Applies the update `tag` to every vertex on the path from `a` to `b`, through
    /// `ds.apply(range, tag)` once per chain range. With `edges`, the lowest common ancestor is
    /// left out.
    ///
    /// cost: O(log² n)
    /// empty: nothing changes for `a == b` with `edges`
    /// traps: when `a` and `b` are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); var seg = SegTree(+, t.reorder([0, 0, 0, 0, 0]), act: add); t.apply_path(3, 2, mut seg, 5); t.fold_path(3, 2, seg) == 20 and seg.fold(t.sub(1)) == 10`
    pub fn apply_path<S, F>(self, a: V, b: V, mut ds: S, tag: F, edges: Bool = false):
        for c in self.chains(a, b, edges): ds.apply(c.range, tag)

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
    pub fn virtual(self, keys: List<V>) -> VirtualTree<V, _>:
        var ks = List<Pre>()
        ks.reserve(keys.len)
        for v in keys: ks.push(tin[v])
        ks.sort()
        dedup(mut ks)
        var ps = List<Pre>()
        ps.reserve(2 * ks.len)
        for p in ks: ps.push(p)
        for i in 1..ks.len: ps.push(tin[self.lca(at[ks[i - 1]], at[ks[i]])])
        ps.sort()
        dedup(mut ps)
        domain K = 0..ps.len
        var vs: [K]V = self.root
        var up: [K]?K = None
        var len: [K]Int = 0
        var key: [K]Bool = false
        var st = List<K>()
        st.reserve(K.count)
        var j = 0
        for x in K:
            vs[x] = at[ps[x.ord]]
            if j < ks.len and ks[j] == ps[x.ord]:
                key[x] = true
                j += 1
            while st.len > 0 and not self.is_anc(vs[st.last!], vs[x]): st.pop()
            if let p = st.last:
                up[x] = p
                len[x] = far[vs[x]] - far[vs[p]]
            st.push(x)
        VirtualTree<V, K>(vs, up, len, key)

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The edge length from `v` up to its parent: 0 at a root, 1 unless the arcs are `Weighted`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.w(3) == 1 and t.w(0) == 0`
    pub fn w(self, v: V) -> Int:
        if let p = par[v]: far[v] - far[p] else: 0

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// `v`, its parent, and so on up to its root.
    ///
    /// cost: O(1) per vertex, O(depth(v)) in all
    /// empty: never empty; it starts with `v`
    /// order: from `v` up
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.ancestors(3).collect() == [3, 1, 0]`
    pub fn ancestors(self, v: V) -> yields V:
        var x = v
        var more = true
        while more:
            yield x
            if let p = par[x]: x = p
            else: more = false

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The top of the heavy chain through `v`: the highest vertex reached by heavy edges.
    ///
    /// cost: O(1)
    /// ties: None; the heavy children fix the chain
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.top(3) == 0 and t.top(4) == 4 and t.top(2) == 2`
    pub fn top(self, v: V) -> V = top[v]

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The heavy chain through `v` as one range of positions, its top first.
    ///
    /// cost: O(1)
    /// empty: never empty
    /// order: top first, down the chain
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.chain(3).map(p => t.at[p]).collect() == [0, 1, 3] and t.chain(2).count() == 1`
    pub fn chain(self, v: V) -> Range<Pre>:
        let h = top[v]
        var cur = v
        while let c = heavy[cur]: cur = c
        tin[h].l..tin[cur].r

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The subtree of `v` were the tree rooted at `root`, as at most two ranges of positions:
    /// `sub(v)` when `root` is outside it, all of `t.Pre` when `v == root`, and otherwise
    /// everything except the subtree of the child of `v` toward `root`.
    ///
    /// cost: O(log n)
    /// empty: empty ranges are skipped, so one or two ranges come out
    /// order: increasing positions
    /// traps: when `v` and `root` are in different trees of a forest
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); [t.at[p] for r in t.sub_at(1, 3) for p in r] == [0, 1, 4, 2] and t.sub_at(1, 2).count() == 1`
    pub fn sub_at(self, v: V, root: V) -> yields Range<Pre>:
        if self.lca(v, root) != v:
            yield self.sub(v)
        elif v == root:
            yield span(Pre.cuts(0), Pre.cuts(Pre.count))
        else:
            let c = self.ancestor(root, dep[root] - dep[v] - 1)!
            yield span(Pre.cuts(0), tin[c].l)
            let after = span(tin[c].r(sz[c]), Pre.cuts(Pre.count))
            if not after.is_empty: yield after

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// Whether `c` lies on the path from `a` to `b`, ends included.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.on_path(1, 3, 2) and t.on_path(3, 3, 2) and not t.on_path(4, 3, 2)`
    pub fn on_path(self, c: V, a: V, b: V) -> Bool:
        let m = self.lca(a, b)
        self.is_anc(m, c) and (self.is_anc(c, a) or self.is_anc(c, b))

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The one vertex on all three paths between `a`, `b` and `c`: the deepest of their pairwise
    /// lowest common ancestors.
    ///
    /// cost: O(log n)
    /// traps: when the three are not in one tree
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.meet(3, 4, 2) == 1 and t.meet(3, 2, 0) == 0`
    pub fn meet(self, a: V, b: V, c: V) -> V:
        let (x, y, z) = (self.lca(a, b), self.lca(b, c), self.lca(a, c))
        if dep[x] >= dep[y] and dep[x] >= dep[z]: x
        elif dep[y] >= dep[z]: y
        else: z

    /// The other half of `reroot`: for `v` that is not a root, the DP of the tree without `sub(v)`,
    /// rooted at `parent(v)`; `None` at a root. Takes the arguments of `reroot`, for answers that
    /// are per edge rather than per vertex.
    ///
    /// cost: O(n) calls of `lift` and `finish`, O(n) calls of `op`
    /// empty: never empty
    /// order: keyed by `V`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.above(+, finish: (s, v) => s + 1) == [None, Some(2), Some(4), Some(4), Some(4)]`
    pub fn above<T, U, M>(
        self,
        op: M,
        lift: (T, V, V) -> U = (x, c, p) => x,
        finish: (U, V) -> T,
    ) -> [V]?T:
        let mon = Monoid<U>.of(op)
        let f = mon.op
        var dn: [V]?T = None
        for p in Pre.rev:
            let v = at[p]
            var acc = mon.id
            for i in k0[v]..k1[v]:
                let c = bfs[i]
                acc = f(acc, lift(dn[c]!, c, v))
            dn[v] = Some(finish(acc, v))
        var ab: [V]?T = None
        for p in Pre:
            let v = at[p]
            let m = k1[v] - k0[v]
            if m == 0: continue
            var pre = List.filled(m + 1, mon.id)
            if let pv = par[v]:
                pre[0] = lift(ab[v]!, pv, v)
            for j in 0..m:
                let c = bfs[k0[v] + j]
                pre[j + 1] = f(pre[j], lift(dn[c]!, c, v))
            var suf = mon.id
            var j = m - 1
            while j >= 0:
                let c = bfs[k0[v] + j]
                ab[c] = Some(finish(f(pre[j], suf), v))
                suf = f(lift(dn[c]!, c, v), suf)
                j -= 1
        ab

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
    ) -> ?(len: Int, u: V, v: V):
        let Some(p) = x else: return y
        let Some(q) = y else: return x
        var best = if q.len > p.len: q else: p
        for (u, v) in [(p.u, q.u), (p.u, q.v), (p.v, q.u), (p.v, q.v)]:
            let d = self.dist(u, v)
            if d > best.len: best = (len: d, u: u, v: v)
        Some(best)

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// Every vertex ordered by depth, each depth in `t.Pre` order, over a fresh domain, so the
    /// descendants of `v` at one depth are one range.
    ///
    /// cost: O(n)
    /// empty: never empty
    /// order: by depth, then `t.Pre`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ly = t.layers(); ly.level(1).map(b => ly.at[b]).collect() == [1, 2]`
    pub fn layers(self) -> Layers<V, _>:
        var start: List<Int> = List.filled(V.count + 1, 0)
        for v in V: start[dep[v] + 1] += 1
        for d in 1..start.len: start[d] += start[d - 1]
        var fill = start.copy()
        var order: List<V> = List.filled(V.count, root)
        var slot: [V]Int = 0
        for p in Pre:
            let v = at[p]
            slot[v] = fill[dep[v]]
            order[slot[v]] = v
            fill[dep[v]] += 1
        domain Level = 0..V.count
        Layers::<V, Level>(
            [Level] b => order[b.to_idx],
            [V] v => Level.nth(slot[v]),
            start,
            [V] v => tin[v].to_idx,
            [V] v => sz[v],
            [V] v => dep[v],
        )

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The 2n enter and exit events of `tour` over a fresh domain, for Mo's algorithm on paths and
    /// on subtrees.
    ///
    /// cost: O(n)
    /// empty: never empty
    /// order: `tour` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ev = t.events(); ev.sub(1).count() == 6`
    pub fn events(self) -> Events<V, _>:
        domain Ev = 0..2 * V.count
        var jt = JumpTree<V>()
        for p in Pre:
            if let u = par[at[p]]: jt.add(at[p], u)
        var tour: List<Visit<V, V>> = []
        for e in self.tour: tour.push(e)
        let ent = [V] v => 2 * tin[v].to_idx - dep[v]
        Events<V, Ev>(
            [Ev] i => tour[i.to_idx],
            [V] v => Ev.nth(ent[v]),
            [V] v => Ev.nth(ent[v] + 2 * sz[v] - 1),
            jt,
        )

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The closed Euler walk: 2n - 1 visits from the root back to it, each pair of neighbours
    /// joined by a tree edge, children in `t.Pre` order. `by:` reorders each vertex's children
    /// by a key, stably, so `by: c => c in deep` walks the children in `deep` last. A walk that
    /// visits every vertex of a connected graph is `Rooted(g, root: r).euler_tour()`. A forest
    /// walks only the first tree.
    ///
    /// cost: O(1) per item
    /// empty: never empty; `[r]` for a single vertex
    /// order: walk order from the root
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.euler_tour().collect() == [0, 1, 3, 1, 4, 1, 0, 2, 0]`
    /// example: `domain V = 0..3; let adj: [V]List<V> = [[1, 2], [0], [0]]; let t = Rooted(adj, root: 0); t.euler_tour(by: c => c == 1).collect() == [0, 2, 0, 1, 0]`
    pub fn euler_tour<K: Ord>(self, by: V -> K = _ => ()) -> yields V:
        var kids: [V]List<V> = []
        kids[root] = self.children(root).sorted(by: c => tin[c]).sorted(by: by)
        var stack = [root]
        var next: List<Int> = [0]
        yield root
        while stack.len > 0:
            let v = stack[stack.len - 1]
            let i = next[next.len - 1]
            if i < kids[v].len:
                next[next.len - 1] = i + 1
                let c = kids[v][i]
                kids[c] = self.children(c).sorted(by: x => tin[x]).sorted(by: by)
                stack.push(c)
                next.push(0)
                yield c
            else:
                stack.pop()
                next.pop()
                if stack.len > 0: yield stack[stack.len - 1]

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The long-path decomposition, for DP indexed by depth merged in O(n) in total. The tallest
    /// child is the first in child order on ties.
    ///
    /// cost: O(n)
    /// empty: never empty
    /// ties: the first tallest child in child order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let lp = t.long_paths(); lp.height == [2, 1, 0, 0, 0] and lp.deep[1] == Some(3)`
    pub fn long_paths(self) -> LongPaths<V, _>:
        var deep: [V]?V = None
        var height: [V]Int = 0
        for p in Pre.rev:
            let v = at[p]
            var best_h = -1
            var best_c: ?V = None
            for i in k0[v]..k1[v]:
                let c = bfs[i]
                if height[c] > best_h:
                    best_h = height[c]
                    best_c = Some(c)
            if let c = best_c:
                deep[v] = Some(c)
                height[v] = best_h + 1
        var slot_idx: [V]Int = 0
        var cur = 0
        for p in Pre:
            let v = at[p]
            let is_top = if let pv = par[v]: deep[pv] != Some(v) else: true
            if is_top:
                var x = v
                var more = true
                while more:
                    slot_idx[x] = cur
                    cur += 1
                    if let c = deep[x]: x = c
                    else: more = false
        domain Slot = 0..V.count
        LongPaths::<V, Slot>(deep, height, [V] v => Slot.nth(slot_idx[v]))

    // blocked: `Rooted` resolves to a compiler built-in (GAPS.md tree `Rooted`)
    /// The subtree of `v` in `t.Pre` order as `std.dp` steps: `step.state` is the vertex,
    /// `step.depth` its depth below `v`, and `step.prune()` skips the rest of its subtree
    /// (decision 126).
    ///
    /// cost: O(size(v)), less when pruned
    /// empty: never empty; it starts with `v`
    /// order: `t.Pre` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); t.preorder(1).map(s => (s.state, s.depth)).collect() == [(1, 0), (3, 1), (4, 1)]`
    pub fn preorder(self, v: V) -> yields Step<V>:
        let d0 = dep[v]
        for p in self.sub(v):
            let u = at[p]
            yield Step(u, dep[u] - d0)

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
    pub fn new(next: Next<V>) -> Centroids<V>:
        var adj: [V]List<V> = []
        for u in V:
            for a in next.out(u): adj[u].push(a.to)
        var dead: [V]Bool = false
        var cpar: [V]?V = None
        var cdep: [V]Int = 0
        var down: [V]List<(c: V, dist: Int, branch: V)> = []
        var held: [V]List<V> = []
        var from: [V]V = [V] v => v
        var dist: [V]Int = 0
        var via: [V]V = [V] v => v
        var sz: [V]Int = 0
        var root = V.nth(0)
        var work: List<(v: V, p: ?V, d: Int)> = [(v: V.nth(0), p: None, d: 0)]
        var next_job = 0
        while next_job < work.len:
            let job = work[next_job]
            next_job += 1
            from[job.v] = job.v
            var comp = [job.v]
            var i = 0
            while i < comp.len:
                let u = comp[i]
                i += 1
                for x in adj[u]:
                    if not dead[x] and x != from[u]:
                        from[x] = u
                        comp.push(x)
            for u in comp: sz[u] = 1
            for j in (1..comp.len).rev: sz[from[comp[j]]] += sz[comp[j]]
            var c = job.v
            var moved = true
            while moved:
                moved = false
                for x in adj[c]:
                    if not moved and not dead[x] and x != from[c] and 2 * sz[x] > comp.len:
                        c = x
                        moved = true
            cpar[c] = job.p
            cdep[c] = job.d
            if job.d == 0: root = c
            from[c] = c
            dist[c] = 0
            via[c] = c
            var ring = [c]
            var k = 0
            while k < ring.len:
                let u = ring[k]
                k += 1
                down[u].push((c:, dist: dist[u], branch: via[u]))
                held[c].push(u)
                for x in adj[u]:
                    if not dead[x] and x != from[u]:
                        from[x] = u
                        dist[x] = dist[u] + 1
                        via[x] = if u == c: x else: via[u]
                        ring.push(x)
            dead[c] = true
            for x in adj[c]:
                if not dead[x]: work.push((v: x, p: Some(c), d: job.d + 1))
        assert work.len == V.count, "`Centroids`: `next` is not a connected tree"
        var anc: [V]List<(c: V, dist: Int, branch: V)> = []
        for v in V:
            for j in (0..down[v].len).rev: anc[v].push(down[v][j])
        Centroids::<V>(root, cpar, [V] v => cdep[v], anc, held)

    /// The parent of centroid `c` in the centroid tree; `None` at `cd.root`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let cd = Centroids(adj); cd.parent(0) == Some(1) and cd.parent(1) == None`
    pub fn parent(self, c: V) -> ?V = cpar[c]

    /// The depth of `c` in the centroid tree, at most log2 n.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let cd = Centroids(adj); cd.depth(1) == 0 and cd.depth(3) == 1 and cd.depth(2) == 2`
    pub fn depth(self, c: V) -> Int = cdep[c]

    /// Every centroid whose piece holds `v`, with the tree distance from it: `v` itself at distance
    /// 0, then up the centroid tree. At most log2 n + 1 entries, no allocation.
    ///
    /// cost: O(1) per entry, O(log n) in all
    /// empty: never empty; it starts with `(c: v, dist: 0)`
    /// order: nearest centroid first, up to `cd.root`
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let cd = Centroids(adj); cd.ancestors(2).collect() == [(c: 2, dist: 0), (c: 0, dist: 1), (c: 1, dist: 2)]`
    pub fn ancestors(self, v: V) -> yields (c: V, dist: Int):
        for x in anc[v]: yield (c: x.c, dist: x.dist)

    /// The piece of centroid `c`: each vertex `v`, its distance from `c`, and `branch`, the
    /// neighbour of `c` on the way to `v` (`c` itself for `v == c`). Pairs in different branches are
    /// exactly the paths whose highest centroid is `c`.
    ///
    /// cost: O(size of the piece)
    /// empty: never empty; it starts with `(v: c, dist: 0, branch: c)`
    /// order: breadth-first from `c`, neighbours in the order `next` lists them
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let cd = Centroids(adj); cd.piece(0).collect() == [(v: 0, dist: 0, branch: 0), (v: 2, dist: 1, branch: 2)] and cd.piece(1).count() == 5`
    pub fn piece(self, c: V) -> yields (v: V, dist: Int, branch: V):
        for v in held[c]:
            let x = anc[v][cdep[v] - cdep[c]]
            yield (v:, dist: x.dist, branch: x.branch)

// ---- tests

/// A random tree on `V` as adjacency lists, with arcs in both directions.
fn random_tree<V>(any: V) -> [V]List<V>:
    var adj: [V]List<V> = []
    var label = [v for v in V]
    shuffle(mut label)
    for i in 1..V.K:
        let (c, p) = (label[i], label[rand(0, i - 1)])
        adj[c].push(p)
        adj[p].push(c)
    adj

/// The vertices from `a` to `b`, by climbing parents.
fn slow_path<V, Pre>(r: Rooted<V, Pre>, a: V, b: V) -> List<V>:
    var (x, y) = (a, b)
    var (left, right) = (List<V>(), List<V>())
    while x != y:
        if r.depth(x) >= r.depth(y):
            left.push(x)
            x = r.parent(x)!
        else:
            right.push(y)
            y = r.parent(y)!
    left.push(x)
    left + right.rev

@test
fn subtrees_and_chains_are_contiguous():
    domain V = 0..14
    let adj = random_tree(V.first!)
    let r = Rooted(adj, V.first!)
    for v in V:
        var inside = 0
        for p in r.sub(v):
            assert r.is_anc(v, r.at[p])
            inside += 1
        assert inside == r.size(v) and r.at[r.pos(v)] == v and r.sub(v).first! == r.pos(v)
        let ch = [r.at[p] for p in r.chain(v)]
        assert ch.any(u => u == v) and ch.first! == r.top(v)
        for k in 1..ch.len: assert r.parent(ch[k]) == Some(ch[k - 1])

@test
fn a_table_of_neighbour_lists_roots_like_the_fn_that_reads_it():
    domain V = 0..9
    let adj = random_tree(V.first!)
    let (a, b) = (Rooted(adj, V.first!), Rooted(v => adj[v], V.first!))
    assert [v for v in a.at] == [v for v in b.at] and a.par == b.par and a.sz == b.sz

@test
fn lca_dist_ancestor_and_step_match_climbing():
    domain V = 0..14
    let adj = random_tree(V.first!)
    let r = Rooted(adj, V.first!)
    for a in V:
        var up = [a]
        while let p = r.parent(up.last!): up.push(p)
        assert r.depth(a) == up.len - 1
        for k in 0..up.len + 1:
            assert r.ancestor(a, k) == (if k < up.len: Some(up[k]) else: None)
        for b in V:
            let p = slow_path(r, a, b)
            assert r.dist(a, b) == p.len - 1
            assert r.lca(a, b) == p.min(by: v => r.depth(v))!
            for k in 0..p.len + 1:
                assert r.jump(a, b, k) == (if k < p.len: Some(p[k]) else: None)

@test
fn weighted_dist_adds_the_lengths_on_the_path():
    domain V = 0..12
    let plain = random_tree(V.first!)
    let len = [V, V] (a, b) => (a.ord + 1) * (b.ord + 1) % 7 + 1
    let adj = [V] u => [(to: v, w: len[u, v]) for v in plain[u]]
    let r = Rooted(adj, V.first!)
    for a in V:
        for b in V:
            let p = slow_path(r, a, b)
            assert r.dist(a, b) == sum(len[p[i], p[i + 1]] for i in 0..p.len - 1)

@test
fn path_folds_keep_the_order_from_a_to_b():
    domain V = 0..10
    let adj = random_tree(V.first!)
    let r = Rooted(adj, V.first!)
    let words = Monoid<Word>(id: Word(""), op: (x, y) => Word(x.s + y.s))
    let seg = SegTree(words, r.reorder([V] v => Word(Str(v.to_idx))))
    for a in V:
        for b in V:
            let p = slow_path(r, a, b)
            var all = ""
            var cut = ""
            for v in p:
                all = all + Str(v.to_idx)
                if v != r.lca(a, b): cut = cut + Str(v.to_idx)
            assert r.fold_path(a, b, seg) == Word(all)
            assert r.fold_path(a, b, seg, edges: true) == Word(cut)

struct Word(s: Str)

impl Reversible for Word:
    fn rev(self) -> Word = Word(Str([s[s.len - 1 - i] for i in 0..s.len]))

@test
fn path_updates_and_subtree_folds_over_one_lazy_tree():
    domain V = 0..12
    let adj = random_tree(V.first!)
    let r = Rooted(adj, V.first!)
    var val: [V]Int = [V] v => rand(0, 9)
    var seg = SegTree(+, r.reorder(val), act: add)
    for _ in 0..300:
        let (a, b, x) = (V(rand(0, V.count - 1)), V(rand(0, V.count - 1)), rand(-5, 5))
        let edges = rand(0, 1) == 1
        var on = List<V>()
        for v in slow_path(r, a, b):
            if not edges or v != r.lca(a, b): on.push(v)
        match rand(0, 2):
            0 =>
                r.apply_path(a, b, mut seg, x, edges:)
                for v in on: val[v] += x
            1 =>
                seg.apply(r.sub(a), x)
                for v in V:
                    if r.is_anc(a, v): val[v] += x
            _ =>
                var want = 0
                for v in on: want += val[v]
                assert r.fold_path(a, b, seg, edges:) == want
                var below = 0
                for v in V:
                    if r.is_anc(a, v): below += val[v]
                assert seg.fold(r.sub(a)) == below

@test
fn centroids_leave_no_piece_over_half():
    for n in 1..=12:
        domain V = 0..n
        let adj = random_tree(V.first!)
        let r = Rooted(adj, V.first!)
        let cs = r.centroids()
        assert cs.first! == r.centroid() and cs.len <= 2
        for v in V:
            let biggest = max(V.count - r.size(v), r.children(v).map(c => r.size(c)).max() ?? 0)
            assert (2 * biggest <= V.count) == cs.any(c => c == v)

@test
fn excl_fold_folds_the_other_children():
    domain V = 0..12
    let adj = random_tree(V.first!)
    let r = Rooted(adj, V.first!)
    let words = Monoid<Str>(id: "", op: (x, y) => x + y)
    for v in V:
        let kids = r.children(v).collect()
        var i = 0
        for e in r.excl_fold(v, words, c => Str(c.to_idx)):
            assert e.c == kids[i]
            var want = ""
            for j in 0..kids.len:
                if j != i: want = want + Str(kids[j].to_idx)
            assert e.rest == want
            i += 1
        assert i == kids.len

@test
fn rerooting_counts_connected_sets_through_every_vertex():
    domain V = 0..10
    let adj = random_tree(V.first!)
    let r = Rooted(adj, V.first!)
    let p = 1_000_000_007
    let times = Monoid<Int>(id: 1, op: (x, y) => x * y % p)
    let all = r.reroot(times, lift: (x, c, u) => x + 1, finish: (s, v) => s)
    var count: [V]Int = 0
    for mask in 1..(1 << V.count):
        var start: ?V = None
        var size = 0
        for v in V:
            if (mask >> v.to_idx) & 1 == 1:
                size += 1
                if start == None: start = Some(v)
        var seen = 1 << start!.to_idx
        var st = [start!]
        while let u = st.pop():
            for w in adj[u]:
                if (mask >> w.to_idx) & 1 == 1 and (seen >> w.to_idx) & 1 == 0:
                    seen |= 1 << w.to_idx
                    st.push(w)
        if seen == mask:
            for v in V:
                if (mask >> v.to_idx) & 1 == 1: count[v] += 1
    for v in V: assert all[v] == count[v]

@test
fn orders_lifting_and_the_virtual_tree():
    domain V = 0..14
    let adj = random_tree(V.first!)
    let r = Rooted(adj, V.first!)
    let up: [V]?(
        V,
        Word,
    ) = [V] v => if let p = r.parent(v): Some((p, Word(Str(v.to_idx)))) else: None
    let l = Lift(up, op: (x, y) => Word(x.s + y.s), identity: Word(""))
    let cost: [V]?(V, Int) = [V] v => if let p = r.parent(v): Some((p, v.to_idx + 1)) else: None
    let lc = Lift(cost, op: (x, y) => x + y, identity: 0)
    for a in V:
        assert l.parent(a) == r.parent(a) and l.depth(a) == r.depth(a)
        for k in 0..r.depth(a) + 2: assert l.ancestor(a, k) == r.ancestor(a, k)
        var word = ""
        var sum = 0
        for u in r.ancestors(a):
            assert l.fold(a, up_to: u) == Word(word.clone()) and lc.fold(a, up_to: u) == sum
            for lim in [sum - 1, sum, sum + 1]:
                var want = a
                for x in r.ancestors(a):
                    if lc.fold(a, up_to: x) <= lim: want = x
                if lim >= 0: assert lc.climb_while(a, s => s <= lim) == want
            word = word + Str(u.to_idx)
            sum += u.to_idx + 1
        for t in 0..r.depth(a) + 1:
            let c = l.climb(a, u => r.depth(u) >= t)
            assert c.at == r.ancestor(
                a,
                max(r.depth(a) - t, 0),
            )! and c.steps == r.depth(a) - r.depth(c.at)
        for b in V: assert l.lca(a, b) == Some(r.lca(a, b))
    for size in 0..6:
        var keys = List<V>()
        for _ in 0..size: keys.push(V(rand(0, V.count - 1)))
        let vt = r.virtual(keys)
        var kept = List<V>()
        for v in V:
            var on = keys.any(k => k == v)
            for a in keys:
                for b in keys:
                    if r.lca(a, b) == v: on = true
            if on: kept.push(v)
        sort(mut kept, by: v => r.pos(v))
        assert [vt.at[x] for x in vt.K] == kept
        for x in vt.K:
            let v = vt.at[x]
            assert vt.key[x] == keys.any(k => k == v)
            var want: ?V = None
            for u in kept:
                if u != v and r.is_anc(u, v):
                    if want == None or r.depth(u) > r.depth(want!): want = Some(u)
            if let y = vt.parent[x]:
                assert Some(vt.at[y]) == want and vt.w[x] == r.dist(v, vt.at[y])
            else:
                assert want == None and vt.w[x] == 0

@test
fn a_tree_from_a_stored_graph():
    domain V = 1..=6
    domain E = 1..=5
    let edges = [E] e => [
        (u: V(1), v: V(2)),
        (u: V(1), v: V(3)),
        (u: V(2), v: V(4)),
        (u: V(2), v: V(5)),
        (u: V(5), v: V(6)),
    ][e.to_idx]
    let g = Graph.undirected(edges)
    let r = Rooted(g, root: V(1))
    assert r.lca(V(4), V(6)) == V(2) and r.dist(V(4), V(6)) == 3 and r.size(V(2)) == 4
    assert r.children(V(2)) == [V(4), V(5)] and r.parent(V(6)) == V(5)

@test
fn from_parents_tour_pos_and_value_owned_pre():
    domain V = 0..13
    var label = [v for v in V]
    shuffle(mut label)
    var par: [V]?V = None
    for i in 1..V.count: par[label[i]] = Some(label[rand(0, i - 1)])
    let t = Rooted.from_parents(par)
    assert t.root == label[0] and Rooted.try_from_parents(par) != None
    let deep = t.reorder([V] v => t.depth(v))
    for v in V:
        assert t.parent(v) == par[v] and t.at[t.pos(v)] == v and deep[t.pos(v)] == t.depth(v)
        assert t.children(v).collect() == [u for u in V if par[u] == Some(v)]
    var entered = List<V>()
    var st = List<V>()
    for e in t.tour:
        match e:
            Enter(v, via) =>
                assert via == par[v]
                entered.push(v)
                st.push(v)
            Exit(v, via) =>
                assert via == par[v] and st.last == Some(v)
                st.pop()
            _ => assert false
    assert st.len == 0 and entered == [t.at[q] for q in t.sub(t.root)]
    domain W = 0..3
    let cyc: [W]?W = [None, Some(2), Some(1)]
    assert Rooted.try_from_parents(cyc) == None

@test
fn weighted_rooted_bfs_tree_on_cyclic_graph():
    domain V = 0..4
    let adj: [V]List<(to: V, w: Int)> = [
        [(to: 1, w: 5), (to: 2, w: 1)],
        [(to: 0, w: 5), (to: 3, w: 2)],
        [(to: 0, w: 1), (to: 3, w: 9)],
        [(to: 1, w: 2), (to: 2, w: 9)],
    ]
    let r = Rooted(adj, V(0))
    assert r.parent(3) == Some(1) and r.depth(3) == 2
    assert r.dist(0, 3) == 7 and r.w(2) == 1 and r.dist(2, 3) == 8

@test
fn lower_is_the_child_end_of_an_edge():
    domain V = 0..9
    let adj = random_tree(V.first!)
    let r = Rooted(adj, V.first!)
    for v in V:
        if let p = r.parent(v):
            assert r.lower(v, p) == v and r.lower(p, v) == v

@test
fn path_and_diameter_match_pairwise_distances():
    domain V = 0..12
    let adj = random_tree(V.first!)
    let r = Rooted(adj, V.first!)
    let d = r.diameter()
    assert r.dist(d.u, d.v) == d.len
    for a in V:
        for b in V:
            assert r.dist(a, b) <= d.len
            let p = r.path(a, b)
            assert p.first == a and p.last == b and p.len == r.dist(a, b) + 1
            for k in 0..p.len - 1:
                assert r.parent(p[k]) == p[k + 1] or r.parent(p[k + 1]) == p[k]

@test
fn heavy_light_and_subtree_cover_the_children():
    domain V = 0..16
    let adj = random_tree(V.first!)
    let r = Rooted(adj, V.first!)
    for v in V:
        let kids = r.children(v).collect()
        let light = r.light(v).collect()
        assert light.len == max(kids.len - 1, 0)
        for c in kids:
            if c not in light:
                for d in kids: assert r.size(d) <= r.size(c)
        for c in light: assert c in kids
        if let p = r.parent(v):
            assert r.top(v) == (if v in r.light(p).collect(): v else: r.top(p))
        else:
            assert r.top(v) == v
        let chain = r.chain(v).map(q => r.at[q]).collect()
        assert chain.first == r.top(v) and r.children(chain.last!).count() == 0
        for k in 1..chain.len:
            assert r.parent(chain[k]) == Some(chain[k - 1]) and chain[k] not in r.light(chain[k - 1]).collect()
        var lights = 0
        for a in r.ancestors(v):
            if let p = r.parent(a):
                if a in r.light(p).collect(): lights += 1
        assert (1 << lights) <= V.count
        for root in V:
            var got = [r.at[q] for g in r.sub_at(v, root) for q in g]
            got.sort()
            var want = Rooted(adj, root).subtree(v).collect()
            want.sort()
            assert got == want

@test
fn a_forest_keeps_each_tree_in_one_range():
    domain V = 0..14
    var par: [V]?V = None
    var adj: [V]List<V> = []
    var roots = [V.first!]
    for i in 1..V.count:
        if rand(0, 3) == 0:
            roots.push(V(i))
        else:
            let p = V(rand(0, i - 1))
            par[V(i)] = Some(p)
            adj[V(i)].push(p)
            adj[p].push(V(i))
    shuffle(mut roots)
    let t = Rooted.forest(adj, from: roots)
    assert t.roots.collect() == roots and t.root == roots[0]
    var top: [V]V = [V] v => v
    for v in V:
        while let p = par[top[v]]: top[v] = p
    for v in V:
        assert t.parent(v) == par[v] and t.root_of(v) == top[v]
        for u in V: assert (t.pos(u) in t.sub(top[v])) == (top[u] == top[v])
    var n = 0
    for k in 0..roots.len:
        assert t.sub(roots[k]).len == [v for v in V if top[v] == roots[k]].len
        if k > 0: assert t.pos(roots[k - 1]) < t.pos(roots[k])
        n += t.sub(roots[k]).len
    assert n == V.count

@test
fn down_and_up_match_depth_size_and_height():
    domain V = 0..14
    let adj = random_tree(V.first!)
    let r = Rooted(adj, V.first!)
    let dep = r.down(-1, (d, v) => d + 1)
    let sz = r.up(+, finish: (s, v) => s + 1)
    let ht = r.up(Monoid<Int>(id: 0, op: max), lift: (h, c, p) => h + 1, finish: (h, v) => h)
    for v in V:
        assert dep[v] == r.depth(v) and sz[v] == r.size(v)
        var (n, h) = (0, 0)
        for u in V:
            if r.is_anc(v, u):
                n += 1
                h = max(h, r.depth(u) - r.depth(v))
        assert sz[v] == n and ht[v] == h

@test
fn reroot_matches_up_from_every_root():
    // reroot folds the parent's side first, a fresh root its neighbours in list order, so the
    // fold commutes; squaring in `lift` still tells shapes apart
    domain V = 0..10
    let adj = random_tree(V.first!)
    let r = Rooted(adj, V.first!)
    let p = 1_000_003
    let lift = (x: Int, c: V, u: V) => (x * x + 3) % p
    let finish = (s: Int, v: V) => (s + v.to_idx + 1) % p
    let all = r.reroot(+, lift:, finish:)
    for v in V: assert all[v] == Rooted(adj, v).up(+, lift:, finish:)[v]

@test
fn shapes_match_the_canonical_string():
    for n in 1..=10:
        domain V = 0..n
        let adj = random_tree(V.first!)
        let r = Rooted(adj, V.first!)
        let (domain S, of) = r.shapes()
        var enc: [V]List<Int> = []
        for d in (0..V.count).rev:
            for v in V:
                if r.depth(v) == d:
                    var kids = [enc[c] for c in r.children(v)]
                    kids.sort()
                    enc[v] = [1] + [x for k in kids for x in k] + [0]
        var seen = Set<List<Int>>()
        for v in V: seen.insert(enc[v])
        assert S.count == seen.len
        for a in V:
            for b in V: assert (of[a] == of[b]) == (enc[a] == enc[b])

@test
fn centroid_ancestors_split_every_path():
    for n in 1..=12:
        domain V = 0..n
        let adj = random_tree(V.first!)
        let r = Rooted(adj, V.first!)
        let cd = Centroids(adj)
        assert cd.parent(cd.root) == None
        var anc: [V]List<(c: V, dist: Int)> = []
        for v in V:
            anc[v] = cd.ancestors(v).collect()
            assert anc[v].len == cd.depth(v) + 1 and anc[v].last!.c == cd.root
            assert anc[v][0].c == v and anc[v][0].dist == 0
            for i in 0..anc[v].len:
                assert anc[v][i].dist == r.dist(v, anc[v][i].c)
                if i + 1 < anc[v].len: assert cd.parent(anc[v][i].c) == Some(anc[v][i + 1].c)
        for a in V:
            for b in V:
                var best = anc[a].last!
                var db = 0
                for x in anc[a]:
                    for y in anc[b]:
                        if x.c == y.c and cd.depth(x.c) >= cd.depth(best.c):
                            best = x
                            db = y.dist
                assert best.dist + db == r.dist(a, b)
                var (ba, bb): (?V, ?V) = (None, None)
                for p in cd.piece(best.c):
                    if p.v == a: ba = Some(p.branch)
                    if p.v == b: bb = Some(p.branch)
                assert ba != None and bb != None
                assert a == b or a == best.c or b == best.c or ba != bb

@test
fn a_jump_tree_answers_what_the_rooted_tree_does():
    domain V = 0..14
    let adj = random_tree(V.first!)
    let r = Rooted(adj, V.first!)
    var j = JumpTree<V>()
    for d in 1..V.count:
        for v in V:
            if r.depth(v) == d: j.add(v, r.parent(v)!)
    for a in V:
        assert j.parent(a) == r.parent(a) and j.depth(a) == r.depth(a) and j.root_of(a) == V.first!
        for k in 0..r.depth(a) + 2: assert j.ancestor(a, k) == r.ancestor(a, k)
        for t in 0..r.depth(a) + 2:
            let ok = (v: V) => r.depth(v) >= t
            var want: ?V = None
            if ok(a):
                var x = a
                while let p = r.parent(x):
                    if not ok(p): break
                    x = p
                want = Some(x)
            assert j.jump_while(a, ok) == want
        for b in V:
            assert j.lca(a, b) == Some(r.lca(a, b)) and j.dist(a, b) == Some(r.dist(a, b))
            for k in 0..r.dist(a, b) + 2: assert j.jump(a, b, k) == r.jump(a, b, k)

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
    pub fn new() -> JumpTree<V> = JumpTree([V] _ => None, [V] v => v, [V] _ => 0)

    /// Hangs `v`, a lone vertex not yet in any tree, below `p`, setting its skew-binary jump
    /// pointer from `p`'s.
    ///
    /// cost: O(1)
    /// traps: when `v` already has a parent, or `v == p`
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.parent(2) == Some(1) and j.depth(3) == 2`
    pub fn add(mut self, v: V, p: V):
        assert par[v] == None and jump[v] == v and v != p
        par[v] = Some(p)
        dep[v] = dep[p] + 1
        let j = jump[p]
        jump[v] = if dep[p] - dep[j] == dep[j] - dep[jump[j]]: jump[j] else: p

    /// The parent of `v`, or `None` at a root.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.parent(1) == Some(0) and j.parent(0) == None`
    pub fn parent(self, v: V) -> ?V = par[v]

    /// The number of edges from `v` up to its root.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.depth(0) == 0 and j.depth(2) == 2`
    pub fn depth(self, v: V) -> Int = dep[v]

    /// The root of `v`'s tree.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.root_of(3) == 0`
    pub fn root_of(self, v: V) -> V:
        var x = v
        while jump[x] != x: x = jump[x]
        x

    /// The ancestor `k` edges above `v` (`v` for `k == 0`), or `None` if that is above the root.
    ///
    /// cost: O(log n)
    /// traps: when `k` is negative
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.ancestor(2, 2) == Some(0) and j.ancestor(2, 3) == None`
    pub fn ancestor(self, v: V, k: Int) -> ?V:
        if k < 0 or k > dep[v]: return None
        let d = dep[v] - k
        var x = v
        while dep[x] > d: x = if dep[jump[x]] >= d: jump[x] else: par[x]!
        x

    /// The lowest common ancestor of `a` and `b`, or `None` in different trees.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.lca(2, 3) == Some(1) and j.lca(2, 0) == Some(0)`
    /// example: `domain V = 0..3; var j = JumpTree<V>(); j.add(1, 0); j.lca(1, 2) == None`
    pub fn lca(self, a: V, b: V) -> ?V:
        var x = if dep[a] > dep[b]: self.ancestor(a, dep[a] - dep[b])! else: a
        var y = if dep[b] > dep[a]: self.ancestor(b, dep[b] - dep[a])! else: b
        while x != y:
            if par[x] == None: return None
            if jump[x] != jump[y]:
                x = jump[x]
                y = jump[y]
            else:
                x = par[x]!
                y = par[y]!
        x

    /// The highest ancestor `u` of `v` such that `pred(u)` holds, or `None` when `pred(v)` is false.
    /// `pred` must be true and then false on the way up.
    ///
    /// cost: O(log n) calls of `pred`
    /// traps: never
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.jump_while(2, v => v != 0) == Some(1) and j.jump_while(2, v => v == 0) == None`
    pub fn jump_while(self, v: V, pred: V -> Bool) -> ?V:
        if not pred(v): return None
        var x = v
        while par[x] != None:
            if pred(jump[x]): x = jump[x]
            elif pred(par[x]!): x = par[x]!
            else: return Some(x)
        Some(x)

    /// The number of edges between `a` and `b`; `None` across trees.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.dist(2, 3) == Some(2) and j.dist(0, 0) == Some(0)`
    pub fn dist(self, a: V, b: V) -> ?Int:
        let c = self.lca(a, b)
        if c == None: return None
        Some(dep[a] + dep[b] - 2 * dep[c!])

    /// The `k`-th vertex on the path from `a` to `b` (`a` is the 0th); `None` past `b` or across
    /// trees.
    ///
    /// cost: O(log n)
    /// traps: when `k` is negative
    /// example: `domain V = 0..4; var j = JumpTree<V>(); j.add(1, 0); j.add(2, 1); j.add(3, 1); j.jump(2, 3, 1) == Some(1) and j.jump(2, 3, 2) == Some(3) and j.jump(2, 3, 3) == None`
    pub fn jump(self, a: V, b: V, k: Int) -> ?V:
        assert k >= 0
        let c = self.lca(a, b)
        if c == None: return None
        let up = dep[a] - dep[c!]
        let len = up + dep[b] - dep[c!]
        if k <= up: self.ancestor(a, k) elif k <= len: self.ancestor(b, len - k) else: None

/// Binary lifting over a fixed forest given by parent links, each carrying a value on the edge
/// up: O(n log n) build and memory, O(log n) queries. A forest that grows leaf by leaf is a
/// `JumpTree`; one that changes is a `LinkCut`.
pub struct Lift<V, T>(op: (T, T) -> T, identity: T):
    up: [V]?V
    dep: [V]0..=V.count
    jmp: List<[V]?V>
    val: List<[V]T>

    /// Binary lifting over the forest `par`: `par[v]` is `v`'s parent with the value on the edge
    /// to it, `None` at a root. `op` must be associative with identity `identity`; it need not
    /// commute, since folds run from the lower end up.
    ///
    /// cost: O(n log n) time and memory
    /// traps: when the parent links have a cycle
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.fold(2, up_to: 0) == 7 and l.depth(3) == 2`
    pub fn new(par: [V]?(V, T), op: (T, T) -> T, identity: T) -> Lift<V, T>:
        var up: [V]?V = None
        var e: [V]T = identity
        for v in V:
            if let (p, x) = par[v]:
                up[v] = Some(p)
                e[v] = x
        var dep: [V]0..=V.count = 0
        var state: [V]Int = 0
        var stack = List<V>()
        for s in V:
            var x = s
            while state[x] == 0:
                state[x] = 1
                stack.push(x)
                if let p = up[x]: x = p
                else: break
            assert state[x] != 1 or up[x] == None, "`Lift.new`: the parent links have a cycle"
            while stack.len > 0:
                let y = stack.pop()!
                if let p = up[y]: dep[y] = dep[p] + 1
                state[y] = 2
        var jmp = [up]
        var val = [e]
        var k = 1
        while (1 << k) <= V.count:
            let pj = jmp[k - 1]
            let pv = val[k - 1]
            var nj: [V]?V = None
            var nv: [V]T = identity
            for v in V:
                if let m = pj[v]:
                    nj[v] = pj[m]
                    nv[v] = op(pv[v], pv[m])
            jmp.push(nj)
            val.push(nv)
            k += 1
        Lift(op, identity, up, dep, jmp, val)

    /// Lifting over any partial successor function, such as "the next interval a greedy picks":
    /// `Lift.of(i => first_from(iv[i].r))`. `None` ends a chain; the links must form a forest.
    /// Edge values are `()`, so `climb` and `ancestor` are what it is for.
    ///
    /// cost: O(n log n) time and memory, n calls of `next`
    /// traps: when the links have a cycle
    /// example: `domain I = 0..4; let l = Lift<I, ()>.of(i => if i < 3: Some(I(i + 1)) else: None); l.ancestor(0, 3) == Some(3) and l.depth(0) == 3`
    pub fn of(next: V -> ?V) -> Lift<V, ()>:
        var par: [V]?(V, ()) = None
        for v in V:
            if let p = next(v): par[v] = Some((p, ()))
        Lift.new(par, (a, b) => a, ())

    /// Climbs from `v` while the next vertex up satisfies `ok`, which must hold on a prefix of
    /// every root path: how many edges were climbed and where it stopped. Greedy "jump while the
    /// next one still fits" counts are `Lift.of(next).climb(v, ok).steps`.
    ///
    /// cost: O(log n) calls of `ok`
    /// traps: never
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.climb(2, u => u != 0) == (steps: 1, at: 1) and l.climb(2, u => false) == (steps: 0, at: 2)`
    pub fn climb(self, v: V, ok: V -> Bool) -> (steps: Int, at: V):
        var at = v
        var steps = 0
        for k in (0..jmp.len).rev:
            if let w = jmp[k][at]:
                if ok(w):
                    at = w
                    steps += 1 << k
        (steps:, at:)

    /// The parent of `v`, or `None` at a root.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.parent(2) == Some(1) and l.parent(0) == None`
    pub fn parent(self, v: V) -> ?V = up[v]

    /// The number of edges from `v` up to its root.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.depth(0) == 0 and l.depth(3) == 2`
    pub fn depth(self, v: V) -> Int = dep[v]

    /// The ancestor `k` edges above `v` (`v` for `k == 0`), or `None` if that is above the root.
    ///
    /// cost: O(log n)
    /// traps: when `k` is negative
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.ancestor(3, 2) == Some(0) and l.ancestor(3, 3) == None`
    pub fn ancestor(self, v: V, k: Int) -> ?V:
        assert k >= 0, "`ancestor` needs `k >= 0`"
        if k > dep[v]: return None
        var at = v
        for b in 0..jmp.len:
            if ((k >> b) & 1) == 1: at = jmp[b][at]!
        Some(at)

    /// The lowest common ancestor of `a` and `b`, or `None` in different trees.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.lca(2, 3) == Some(1)`
    pub fn lca(self, a: V, b: V) -> ?V:
        var x = a
        var y = b
        if dep[x] < dep[y]:
            let t = x
            x = y
            y = t
        x = self.ancestor(x, dep[x] - dep[y])!
        if x == y: return Some(x)
        for k in (0..jmp.len).rev:
            if let a = jmp[k][x]:
                let b = jmp[k][y]!
                if a != b:
                    x = a
                    y = b
        up[x]

    /// The fold with `op` of the edge values from `v` up to its ancestor `u`, lowest first;
    /// `identity` when `u == v`.
    ///
    /// cost: O(log n) calls of `op`
    /// traps: when `u` is not an ancestor of `v`
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.fold(2, up_to: 1) == 2 and l.fold(2, up_to: 0) == 7 and l.fold(2, up_to: 2) == 0`
    pub fn fold(self, v: V, up_to u: V) -> T:
        let d = dep[v] - dep[u]
        assert d >= 0 and self.ancestor(v, d) == Some(u), "`fold`: `u` is not an ancestor of `v`"
        var acc = identity
        var at = v
        for b in 0..jmp.len:
            if ((d >> b) & 1) == 1:
                acc = op(acc, val[b][at])
                at = jmp[b][at]!
        acc

    /// The highest ancestor `u` of `v` with `ok(self.fold(v, up_to: u))`, or `v` itself; `ok` must
    /// turn false at most once on the way up, and holds for `identity`.
    ///
    /// cost: O(log n) calls of `ok` and `op`
    /// traps: never
    /// example: `domain V = 0..4; let par: [V]?(V, Int) = [None, Some((0, 5)), Some((1, 2)), Some((1, 7))]; let l = Lift(par, op: (x, y) => x + y, identity: 0); l.climb_while(2, s => s <= 5) == 1 and l.climb_while(2, s => s <= 7) == 0`
    pub fn climb_while(self, v: V, ok: T -> Bool) -> V:
        var at = v
        var acc = identity
        for k in (0..jmp.len).rev:
            if let w = jmp[k][at]:
                let nx = op(acc, val[k][at])
                if ok(nx):
                    acc = nx
                    at = w
        at

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
    /// The number of vertices in the splay subtree.
    cnt: [V]0..=V.count
    stk: List<V>

    /// `V.count` single-vertex trees holding `a`, or the identity of `op` everywhere when `a` is
    /// absent.
    ///
    /// cost: O(n)
    /// empty: `a: None` fills every vertex with the identity
    /// traps: never
    /// example: `domain V = 0..3; var lc = LinkCut<V, Int>(+); lc.get(2) == 0 and not lc.connected(0, 1)`
    pub fn new<M>(op: M, a: ?[V]T = None) -> LinkCut<V, T>:
        let m = Monoid<T>.of(op)
        let val = if let init = a: init else: [V] _ => m.id
        LinkCut(
            m,
            [V] _ => None,
            [V] _ => None,
            [V] _ => None,
            [V] _ => false,
            val,
            val,
            [V] _ => 1,
            [],
        )

    fn top(self, x: V) -> Bool:
        if let p = pa[x]: lc[p] != Some(x) and rc[p] != Some(x) else: true

    fn turn(mut self, x: V):
        let r = agg[x].rev()
        agg[x] = r
        flip[x] = not flip[x]

    fn push(mut self, x: V):
        if flip[x]:
            let l = lc[x]
            lc[x] = rc[x]
            rc[x] = l
            if let c = lc[x]: self.turn(c)
            if let c = rc[x]: self.turn(c)
            flip[x] = false

    fn pull(mut self, x: V):
        var s = val[x]
        var k = 1
        if let c = lc[x]:
            s = m.op(agg[c], s)
            k += cnt[c]
        if let c = rc[x]:
            s = m.op(s, agg[c])
            k += cnt[c]
        agg[x] = s
        cnt[x] = k

    fn rotate(mut self, x: V):
        let p = pa[x]!
        let g = pa[p]
        let p_top = self.top(p)
        if lc[p] == Some(x):
            lc[p] = rc[x]
            if let c = rc[x]: pa[c] = Some(p)
            rc[x] = Some(p)
        else:
            rc[p] = lc[x]
            if let c = lc[x]: pa[c] = Some(p)
            lc[x] = Some(p)
        pa[p] = Some(x)
        pa[x] = g
        if not p_top, let q = g:
            if lc[q] == Some(p): lc[q] = Some(x) else: rc[q] = Some(x)
        self.pull(p)
        self.pull(x)

    fn splay(mut self, x: V):
        stk.clear()
        var y = x
        stk.push(y)
        while not self.top(y):
            y = pa[y]!
            stk.push(y)
        while let z = stk.pop(): self.push(z)
        while not self.top(x):
            let p = pa[x]!
            if not self.top(p):
                let g = pa[p]!
                if (lc[g] == Some(p)) == (lc[p] == Some(x)): self.rotate(p) else: self.rotate(x)
            self.rotate(x)

    // Makes root -> x the preferred path with x at the root of its splay tree; returns the
    // last vertex where the walk joined the old preferred path (the lca with the previous access).
    fn access(mut self, x: V) -> V:
        var last: ?V = None
        var joined = x
        var y: ?V = Some(x)
        while let u = y:
            self.splay(u)
            rc[u] = last
            self.pull(u)
            last = Some(u)
            joined = u
            y = pa[u]
        self.splay(x)
        joined

    /// Makes `v` the root of its tree.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.make_root(0); lc.root_of(2) == 0`
    pub fn make_root(mut self, v: V):
        let _ = self.access(v)
        self.turn(v)

    /// The root of the tree holding `v`: the last vertex made root there, by `make_root`, `link`,
    /// `lca` or `fold_path`.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.root_of(0) == 2 and lc.root_of(3) == 3`
    pub fn root_of(mut self, v: V) -> V:
        let _ = self.access(v)
        var x = v
        self.push(x)
        while let c = lc[x]:
            x = c
            self.push(x)
        self.splay(x)
        x

    /// Whether `u` and `v` are in the same tree.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.connected(0, 2) and not lc.connected(0, 3)`
    pub fn connected(mut self, u: V, v: V) -> Bool = self.root_of(u) == self.root_of(v)

    /// Adds the edge `u`-`v`: `u` becomes the root of its tree, which hangs below `v`, so the
    /// joined tree keeps `v`'s root.
    ///
    /// cost: amortized O(log n), with one `root_of` to check the trees differ
    /// traps: when `u` and `v` are already in one tree
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.link(3, 0); lc.connected(3, 2) and lc.root_of(3) == 2`
    pub fn link(mut self, u: V, v: V):
        self.make_root(u)
        assert self.root_of(v) != u, "link joins two trees; these vertices are already connected"
        pa[u] = Some(v)

    /// Removes the edge `u`-`v`.
    ///
    /// cost: amortized O(log n)
    /// traps: when there is no edge `u`-`v`
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.cut(1, 2); not lc.connected(0, 2) and lc.connected(0, 1)`
    pub fn cut(mut self, u: V, v: V):
        self.make_root(u)
        let _ = self.access(v)
        assert lc[v] == Some(u) and rc[u] == None, "cut needs an edge between these vertices"
        lc[v] = None
        pa[u] = None
        self.pull(v)

    /// The lowest common ancestor of `u` and `v` when their tree is rooted at `root`, which
    /// becomes its root; `None` unless all three are connected.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.lca(0, 2, 1) == Some(1) and lc.root_of(0) == 1 and lc.lca(0, 3, 1) == None`
    pub fn lca(mut self, u: V, v: V, root: V) -> ?V:
        self.make_root(root)
        if self.root_of(u) != root or self.root_of(v) != root: return None
        let _ = self.access(u)
        Some(self.access(v))

    /// The value at `v`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.get(2) == 3`
    pub fn get(self, v: V) -> T = val[v]

    /// Sets the value at `v` to `x`.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.set(2, 10); lc.get(2) == 10 and lc.fold_path(0, 2) == 13`
    pub fn set(mut self, v: V, x: T):
        let _ = self.access(v)
        val[v] = x
        self.pull(v)

    /// The fold of the values on the path from `u` to `v`, in that order; `u` becomes the root.
    ///
    /// cost: amortized O(log n)
    /// traps: when `u` and `v` are in different trees
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.fold_path(0, 2) == 6 and lc.root_of(2) == 0 and lc.fold_path(1, 1) == 2`
    pub fn fold_path(mut self, u: V, v: V) -> T:
        self.make_root(u)
        let _ = self.access(v)
        agg[v]

    /// The number of edges between `a` and `b`; `None` across trees.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.dist(0, 2) == Some(2) and lc.dist(0, 3) == None`
    pub fn dist(mut self, a: V, b: V) -> ?Int:
        if not self.connected(a, b): return None
        self.make_root(a)
        let _ = self.access(b)
        Some(cnt[b] - 1)

    /// The `k`-th vertex on the path from `a` to `b` (`a` is the 0th); `None` past `b` or across
    /// trees.
    ///
    /// cost: amortized O(log n)
    /// traps: when `k` is negative
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var lc = LinkCut(+, a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.jump(0, 2, 1) == Some(1) and lc.jump(0, 2, 0) == Some(0) and lc.jump(0, 2, 3) == None`
    pub fn jump(mut self, a: V, b: V, k: Int) -> ?V:
        assert k >= 0
        if not self.connected(a, b): return None
        self.make_root(a)
        let _ = self.access(b)
        if k >= cnt[b]: return None
        var x = b
        var r = k
        while true:
            self.push(x)
            let left = if let c = lc[x]: cnt[c] else: 0
            if r < left: x = lc[x]!
            elif r == left: break
            else:
                r -= left + 1
                x = rc[x]!
        self.splay(x)
        Some(x)

/// The preferred-path half of a link-cut tree over a fixed rooted tree: every node starts as its
/// own path. Counting "how many paths lie between v and the root" (colours painted along root
/// paths, access counts) reads off the switches each `access` makes.
pub struct PreferredPaths<V>:
    lc: [V]?V
    rc: [V]?V
    pa: [V]?V
    /// The top vertex of each splay subtree's path segment: its leftmost node.
    hd: [V]V
    stk: List<V>

    /// The preferred paths of `t` with every vertex its own path.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); var pp = PreferredPaths(t); pp.access(0).count() == 0`
    pub fn new<P>(t: Rooted<V, P>) -> PreferredPaths<V> =
        PreferredPaths([V] _ => None, [V] _ => None, [V] v => t.parent(v), [V] v => v, [])

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
    pub fn access(mut self, v: V) -> yields (lost: ?V, gained: V):
        var last: ?V = None
        var y: ?V = Some(v)
        while let u = y:
            self.splay(u)
            if let g = last:
                var lost: ?V = None
                if let c = rc[u]: lost = Some(hd[c])
                yield (lost:, gained: hd[g])
            rc[u] = last
            self.pull(u)
            last = Some(u)
            y = pa[u]
        self.splay(v)

    fn top(self, x: V) -> Bool:
        if let p = pa[x]: lc[p] != Some(x) and rc[p] != Some(x) else: true

    fn pull(mut self, x: V):
        hd[x] = if let c = lc[x]: hd[c] else: x

    fn rotate(mut self, x: V):
        let p = pa[x]!
        let g = pa[p]
        let p_top = self.top(p)
        if lc[p] == Some(x):
            lc[p] = rc[x]
            if let c = rc[x]: pa[c] = Some(p)
            rc[x] = Some(p)
        else:
            rc[p] = lc[x]
            if let c = lc[x]: pa[c] = Some(p)
            lc[x] = Some(p)
        pa[p] = Some(x)
        pa[x] = g
        if not p_top, let q = g:
            if lc[q] == Some(p): lc[q] = Some(x) else: rc[q] = Some(x)
        self.pull(p)
        self.pull(x)

    fn splay(mut self, x: V):
        while not self.top(x):
            let p = pa[x]!
            if not self.top(p):
                let g = pa[p]!
                if (lc[g] == Some(p)) == (lc[p] == Some(x)): self.rotate(p) else: self.rotate(x)
            self.rotate(x)

/// `LinkCut` with updates along a path: `LazyLinkCut(act, a)` with an add `Action` adds to every value on
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

    /// `V.count` single-vertex trees holding `a`, or the identity of `act.over` everywhere when `a`
    /// is absent.
    ///
    /// cost: O(n)
    /// empty: `a: None` fills every vertex with the identity
    /// traps: never
    /// example: `domain V = 0..3; var lc = LazyLinkCut<V, Int, Int>(Action<Int, Int>(apply: (d, x, n) => x + d * n)); lc.get(1) == 0`
    pub fn new(act: Action<T, F>, a: ?[V]T = None) -> LazyLinkCut<V, T, F>:
        let val = if let init = a: init else: [V] _ => act.over.id
        LazyLinkCut(
            act,
            [V] _ => None,
            [V] _ => None,
            [V] _ => None,
            [V] _ => false,
            val,
            val,
            [V] _ => 1,
            [V] _ => act.id,
            [],
        )

    fn top(self, x: V) -> Bool:
        if let p = pa[x]: lc[p] != Some(x) and rc[p] != Some(x) else: true

    fn turn(mut self, x: V):
        let r = agg[x].rev()
        agg[x] = r
        flip[x] = not flip[x]

    fn put(mut self, x: V, f: F):
        val[x] = act.apply(f, val[x], 1)
        agg[x] = act.apply(f, agg[x], cnt[x])
        lz[x] = act.compose(f, lz[x])

    fn push(mut self, x: V):
        if flip[x]:
            let l = lc[x]
            lc[x] = rc[x]
            rc[x] = l
            if let c = lc[x]: self.turn(c)
            if let c = rc[x]: self.turn(c)
            flip[x] = false
        let f = lz[x]
        if let c = lc[x]: self.put(c, f)
        if let c = rc[x]: self.put(c, f)
        lz[x] = act.id

    fn pull(mut self, x: V):
        var s = val[x]
        var k = 1
        if let c = lc[x]:
            s = act.over.op(agg[c], s)
            k += cnt[c]
        if let c = rc[x]:
            s = act.over.op(s, agg[c])
            k += cnt[c]
        agg[x] = s
        cnt[x] = k

    fn rotate(mut self, x: V):
        let p = pa[x]!
        let g = pa[p]
        let p_top = self.top(p)
        if lc[p] == Some(x):
            lc[p] = rc[x]
            if let c = rc[x]: pa[c] = Some(p)
            rc[x] = Some(p)
        else:
            rc[p] = lc[x]
            if let c = lc[x]: pa[c] = Some(p)
            lc[x] = Some(p)
        pa[p] = Some(x)
        pa[x] = g
        if not p_top, let q = g:
            if lc[q] == Some(p): lc[q] = Some(x) else: rc[q] = Some(x)
        self.pull(p)
        self.pull(x)

    fn splay(mut self, x: V):
        stk.clear()
        var y = x
        stk.push(y)
        while not self.top(y):
            y = pa[y]!
            stk.push(y)
        while let z = stk.pop(): self.push(z)
        while not self.top(x):
            let p = pa[x]!
            if not self.top(p):
                let g = pa[p]!
                if (lc[g] == Some(p)) == (lc[p] == Some(x)): self.rotate(p) else: self.rotate(x)
            self.rotate(x)

    fn access(mut self, x: V) -> V:
        var last: ?V = None
        var joined = x
        var y: ?V = Some(x)
        while let u = y:
            self.splay(u)
            rc[u] = last
            self.pull(u)
            last = Some(u)
            joined = u
            y = pa[u]
        self.splay(x)
        joined

    /// Makes `v` the root of its tree.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.make_root(0); lc.root_of(2) == 0`
    pub fn make_root(mut self, v: V):
        let _ = self.access(v)
        self.turn(v)

    /// The root of the tree holding `v`, as for `LinkCut.root_of`.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.root_of(0) == 2`
    pub fn root_of(mut self, v: V) -> V:
        let _ = self.access(v)
        var x = v
        self.push(x)
        while let c = lc[x]:
            x = c
            self.push(x)
        self.splay(x)
        x

    /// Whether `u` and `v` are in the same tree.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.connected(0, 2)`
    pub fn connected(mut self, u: V, v: V) -> Bool = self.root_of(u) == self.root_of(v)

    /// Adds the edge `u`-`v` as `LinkCut.link` does: `u`'s tree hangs below `v`.
    ///
    /// cost: amortized O(log n), with one `root_of` to check the trees differ
    /// traps: when `u` and `v` are already in one tree
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.connected(0, 2) and lc.dist(0, 2) == Some(2)`
    pub fn link(mut self, u: V, v: V):
        self.make_root(u)
        assert self.root_of(v) != u, "link joins two trees; these vertices are already connected"
        pa[u] = Some(v)

    /// Removes the edge `u`-`v`.
    ///
    /// cost: amortized O(log n)
    /// traps: when there is no edge `u`-`v`
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.cut(0, 1); not lc.connected(0, 2) and lc.connected(1, 2)`
    pub fn cut(mut self, u: V, v: V):
        self.make_root(u)
        let _ = self.access(v)
        assert lc[v] == Some(u) and rc[u] == None, "cut needs an edge between these vertices"
        lc[v] = None
        pa[u] = None
        self.pull(v)

    /// The lowest common ancestor of `u` and `v` when their tree is rooted at `root`, which
    /// becomes its root; `None` unless all three are connected.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.lca(0, 2, 1) == Some(1)`
    pub fn lca(mut self, u: V, v: V, root: V) -> ?V:
        self.make_root(root)
        if self.root_of(u) != root or self.root_of(v) != root: return None
        let _ = self.access(u)
        Some(self.access(v))

    /// The value at `v`, with every update along paths through it applied.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.apply_path(0, 1, 10); lc.get(1) == 12 and lc.get(2) == 3`
    pub fn get(mut self, v: V) -> T:
        let _ = self.access(v)
        val[v]

    /// Sets the value at `v` to `x`, replacing every earlier update there.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.set(1, 7); lc.fold_path(0, 2) == 11`
    pub fn set(mut self, v: V, x: T):
        let _ = self.access(v)
        val[v] = x
        self.pull(v)

    /// The fold of the values on the path from `u` to `v`, in that order; `u` becomes the root.
    ///
    /// cost: amortized O(log n)
    /// traps: when `u` and `v` are in different trees
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.fold_path(0, 2) == 6`
    pub fn fold_path(mut self, u: V, v: V) -> T:
        self.make_root(u)
        let _ = self.access(v)
        agg[v]

    /// Applies `f` to every value on the path from `u` to `v`; `u` becomes the root.
    ///
    /// cost: amortized O(log n)
    /// traps: when `u` and `v` are in different trees
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.apply_path(0, 1, 10); lc.fold_path(0, 2) == 26`
    pub fn apply_path(mut self, u: V, v: V, f: F):
        self.make_root(u)
        let _ = self.access(v)
        self.put(v, f)

    /// The number of edges between `a` and `b`; `None` across trees.
    ///
    /// cost: amortized O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.dist(0, 2) == Some(2) and lc.dist(1, 1) == Some(0)`
    pub fn dist(mut self, a: V, b: V) -> ?Int:
        if not self.connected(a, b): return None
        self.make_root(a)
        let _ = self.access(b)
        Some(cnt[b] - 1)

    /// The `k`-th vertex on the path from `a` to `b` (`a` is the 0th); `None` past `b` or across
    /// trees.
    ///
    /// cost: amortized O(log n)
    /// traps: when `k` is negative
    /// example: `domain V = 0..3; let vals: [V]Int = [1, 2, 3]; var lc = LazyLinkCut(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); lc.link(0, 1); lc.link(1, 2); lc.jump(2, 0, 1) == Some(1) and lc.jump(2, 0, 3) == None`
    pub fn jump(mut self, a: V, b: V, k: Int) -> ?V:
        assert k >= 0
        if not self.connected(a, b): return None
        self.make_root(a)
        let _ = self.access(b)
        if k >= cnt[b]: return None
        var x = b
        var r = k
        while true:
            self.push(x)
            let left = if let c = lc[x]: cnt[c] else: 0
            if r < left: x = lc[x]!
            elif r == left: break
            else:
                r -= left + 1
                x = rc[x]!
        self.splay(x)
        Some(x)

@test
fn link_cut_follows_a_changing_forest():
    domain V = 1..=6
    var lct = LinkCut<V, Int>.new(+, [V] v => v.to_idx + 1)
    lct.link(V(1), V(2))
    lct.link(V(2), V(3))
    lct.link(V(4), V(3))
    assert lct.connected(V(1), V(4)) and not lct.connected(V(1), V(5))
    assert lct.fold_path(V(1), V(4)) == 10
    assert lct.lca(V(3), V(4), V(1)) == Some(V(3)) and lct.lca(V(2), V(4), V(1)) == Some(V(2))
    assert lct.lca(V(1), V(4), V(3)) == Some(V(3)) and lct.lca(V(1), V(5), V(1)) == None
    lct.cut(V(2), V(3))
    assert not lct.connected(V(1), V(4)) and lct.fold_path(V(3), V(4)) == 7
    lct.set(V(4), 10)
    lct.link(V(5), V(4))
    assert lct.fold_path(V(5), V(3)) == 18 and lct.root_of(V(3)) == V(5)

@test
fn link_cut_matches_a_brute_forest_with_ordered_folds():
    domain V = 1..=9
    var rng = Rng(7)
    let w = [V] _ => rng.int(-5, 5)
    var lct = LinkCut<V, MaxSub>.new(MaxSub.monoid(), [V] v => MaxSub.of(w[v]))
    var adj: [V]List<V> = []
    for _ in 0..400:
        let u = V(rng.int(1, 9))
        let v = V(rng.int(1, 9))
        var from: [V]?V = None
        var seen: [V]Bool = false
        seen[u] = true
        var st = [u]
        while let x = st.pop():
            for y in adj[x]:
                if not seen[y]:
                    seen[y] = true
                    from[y] = Some(x)
                    st.push(y)
        assert lct.connected(u, v) == seen[v]
        if seen[v] and u != v:
            var acc = MaxSub.of(w[v])
            var x = v
            while let p = from[x]:
                acc = MaxSub.of(w[p]) + acc
                x = p
            assert lct.fold_path(u, v) == acc
            if rng.int(0, 2) == 0 and from[v] != None:
                let p = from[v]!
                lct.cut(v, p)
                adj[v] = [y for y in adj[v] if y != p]
                adj[p] = [y for y in adj[p] if y != v]
        elif not seen[v]:
            lct.link(u, v)
            for (x, y) in [(u, v), (v, u)]: adj[x].push(y)

@test
fn lazy_link_cut_matches_a_brute_forest():
    domain V = 1..=9
    var rng = Rng(11)
    var w = [V] _ => rng.int(-5, 5)
    let add = Action<Int, Int>(
        0,
        (f, g) => f + g,
        (d, x, n) => x + d * n,
        Monoid<Int>(0, (a, b) => a + b),
    )
    var lct = LazyLinkCut<V, Int, Int>.new(add, Some(w))
    var adj: [V]List<V> = []
    for _ in 0..400:
        let u = V(rng.int(1, 9))
        let v = V(rng.int(1, 9))
        var from: [V]?V = None
        var seen: [V]Bool = false
        seen[u] = true
        var st = [u]
        while let x = st.pop():
            for y in adj[x]:
                if not seen[y]:
                    seen[y] = true
                    from[y] = Some(x)
                    st.push(y)
        assert lct.connected(u, v) == seen[v]
        if seen[v]:
            var path = [v]
            var x = v
            while let p = from[x]:
                path.push(p)
                x = p
            let ab = path.rev.collect()
            assert lct.dist(u, v) == Some(ab.len - 1)
            let k = rng.int(0, ab.len)
            assert lct.jump(u, v, k) == (if k < ab.len: Some(ab[k]) else: None)
            let op = rng.int(0, 3)
            if op == 0:
                let d = rng.int(-3, 3)
                lct.apply_path(u, v, d)
                for y in ab: w[y] += d
            elif op == 1 and from[v] != None:
                let p = from[v]!
                lct.cut(v, p)
                adj[v] = [y for y in adj[v] if y != p]
                adj[p] = [y for y in adj[p] if y != v]
            elif op == 2:
                let s = rng.int(-5, 5)
                lct.set(v, s)
                w[v] = s
            assert lct.fold_path(u, v) == sum([w[y] for y in ab]) or op == 1
            assert lct.get(u) == w[u]
        else:
            assert lct.dist(u, v) == None and lct.jump(u, v, 0) == None
            lct.link(u, v)
            for (x, y) in [(u, v), (v, u)]: adj[x].push(y)

@test
fn lazy_link_cut_keeps_path_order_under_assignment():
    domain V = 1..=6
    let paint = Action<MaxSub, ?Int>(
        None,
        (f, g) => f ?? g,
        (f, x, k) => if let c = f: (
            if c > 0: MaxSub.of(c).scale(k) else: MaxSub(true, c, c * k, c, c)
        ) else: x,
        over: MaxSub.monoid(),
    )
    var lct = LazyLinkCut<V, MaxSub, ?Int>.new(
        paint,
        [V] v => MaxSub.of([3, -1, 4, -1, 5, -9][v.to_idx]),
    )
    for v in 2..=6: lct.link(V(v), V(v - 1))
    lct.apply_path(V(2), V(3), Some(-2))
    assert lct.fold_path(V(1), V(6)).best == 5 and lct.fold_path(
        V(6),
        V(1),
    ).total == 3 - 2 - 2 - 1 + 5 - 9
    lct.apply_path(V(5), V(4), Some(2))
    assert lct.fold_path(V(1), V(5)).best == 4 and lct.fold_path(V(4), V(2)).pre == 2
    assert lct.get(V(3)) == MaxSub.of(-2) and lct.lca(V(2), V(6), V(4)) == Some(V(4))

// The vertices `s` reaches, without the edge `block`.
fn brute_side<V>(adj: [V]List<V>, s: V, block: ?(V, V)) -> List<V>:
    var seen: [V]Bool = false
    var q = [s]
    seen[s] = true
    var i = 0
    while i < q.len:
        let u = q[i]
        i += 1
        for w in adj[u]:
            if block == Some((u, w)) or block == Some((w, u)): continue
            if not seen[w]:
                seen[w] = true
                q.push(w)
    q

fn drop_one<V>(mut l: List<V>, x: V):
    var k = 0
    while l[k] != x: k += 1
    l[k] = l.last!
    let _ = l.pop()

@test
fn euler_tour_trees_match_a_brute_forest():
    domain V = 0..9
    for _ in 0..30:
        var w: [V]Int = [V] _ => rand(-5, 5)
        var et = EulerTourTree(+, a: Some(w))
        let add = Action<Int, Int>(
            0,
            (f, g) => f + g,
            (d, x, n) => x + d * n,
            Monoid<Int>(0, (a, b) => a + b),
        )
        var lz = LazyEulerTourTree<V, Int, Int>.new(add, Some(w))
        var adj: [V]List<V> = [V] _ => []
        var edges: List<(V, V)> = []
        for _ in 0..200:
            let op = rand(0, 5)
            let u = V.nth(rand(0, V.count - 1))
            let v = V.nth(rand(0, V.count - 1))
            let cu = brute_side(adj, u, None)
            if op == 0 and not cu.contains(v):
                et.link(u, v)
                lz.link(u, v)
                adj[u].push(v)
                adj[v].push(u)
                edges.push((u, v))
            elif op == 1 and not edges.is_empty:
                let k = rand(0, edges.len - 1)
                let (a, b) = edges[k]
                edges[k] = edges.last!
                let _ = edges.pop()
                if rand(0, 1) == 0: et.cut(a, b) else: et.cut(b, a)
                lz.cut(a, b)
                drop_one(mut adj[a], b)
                drop_one(mut adj[b], a)
            elif op == 2:
                let x = rand(-5, 5)
                et.set(u, x)
                lz.set(u, x)
                w[u] = x
            elif op == 3 and not edges.is_empty:
                let (a, b) = edges[rand(0, edges.len - 1)]
                let side = brute_side(adj, a, Some((a, b)))
                let d = rand(-3, 3)
                assert et.fold_sub(a, b) == sum([w[y] for y in side])
                lz.apply_sub(a, b, d)
                for y in side:
                    w[y] += d
                et = EulerTourTree(+, a: Some(w))
                for (p, q) in edges: et.link(p, q)
                assert lz.fold_sub(a, b) == sum([w[y] for y in side])
            elif op == 4:
                let d = rand(-3, 3)
                lz.apply_tree(u, d)
                for y in cu:
                    w[y] += d
                    et.set(y, w[y])
            assert et.connected(u, v) == brute_side(adj, u, None).contains(v)
            assert lz.connected(u, v) == et.connected(u, v)
            let cu2 = brute_side(adj, u, None)
            assert et.len(u) == cu2.len and lz.len(u) == cu2.len
            assert et.fold_tree(u) == sum([w[y] for y in cu2])
            assert lz.fold_tree(u) == sum([w[y] for y in cu2])
            assert et.get(v) == w[v] and lz.get(v) == w[v]

/// How a permutation-tree node joins its two children `l` (earlier positions) and `r`.
/// `Incr`: `r`'s values sit just above `l`'s; `Decr`: just below; `Partial`: the union is
/// not an interval on its own and only closes further up; `Full`: `r` fills the gaps of a
/// `Partial` `l`, closing an interval in neither order.
pub enum PermKind = Leaf | Incr | Decr | Partial | Full
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
    pub fn new(p: [I]V) -> PermTree<I, V, _>:
        let n = I.count
        assert n > 0 and n == V.count, "`PermTree` needs a nonempty bijection"
        var seen: [V]Bool = false
        for i in I:
            assert not seen[p[i]], "`PermTree` needs a bijection"
            seen[p[i]] = true
        let val = [p[i].ord for i in I]
        var b = PermBuild(
            [PermKind.Leaf for _ in 0..2 * n - 1],
            [0 for _ in 0..2 * n - 1],
            [0 for _ in 0..2 * n - 1],
            [k / 2 for k in 0..2 * n - 1],
            [k / 2 for k in 0..2 * n - 1],
            [val[k / 2] for k in 0..2 * n - 1],
            [val[k / 2] for k in 0..2 * n - 1],
            1,
        )
        // `gap[L]` is `max - min - (i - L)` of `val[L..=i]`: 0 exactly when that run is an interval
        var gap = SegTree(min, [0 for _ in 0..n], act: add)
        var (mx, mn, st) = (List<Int>(), List<Int>(), List<Int>())
        for i in 0..n:
            if i > 0: gap.apply(0..i, -1)
            while mx.len > 0 and val[mx.last!] < val[i]:
                let j = mx.pop()!
                gap.apply((if mx.len > 0: mx.last! + 1 else: 0)..j + 1, val[i] - val[j])
            mx.push(i)
            while mn.len > 0 and val[mn.last!] > val[i]:
                let j = mn.pop()!
                gap.apply((if mn.len > 0: mn.last! + 1 else: 0)..j + 1, val[j] - val[i])
            mn.push(i)
            var x = 2 * i
            var more = true
            while more:
                more = false
                if let t = st.last:
                    if b.hi[t] + 1 == b.lo[x] or b.hi[x] + 1 == b.lo[t]:
                        st.pop()
                        x = b.join(t, x, if b.hi[t] < b.lo[x]: Incr else: Decr)
                        more = true
                if not more and b.first[x] > 0 and gap.fold(0..b.first[x]) == 0:
                    var (lo, hi, cnt) = (b.lo[x], b.hi[x], b.last[x] - b.first[x] + 1)
                    var k = st.len
                    var open = true
                    while open:
                        k -= 1
                        let t = st[k]
                        (lo, hi, cnt) = (
                            min(lo, b.lo[t]),
                            max(hi, b.hi[t]),
                            cnt + b.last[t] - b.first[t] + 1,
                        )
                        open = hi - lo + 1 != cnt
                    var acc = st[k]
                    for j in k + 1..st.len: acc = b.join(acc, st[j], Partial)
                    while st.len > k: st.pop()
                    x = b.join(acc, x, Full)
                    more = true
            st.push(x)
        assert st.len == 1
        domain N = 0..2 * I.count - 1
        PermTree<I, V, N>(
            N(st[0]),
            [N] x => b.kind[x.ord],
            [N] x => N(b.l[x.ord]),
            [N] x => N(b.r[x.ord]),
            [N] x => I.nth(b.first[x.ord]),
            [N] x => I.nth(b.last[x.ord]),
            [N] x => V.nth(b.lo[x.ord]),
            [N] x => V.nth(b.hi[x.ord]),
        )

    /// The view of `v` to match on (decision 132): `Leaf(i)` or `Inner(l, r, kind)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain I = 0..3; let p: [I]I = [1, 0, 2]; let pt = PermTree(p); pt.node(pt.leaf(1)) == PermNode.Leaf(1) and pt.node(pt.root) == PermNode.Inner(pt.left(pt.root), pt.leaf(2), PermKind.Incr)`
    pub fn node(self, v: Node) -> PermNode<I, Node>:
        if kind[v] == Leaf: PermNode.Leaf(first[v]) else: PermNode.Inner(l[v], r[v], kind[v])

    /// The leaf of position `i`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain I = 0..3; let p: [I]I = [1, 0, 2]; let pt = PermTree(p); pt.span(pt.leaf(2)) == 2..3 and pt.kind[pt.leaf(2)] == PermKind.Leaf`
    pub fn leaf(self, i: I) -> Node = Node(2 * i.ord)

    /// The earlier child of the inner node `v`.
    ///
    /// cost: O(1)
    /// traps: when `v` is a leaf
    /// example: `domain I = 0..3; let p: [I]I = [1, 0, 2]; let pt = PermTree(p); pt.span(pt.left(pt.root)) == 0..2`
    pub fn left(self, v: Node) -> Node:
        assert kind[v] != Leaf, "a leaf has no children"
        l[v]

    /// The later child of the inner node `v`.
    ///
    /// cost: O(1)
    /// traps: when `v` is a leaf
    /// example: `domain I = 0..3; let p: [I]I = [1, 0, 2]; let pt = PermTree(p); pt.right(pt.root) == pt.leaf(2)`
    pub fn right(self, v: Node) -> Node:
        assert kind[v] != Leaf, "a leaf has no children"
        r[v]

    /// The positions `v` covers, a nonempty range of `I`.
    ///
    /// cost: O(1)
    /// empty: never empty
    /// order: increasing positions
    /// traps: never
    /// example: `domain I = 0..3; let p: [I]I = [1, 0, 2]; let pt = PermTree(p); pt.span(pt.root) == 0..3`
    pub fn span(self, v: Node) -> Range<I> = first[v].l..last[v].r

    /// From the least to the greatest value `v` covers: exactly its values unless `v` is `Partial`.
    ///
    /// cost: O(1)
    /// empty: never empty
    /// order: increasing values
    /// traps: never
    /// example: `domain I = 0..3; let p: [I]I = [1, 0, 2]; let pt = PermTree(p); pt.vals(pt.left(pt.root)) == 0..2 and pt.vals(pt.leaf(0)) == 1..2`
    pub fn vals(self, v: Node) -> Range<V> = lo[v].l..hi[v].r

/// `PermTree.new`'s nodes by ordinal while it builds: leaves at even ordinals, inner nodes at odd.
struct PermBuild:
    var kind: List<PermKind>
    var l: List<Int>
    var r: List<Int>
    var first: List<Int>
    var last: List<Int>
    var lo: List<Int>
    var hi: List<Int>
    var next: Int

    fn join(mut self, a: Int, b: Int, k: PermKind) -> Int:
        let x = next
        next += 2
        kind[x] = k
        l[x] = a
        r[x] = b
        first[x] = first[a]
        last[x] = last[b]
        lo[x] = min(lo[a], lo[b])
        hi[x] = max(hi[a], hi[b])
        x

@test
fn perm_tree_nodes_cover_intervals_of_values():
    for n in 1..=9:
        for _ in 0..20:
            domain I = 0..n
            var vals = [i for i in I]
            shuffle(mut vals)
            let p: [I]I = [I] i => vals[i.ord]
            let pt = PermTree(p)
            assert pt.span(pt.root) == I.first!.l..I.last!.r
            var seen = 0
            var st = [pt.root]
            while let v = st.pop():
                seen += 1
                let sp = pt.span(v)
                let got = [p[i].ord for i in sp]
                let (lo, hi) = (got.min()!, got.max()!)
                assert pt.vals(v) == I.nth(lo).l..I.nth(hi).r
                let interval = hi - lo + 1 == got.len
                assert interval == (pt.kind[v] != PermKind.Partial)
                if got.len == 1:
                    let i = [j for j in sp][0]
                    assert pt.kind[v] == PermKind.Leaf and pt.leaf(i) == v
                    match pt.node(v):
                        Leaf(j) => assert j == i
                        Inner(_, _, _) => assert false
                else:
                    let (a, b) = (pt.left(v), pt.right(v))
                    match pt.node(v):
                        Leaf(_) => assert false
                        Inner(x, y, k) => assert x == a and y == b and k == pt.kind[v]
                    assert [i for i in pt.span(a)] + [i for i in pt.span(b)] == [i for i in sp]
                    let (va, vb) = ([p[i].ord for i in pt.span(a)], [p[i].ord for i in pt.span(b)])
                    let up = va.max()! + 1 == vb.min()!
                    let down = vb.max()! + 1 == va.min()!
                    match pt.kind[v]:
                        Incr => assert up and pt.kind[b] != PermKind.Incr
                        Decr => assert down and pt.kind[b] != PermKind.Decr
                        Full => assert not up and not down and pt.kind[a] == PermKind.Partial
                        Partial => assert pt.kind[b] != PermKind.Partial
                        Leaf => assert false
                    st.push(a)
                    st.push(b)
            assert seen == 2 * n - 1

@test
fn preferred_paths_switch_like_preferred_children():
    for n in 1..=12:
        domain V = 0..n
        let adj = random_tree(V.first!)
        let r = Rooted(adj, V.first!)
        var pp = PreferredPaths(r)
        var pref: [V]?V = [V] _ => None
        for _ in 0..60:
            let v = V.nth(rand(0, n - 1))
            var want: List<(lost: ?V, gained: V)> = []
            pref[v] = None
            var c = v
            while let p = r.parent(c):
                if pref[p] != Some(c):
                    want.push((lost: pref[p], gained: c))
                    pref[p] = Some(c)
                c = p
            assert pp.access(v).collect() == want

@test
fn perm_tree_of_a_small_permutation():
    domain I = 0..4
    let p: [I]I = [1, 3, 0, 2]
    let pt = PermTree(p)
    let (a, d) = (pt.left(pt.root), pt.right(pt.root))
    assert pt.kind[pt.root] == PermKind.Full and d == pt.leaf(3)
    assert pt.kind[a] == PermKind.Partial and pt.right(a) == pt.leaf(2) and [i.ord for i in pt.span(a)] == [
        0,
        1,
        2,
    ]
    let b = pt.left(a)
    assert pt.kind[b] == PermKind.Partial and pt.left(b) == pt.leaf(0) and pt.right(b) == pt.leaf(1)
    domain J = 0..3
    let q: [J]J = [0, 1, 2]
    let qt = PermTree(q)
    assert qt.kind[qt.root] == PermKind.Incr and qt.right(qt.root) == qt.leaf(2)
    assert qt.kind[qt.left(qt.root)] == PermKind.Incr and [
        i.ord for i in qt.span(qt.left(qt.root))
    ] == [0, 1]
    let s: [J]J = [2, 1, 0]
    let st = PermTree(s)
    assert st.kind[st.root] == PermKind.Decr and st.kind[st.left(st.root)] == PermKind.Decr

/// The vertices of a `Rooted` tree by depth, each depth in preorder, over their own domain
/// `B`: the descendants of a vertex at one depth are a contiguous range of `B`.
pub struct Layers<V, B>:
    at: [B]V
    slot: [V]B
    /// The layer of depth `d` is `start[d]..start[d + 1]` as ordinals of `B`.
    start: List<Int>
    pre: [V]Int
    sz: [V]Int
    dep: [V]Int

    /// The position of `v` among the layers.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ly = t.layers(); ly.at[ly.pos(2)] == 2`
    pub fn pos(self, v: V) -> B = slot[v]

    /// Every vertex at depth `d`, in `t.Pre` order; empty past the deepest or below 0.
    ///
    /// cost: O(1)
    /// empty: for `d` outside `0..=` the greatest depth
    /// order: `t.Pre` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ly = t.layers(); ly.level(2).map(b => ly.at[b]).collect() == [3, 4] and ly.level(3).count() == 0`
    pub fn level(self, d: Int) -> Range<B>:
        if d < 0 or d + 1 >= start.len or start[d] == start[d + 1]: return B.nth(0).l..B.nth(0).l
        B.nth(start[d]).l..B.nth(start[d]).r(start[d + 1] - start[d])

    /// The descendants of `v` at depth `d`, counted from the root as in `level`, by binary search
    /// in `level(d)`; `v` itself when `d == depth(v)`.
    ///
    /// cost: O(log n)
    /// empty: when there are None, including `d < depth(v)`
    /// order: `t.Pre` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ly = t.layers(); ly.below(1, 2).map(b => ly.at[b]).collect() == [3, 4] and ly.below(2, 2).count() == 0`
    pub fn below(self, v: V, d: Int) -> Range<B>:
        if d < dep[v] or d + 1 >= start.len: return B.nth(0).l..B.nth(0).l
        let hi = start[d + 1]
        let i = (start[d]..hi).first_true(k => pre[at[B.nth(k)]] >= pre[v]) ?? hi
        let j = (i..hi).first_true(k => pre[at[B.nth(k)]] >= pre[v] + sz[v]) ?? hi
        span(B.cuts(i), B.cuts(j))

/// The enter and exit events of a `Rooted` tree's `tour` over their own domain `P` of 2n
/// members, for Mo's algorithm on trees.
pub struct Events<V, P>:
    at: [P]Visit<V, V>
    ent: [V]P
    ext: [V]P
    jt: JumpTree<V>

    /// The event that enters `v`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ev = t.events(); let want: Visit<V, V> = Visit.Enter(1, Some(0)); ev.at[ev.enter(1)] == want and ev.enter(1) < ev.enter(3)`
    pub fn enter(self, v: V) -> P = ent[v]

    /// The event that leaves `v`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ev = t.events(); let want: Visit<V, V> = Visit.Exit(3, Some(1)); ev.at[ev.exit(3)] == want and ev.exit(3) < ev.exit(1)`
    pub fn exit(self, v: V) -> P = ext[v]

    /// The events inside `v`'s subtree, `enter(v)` through `exit(v)`.
    ///
    /// cost: O(1)
    /// empty: never empty; 2 * size(v) events
    /// order: `tour` order
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let ev = t.events(); let want: List<Visit<V, V>> = [Visit.Enter(1, Some(0)), Visit.Enter(3, Some(1)), Visit.Exit(3, Some(1)), Visit.Enter(4, Some(1)), Visit.Exit(4, Some(1)), Visit.Exit(1, Some(0))]; ev.sub(1).map(p => ev.at[p]).collect() == want`
    pub fn sub(self, v: V) -> Range<P> = ent[v].l..ext[v].r

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
    pub fn path(self, a: V, b: V) -> (range: Range<P>, extra: ?V):
        let (x, y) = if ent[a] <= ent[b]: (a, b) else: (b, a)
        let c = jt.lca(x, y)!
        if c == x: (range: ent[x].l..ent[y].r, extra: None)
        else: (range: ext[x].l..ent[y].r, extra: Some(c))

/// The long-path decomposition of a `Rooted` tree: each vertex continues the path of its
/// tallest child, so a DP indexed by depth below `v` lives in one shared buffer over `L`,
/// inheriting `deep[v]`'s values is free, and merging the other children costs their heights.
pub struct LongPaths<V, L>:
    /// The child with the tallest subtree; `None` at a leaf.
    deep: [V]?V
    /// The number of edges on the longest path down from `v`.
    height: [V]Int
    pos: [V]L

    /// The slots of `v`, one per depth offset `0..=height[v]`, offset 0 first; `slots(deep[v])`
    /// starts one later in the same buffer.
    ///
    /// cost: O(1)
    /// empty: never empty
    /// order: offset 0 first
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let lp = t.long_paths(); lp.slots(0).count() == 3 and lp.slots(2).count() == 1`
    pub fn slots(self, v: V) -> Range<L> = pos[v].l..pos[v].r(height[v] + 1)

    /// The slot of `v` at depth offset `d`.
    ///
    /// cost: O(1)
    /// traps: unless `0 <= d <= height[v]`
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let lp = t.long_paths(); lp.slot(1, 1) == lp.slot(3, 0) and lp.slot(0, 1) == lp.slot(1, 0)`
    pub fn slot(self, v: V, d: Int) -> L:
        assert (0 <= d and d <= height[v])
        L.nth(pos[v].to_idx + d)

/// Lowest common ancestors in O(1) after O(n log n) preprocessing, by a sparse table over the
/// Euler tour: for many queries on a fixed tree when `Rooted.lca` is the bottleneck.
pub struct LcaTable<V>:
    n: Int
    pos: [V]Int
    par: [V]?V
    dep: [V]Int
    far: [V]Int
    /// Level `k` at `k * n`: the shallowest vertex of preorder positions `i..i + 2^k`.
    tab: List<V>

    /// A sparse table over the Euler walk of `t`, for O(1) lowest common ancestors on a fixed tree.
    ///
    /// cost: O(n log n) time and memory
    /// traps: never
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let lt = LcaTable(t); lt.lca(3, 4) == 1`
    pub fn new<Pre>(t: Rooted<V, Pre>) -> LcaTable<V>:
        let n = V.count
        let at = t.reorder([V] v => v)
        var far: [V]Int = 0
        var tab: List<V> = []
        for p in Pre:
            let v = at[p]
            if let u = t.parent(v): far[v] = far[u] + t.w(v)
            tab.push(v)
        let dep = [V] v => t.depth(v)
        var w = 1
        var off = 0
        while 2 * w <= n:
            for i in 0..n:
                if i + w < n:
                    let (x, y) = (tab[off + i], tab[off + i + w])
                    tab.push(if dep[y] < dep[x]: y else: x)
                else: tab.push(tab[off + i])
            off += n
            w *= 2
        LcaTable<V>(n, [V] v => t.pos(v).to_idx, [V] v => t.parent(v), dep, far, tab)

    /// The lowest common ancestor of `a` and `b`.
    ///
    /// cost: O(1)
    /// traps: when `a` and `b` are in different trees
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let lt = LcaTable(t); lt.lca(3, 2) == 0 and lt.lca(4, 4) == 4`
    pub fn lca(self, a: V, b: V) -> V:
        if a == b: return a
        let (i, j) = if pos[a] < pos[b]: (pos[a], pos[b]) else: (pos[b], pos[a])
        let k = bit_width(j - i) - 1
        let (x, y) = (tab[k * n + i + 1], tab[k * n + j + 1 - (1 << k)])
        par[if dep[y] < dep[x]: y else: x]!

    /// The distance between `a` and `b`, weighted as the tree was built.
    ///
    /// cost: O(1)
    /// traps: when `a` and `b` are in different trees
    /// example: `domain V = 0..5; let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]; let t = Rooted(adj, root: 0); let lt = LcaTable(t); lt.dist(3, 2) == 3`
    pub fn dist(self, a: V, b: V) -> Int = far[a] + far[b] - 2 * far[self.lca(a, b)]

/// A forest that changes, with folds over whole trees and subtrees, by an Euler tour tree
/// (a balanced sequence of each tree's tour). Where `LinkCut` folds paths, this folds
/// components. Values fold with `op`, which must commute. Every operation is expected
/// O(log n).
pub struct EulerTourTree<V, T>(m: Monoid<T>):
    val: [V]T
    // One splay node per vertex (node `v.to_idx`) and per arc of a tree edge; each tree is the
    // sequence of its tour. `agg` folds the vertex nodes below, `cnt` counts them.
    lc: List<?Int>
    rc: List<?Int>
    pa: List<?Int>
    agg: List<T>
    cnt: List<Int>
    arc: Map<(Int, Int), Int>
    free: List<Int>
    stk: List<Int>

    fn own(self, x: Int) -> T = if x < V.count: val[V.nth(x)] else: m.id

    fn pull(mut self, x: Int):
        var s = self.own(x)
        var k = if x < V.count: 1 else: 0
        if let c = lc[x]:
            s = m.op(agg[c], s)
            k += cnt[c]
        if let c = rc[x]:
            s = m.op(s, agg[c])
            k += cnt[c]
        agg[x] = s
        cnt[x] = k

    fn rotate(mut self, x: Int):
        let p = pa[x]!
        let g = pa[p]
        if lc[p] == Some(x):
            lc[p] = rc[x]
            if let c = rc[x]: pa[c] = Some(p)
            rc[x] = Some(p)
        else:
            rc[p] = lc[x]
            if let c = lc[x]: pa[c] = Some(p)
            lc[x] = Some(p)
        pa[p] = Some(x)
        pa[x] = g
        if let q = g:
            if lc[q] == Some(p): lc[q] = Some(x) else: rc[q] = Some(x)
        self.pull(p)
        self.pull(x)

    // Brings `x` to the root of its sequence's splay tree.
    fn splay(mut self, x: Int):
        while let p = pa[x]:
            if let g = pa[p]:
                if (lc[g] == Some(p)) == (lc[p] == Some(x)): self.rotate(p) else: self.rotate(x)
            self.rotate(x)

    // The first node of `x`'s sequence, splayed to the root: one name per tree.
    fn first(mut self, x: Int) -> Int:
        self.splay(x)
        var y = x
        while let c = lc[y]:
            y = c
        self.splay(y)
        y

    // The sequence `a` then `b`, by root; returns the new root.
    fn join(mut self, a: ?Int, b: ?Int) -> ?Int:
        if a == None: return b
        if b == None: return a
        let r = b!
        var y = a!
        while let c = rc[y]:
            y = c
        self.splay(y)
        rc[y] = Some(r)
        pa[r] = Some(y)
        self.pull(y)
        Some(y)

    // Detaches and returns what comes before the root `x`.
    fn cut_left(mut self, x: Int) -> ?Int:
        let l = lc[x]
        if let c = l: pa[c] = None
        lc[x] = None
        self.pull(x)
        l

    // Detaches and returns what comes after the root `x`.
    fn cut_right(mut self, x: Int) -> ?Int:
        let r = rc[x]
        if let c = r: pa[c] = None
        rc[x] = None
        self.pull(x)
        r

    // Rotates `x`'s tour so that it starts at `x`.
    fn reroot(mut self, x: Int):
        self.splay(x)
        let l = self.cut_left(x)
        let _ = self.join(Some(x), l)

    // A fresh arc node, reusing a cut one when there is one.
    fn arc_node(mut self) -> Int:
        if let x = free.pop():
            agg[x] = m.id
            cnt[x] = 0
            return x
        lc.push(None)
        rc.push(None)
        pa.push(None)
        agg.push(m.id)
        cnt.push(0)
        lc.len - 1

    // The two arc nodes of the edge `a`-`b`, `a` to `b` first, with `a`'s tour starting at `a`
    // and the arcs splayed out of the way; traps when there is no such edge.
    fn arcs(mut self, a: Int, b: Int) -> (Int, Int):
        let ab = arc.get((a, b))!
        let ba = arc.get((b, a))!
        self.reroot(a)
        (ab, ba)

    // Roots `a`'s tour at `a` and splays the run between the arcs of the edge `a`-`b` (`b`'s
    // side of it, never empty) to one subtree, returning its root.
    fn side(mut self, a: Int, b: Int) -> Int:
        let (ab, ba) = self.arcs(a, b)
        self.splay(ab)
        let r = self.cut_right(ab)
        self.splay(ba)
        let y = lc[ba]!
        rc[ab] = Some(ba)
        pa[ba] = Some(ab)
        self.pull(ab)
        let _ = r
        y

    /// Lone vertices with values `a`, or the identity of `op` everywhere when `a` is `None`.
    ///
    /// cost: O(n)
    /// empty: `a: None` fills every vertex with the identity
    /// traps: never
    /// example: `domain V = 0..3; var et = EulerTourTree<V, Int>(+); et.get(1) == 0 and et.len(1) == 1`
    pub fn new<M>(op: M, a: ?[V]T = None) -> EulerTourTree<V, T>:
        let m = Monoid<T>.of(op)
        let val = if let init = a: init else: [V] _ => m.id
        EulerTourTree(
            m,
            val,
            [None for _ in V],
            [None for _ in V],
            [None for _ in V],
            [val[v] for v in V],
            [1 for _ in V],
            [],
            [],
            [],
        )

    /// Joins the trees of `u` and `v` by the edge `u`-`v`.
    ///
    /// cost: expected O(log n)
    /// traps: when `u` and `v` are already in one tree
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.connected(0, 2) and et.len(2) == 3`
    pub fn link(mut self, u: V, v: V):
        assert not self.connected(u, v)
        let (a, b) = (u.to_idx, v.to_idx)
        self.reroot(a)
        self.reroot(b)
        let ab = self.arc_node()
        let ba = self.arc_node()
        arc[(a, b)] = ab
        arc[(b, a)] = ba
        self.splay(a)
        self.splay(b)
        let x = self.join(Some(a), Some(ab))
        let y = self.join(x, Some(b))
        let _ = self.join(y, Some(ba))

    /// Removes the edge `u`-`v`.
    ///
    /// cost: expected O(log n)
    /// traps: when there is no edge `u`-`v`
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.cut(1, 2); not et.connected(0, 2) and et.len(0) == 2`
    pub fn cut(mut self, u: V, v: V):
        let (a, b) = (u.to_idx, v.to_idx)
        let (ab, ba) = self.arcs(a, b)
        self.splay(ab)
        let x = self.cut_left(ab)
        let _ = self.cut_right(ab)
        self.splay(ba)
        let _ = self.cut_left(ba)
        let z = self.cut_right(ba)
        let _ = self.join(x, z)
        arc.remove((a, b))
        arc.remove((b, a))
        free.push(ab)
        free.push(ba)

    /// Whether `a` and `b` are in one tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.connected(0, 2) and not et.connected(0, 3)`
    pub fn connected(mut self, a: V, b: V) -> Bool:
        self.first(a.to_idx) == self.first(b.to_idx)

    /// The number of vertices in `v`'s tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.len(0) == 3 and et.len(3) == 1`
    pub fn len(mut self, v: V) -> Int:
        self.splay(v.to_idx)
        cnt[v.to_idx]

    /// The value at `v`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.get(3) == 4`
    pub fn get(self, v: V) -> T = val[v]

    /// Replaces the value at `v`.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.set(0, 10); et.fold_tree(2) == 15`
    pub fn set(mut self, v: V, x: T):
        self.splay(v.to_idx)
        val[v] = x
        self.pull(v.to_idx)

    /// The fold of every value in `v`'s tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.fold_tree(0) == 6 and et.fold_tree(3) == 4`
    pub fn fold_tree(mut self, v: V) -> T:
        self.splay(v.to_idx)
        agg[v.to_idx]

    /// The fold over `v`'s side of the edge `v`-`away`: what `v`'s tree would hold were the edge
    /// cut.
    ///
    /// cost: expected O(log n)
    /// traps: when there is no edge `v`-`away`
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = EulerTourTree(+, a: Some(vals)); et.link(0, 1); et.link(1, 2); et.fold_sub(1, 2) == 3 and et.fold_sub(2, 1) == 3`
    pub fn fold_sub(mut self, v: V, away: V) -> T:
        let y = self.side(away.to_idx, v.to_idx)
        let s = agg[y]
        self.splay(v.to_idx)
        s

/// `EulerTourTree` with updates to whole trees and subtrees under an action.
pub struct LazyEulerTourTree<V, T, F>(act: Action<T, F>):
    val: [V]T
    // As in `EulerTourTree`, plus a pending tag per node.
    lc: List<?Int>
    rc: List<?Int>
    pa: List<?Int>
    agg: List<T>
    cnt: List<Int>
    lz: List<F>
    arc: Map<(Int, Int), Int>
    free: List<Int>
    stk: List<Int>

    fn own(self, x: Int) -> T = if x < V.count: val[V.nth(x)] else: act.over.id

    fn put(mut self, x: Int, f: F):
        if x < V.count:
            let v = V.nth(x)
            val[v] = act.apply(f, val[v], 1)
        agg[x] = act.apply(f, agg[x], cnt[x])
        lz[x] = act.compose(f, lz[x])

    fn push(mut self, x: Int):
        let f = lz[x]
        if let c = lc[x]: self.put(c, f)
        if let c = rc[x]: self.put(c, f)
        lz[x] = act.id

    fn pull(mut self, x: Int):
        var s = self.own(x)
        var k = if x < V.count: 1 else: 0
        if let c = lc[x]:
            s = act.over.op(agg[c], s)
            k += cnt[c]
        if let c = rc[x]:
            s = act.over.op(s, agg[c])
            k += cnt[c]
        agg[x] = s
        cnt[x] = k

    fn rotate(mut self, x: Int):
        let p = pa[x]!
        let g = pa[p]
        if lc[p] == Some(x):
            lc[p] = rc[x]
            if let c = rc[x]: pa[c] = Some(p)
            rc[x] = Some(p)
        else:
            rc[p] = lc[x]
            if let c = lc[x]: pa[c] = Some(p)
            lc[x] = Some(p)
        pa[p] = Some(x)
        pa[x] = g
        if let q = g:
            if lc[q] == Some(p): lc[q] = Some(x) else: rc[q] = Some(x)
        self.pull(p)
        self.pull(x)

    // Brings `x` to the root of its sequence's splay tree.
    fn splay(mut self, x: Int):
        stk.clear()
        var y = x
        stk.push(y)
        while let p = pa[y]:
            y = p
            stk.push(y)
        while let z = stk.pop(): self.push(z)
        while let p = pa[x]:
            if let g = pa[p]:
                if (lc[g] == Some(p)) == (lc[p] == Some(x)): self.rotate(p) else: self.rotate(x)
            self.rotate(x)

    // The first node of `x`'s sequence, splayed to the root: one name per tree.
    fn first(mut self, x: Int) -> Int:
        self.splay(x)
        var y = x
        self.push(y)
        while let c = lc[y]:
            y = c
            self.push(y)
        self.splay(y)
        y

    // The sequence `a` then `b`, by root; returns the new root.
    fn join(mut self, a: ?Int, b: ?Int) -> ?Int:
        if a == None: return b
        if b == None: return a
        let r = b!
        var y = a!
        self.push(y)
        while let c = rc[y]:
            y = c
            self.push(y)
        self.splay(y)
        rc[y] = Some(r)
        pa[r] = Some(y)
        self.pull(y)
        Some(y)

    // Detaches and returns what comes before the root `x`.
    fn cut_left(mut self, x: Int) -> ?Int:
        let l = lc[x]
        if let c = l: pa[c] = None
        lc[x] = None
        self.pull(x)
        l

    // Detaches and returns what comes after the root `x`.
    fn cut_right(mut self, x: Int) -> ?Int:
        let r = rc[x]
        if let c = r: pa[c] = None
        rc[x] = None
        self.pull(x)
        r

    // Rotates `x`'s tour so that it starts at `x`.
    fn reroot(mut self, x: Int):
        self.splay(x)
        let l = self.cut_left(x)
        let _ = self.join(Some(x), l)

    // A fresh arc node, reusing a cut one when there is one.
    fn arc_node(mut self) -> Int:
        if let x = free.pop():
            agg[x] = act.over.id
            cnt[x] = 0
            lz[x] = act.id
            return x
        lc.push(None)
        rc.push(None)
        pa.push(None)
        agg.push(act.over.id)
        cnt.push(0)
        lz.push(act.id)
        lc.len - 1

    // The two arc nodes of the edge `a`-`b`, `a` to `b` first, with `a`'s tour starting at `a`
    // and the arcs splayed out of the way; traps when there is no such edge.
    fn arcs(mut self, a: Int, b: Int) -> (Int, Int):
        let ab = arc.get((a, b))!
        let ba = arc.get((b, a))!
        self.reroot(a)
        (ab, ba)

    // Roots `a`'s tour at `a` and splays the run between the arcs of the edge `a`-`b` (`b`'s
    // side of it, never empty) to one subtree, returning its root.
    fn side(mut self, a: Int, b: Int) -> Int:
        let (ab, ba) = self.arcs(a, b)
        self.splay(ab)
        let r = self.cut_right(ab)
        self.splay(ba)
        let y = lc[ba]!
        rc[ab] = Some(ba)
        pa[ba] = Some(ab)
        self.pull(ab)
        let _ = r
        y

    /// Lone vertices with values `a`, or the identity of `act.over` everywhere when `a` is `None`.
    ///
    /// cost: O(n)
    /// empty: `a: None` fills every vertex with the identity
    /// traps: never
    /// example: `domain V = 0..3; var et = LazyEulerTourTree<V, Int, Int>(Action<Int, Int>(apply: (d, x, n) => x + d * n)); et.get(1) == 0`
    pub fn new(act: Action<T, F>, a: ?[V]T = None) -> LazyEulerTourTree<V, T, F>:
        let val = if let init = a: init else: [V] _ => act.over.id
        LazyEulerTourTree(
            act,
            val,
            [None for _ in V],
            [None for _ in V],
            [None for _ in V],
            [val[v] for v in V],
            [1 for _ in V],
            [act.id for _ in V],
            [],
            [],
            [],
        )

    /// Joins the trees of `u` and `v` by the edge `u`-`v`.
    ///
    /// cost: expected O(log n)
    /// traps: when `u` and `v` are already in one tree
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.connected(0, 2) and et.len(2) == 3`
    pub fn link(mut self, u: V, v: V):
        assert not self.connected(u, v)
        let (a, b) = (u.to_idx, v.to_idx)
        self.reroot(a)
        self.reroot(b)
        let ab = self.arc_node()
        let ba = self.arc_node()
        arc[(a, b)] = ab
        arc[(b, a)] = ba
        self.splay(a)
        self.splay(b)
        let x = self.join(Some(a), Some(ab))
        let y = self.join(x, Some(b))
        let _ = self.join(y, Some(ba))

    /// Removes the edge `u`-`v`.
    ///
    /// cost: expected O(log n)
    /// traps: when there is no edge `u`-`v`
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.cut(1, 2); not et.connected(0, 2)`
    pub fn cut(mut self, u: V, v: V):
        let (a, b) = (u.to_idx, v.to_idx)
        let (ab, ba) = self.arcs(a, b)
        self.splay(ab)
        let x = self.cut_left(ab)
        let _ = self.cut_right(ab)
        self.splay(ba)
        let _ = self.cut_left(ba)
        let z = self.cut_right(ba)
        let _ = self.join(x, z)
        arc.remove((a, b))
        arc.remove((b, a))
        free.push(ab)
        free.push(ba)

    /// Whether `a` and `b` are in one tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); not et.connected(0, 3)`
    pub fn connected(mut self, a: V, b: V) -> Bool:
        self.first(a.to_idx) == self.first(b.to_idx)

    /// The number of vertices in `v`'s tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.len(1) == 3`
    pub fn len(mut self, v: V) -> Int:
        self.splay(v.to_idx)
        cnt[v.to_idx]

    /// The value at `v`, with every update to a tree or side holding it applied.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.apply_tree(0, 10); et.get(2) == 13 and et.get(3) == 4`
    pub fn get(mut self, v: V) -> T:
        self.splay(v.to_idx)
        val[v]

    /// Replaces the value at `v`, discarding earlier updates there.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.set(0, 10); et.fold_tree(2) == 15`
    pub fn set(mut self, v: V, x: T):
        self.splay(v.to_idx)
        val[v] = x
        self.pull(v.to_idx)

    /// The fold of every value in `v`'s tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.fold_tree(0) == 6`
    pub fn fold_tree(mut self, v: V) -> T:
        self.splay(v.to_idx)
        agg[v.to_idx]

    /// The fold over `v`'s side of the edge `v`-`away`.
    ///
    /// cost: expected O(log n)
    /// traps: when there is no edge `v`-`away`
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.fold_sub(1, 2) == 3`
    pub fn fold_sub(mut self, v: V, away: V) -> T:
        let y = self.side(away.to_idx, v.to_idx)
        let s = agg[y]
        self.splay(v.to_idx)
        s

    /// Applies `f` to every value in `v`'s tree.
    ///
    /// cost: expected O(log n)
    /// traps: never
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.apply_tree(0, 10); et.fold_tree(2) == 36 and et.get(3) == 4`
    pub fn apply_tree(mut self, v: V, f: F):
        self.splay(v.to_idx)
        self.put(v.to_idx, f)

    /// Applies `f` to every value on `v`'s side of the edge `v`-`away`.
    ///
    /// cost: expected O(log n)
    /// traps: when there is no edge `v`-`away`
    /// example: `domain V = 0..4; let vals: [V]Int = [1, 2, 3, 4]; var et = LazyEulerTourTree(Action<Int, Int>(apply: (d, x, n) => x + d * n), a: Some(vals)); et.link(0, 1); et.link(1, 2); et.apply_sub(1, 2, 10); et.get(0) == 11 and et.get(2) == 3 and et.fold_tree(0) == 26`
    pub fn apply_sub(mut self, v: V, away: V, f: F):
        let y = self.side(away.to_idx, v.to_idx)
        self.put(y, f)
        self.splay(y)

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
pub fn cartesian_tree<D, T>(a: [D]T, by: (T, T) -> Bool = (x, y) => x < y) -> [D]?D:
    var par: [D]?D = None
    var stack: List<D> = []
    for i in D:
        var last: ?D = None
        while not stack.is_empty and by(a[i], a[stack.last!]): last = stack.pop()
        if let l = last: par[l] = Some(i)
        if not stack.is_empty: par[i] = Some(stack.last!)
        stack.push(i)
    par

/// The Kruskal reconstruction tree: each edge that merges two components, in increasing
/// weight, becomes a node above them. The leaves under a node are the vertices connected by
/// edges of weight at most its own, so "what `v` reaches under a limit" is one subtree.
pub struct KruskalTree<V, N, W>:
    /// The leaf node of each vertex.
    leaf: [V]N
    /// The merging weight of each node; `None` at a leaf.
    w: [N]?W
    lift: Lift<N, Int>

    /// The nodes as a `Rooted` forest over `N`: subtree ranges, LCA and folds. The roots, one per
    /// component, are in increasing node order.
    ///
    /// cost: O(1); built with the tree
    /// traps: never
    /// example: `domain V = 0..3; let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3)]; let (domain N, kt) = kruskal_tree(es, w: e => es[e].w); kt.tree.parent(kt.leaf[1]) == kt.tree.parent(kt.leaf[2]) and kt.tree.size(kt.leaf[0]) == 1`
    @get pub fn tree(self) -> Rooted<N, _> = Rooted.from_parents(lift.up)

    /// The highest node above `leaf[v]` whose weight is at most `limit` (`leaf[v]` itself when
    /// None is); its leaves are what `v` reaches by edges of weight at most `limit`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3)]; let (domain N, kt) = kruskal_tree(es, w: e => es[e].w); kt.reach(0, 4) == kt.leaf[0] and kt.w[kt.reach(1, 4)] == Some(3) and kt.w[kt.reach(1, 5)] == Some(5)`
    pub fn reach(self, v: V, limit: W) -> N = lift.climb(leaf[v], ok: x => w[x]! <= limit).at

    /// The highest node above `leaf[v]` whose merging edge, and every one below it, satisfies `ok`,
    /// which gets the edge's key in the list given to `kruskal_tree`
    /// (`kt.top(v, ok: e => roads[e].alt > p)`). `ok` must hold on a prefix of the merge order.
    ///
    /// cost: O(log n) calls of `ok`
    /// ties: None; the highest such node is unique
    /// traps: never
    /// example: `domain V = 0..3; let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3)]; let (domain N, kt) = kruskal_tree(es, w: e => es[e].w); kt.w[kt.top(1, e => es[e].w < 5)] == Some(3) and kt.top(0, e => false) == kt.leaf[0]`
    pub fn top<E>(self, v: V, ok: E -> Bool) -> N = todo

    /// For every node, `op` folded over the `leaf` values of the leaves under it: the least walk
    /// home from any vertex a subtree reaches is `kt.up(min, leaf: v => home[v]!)`. Each node
    /// folds its two children, the lower node first, so `op` should commute unless that order
    /// is the one wanted.
    ///
    /// cost: O(n) calls of `op` and `leaf`
    /// empty: never empty; every leaf has a value
    /// order: keyed by `N`
    /// traps: never
    /// example: `domain V = 0..3; let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3)]; let (domain N, kt) = kruskal_tree(es, w: e => es[e].w); let c = kt.up(+, leaf: v => 1); c[kt.reach(2, 3)] == 2 and c[kt.leaf[0]] == 1`
    pub fn up<T, M>(self, op: M, leaf: V -> T) -> [N]T:
        let g = Monoid<T>.of(op).op
        var acc: [N]?T = None
        for v in V: acc[self.leaf[v]] = Some(leaf(v))
        for x in N:
            if let p = lift.parent(x):
                let c = acc[x]!
                acc[p] = if let a = acc[p]: Some(g(a, c)) else: Some(c)
        [N] x => acc[x]!

    /// The least possible largest edge weight on a path from `a` to `b`: the weight of their
    /// lowest common ancestor. `None` when they are not connected, and at `a == b`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..3; let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3)]; let (domain N, kt) = kruskal_tree(es, w: e => es[e].w); kt.bottleneck(0, 2) == Some(5) and kt.bottleneck(1, 2) == Some(3) and kt.bottleneck(0, 0) == None`
    pub fn bottleneck(self, a: V, b: V) -> ?W:
        if a == b: return None
        if let c = lift.lca(leaf[a], leaf[b]): w[c]
        else: None

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
pub fn kruskal_tree<V, E, W, R: (u: V, v: V, ..)>(edges: [E]R, w: E -> W) -> (
    domain N,
    KruskalTree<V,
    N,
    W>,
):
    var order = [e for e in E]
    order.sort(by: e => w(e))
    var dsu = Dsu<V>()
    var top: [V]Int = [V] v => v.to_idx
    var par: List<Int> = [-1 for _ in V]
    var weight: List<?W> = [None for _ in V]
    for e in order:
        let r = edges[e]
        let a = top[dsu.find(r.u)]
        let b = top[dsu.find(r.v)]
        if dsu.unite(r.u, r.v):
            let x = par.len
            par[a] = x
            par[b] = x
            par.push(-1)
            weight.push(Some(w(e)))
            top[dsu.find(r.u)] = x
    domain N = 0..par.len
    var up: [N]?(N, Int) = None
    for x in N:
        if par[x.to_idx] >= 0: up[x] = Some((N.nth(par[x.to_idx]), 0))
    let lift = Lift(up, op: (a, b) => a + b, identity: 0)
    (N, KruskalTree::<V, N, W>([V] v => N.nth(v.to_idx), [N] x => weight[x.to_idx], lift))

/// The Prüfer code of a tree on `V`: repeatedly remove the least leaf and record its neighbour,
/// until two vertices remain. `V.count - 2` vertices, in bijection with the labelled trees.
///
/// cost: O(n)
/// empty: `[]` when `V` has two vertices
/// order: removal order
/// traps: when `V` has fewer than two vertices, or `edges` is not a tree
/// example: `domain V = 0..4; let tr: Tree<V> = [(u: 0, v: 1), (u: 1, v: 2), (u: 1, v: 3)]; prufer(tr) == [1, 1]`
/// example: `domain V = 0..4; let tr: Tree<V> = [(u: 0, v: 1), (u: 1, v: 2), (u: 2, v: 3)]; prufer(tr) == [1, 2]`
pub fn prufer<V, E, R: (u: V, v: V, ..)>(edges: [E]R) -> List<V>:
    assert V.count >= 2, "`prufer` needs at least two vertices"
    assert E.count == V.count - 1, "`prufer`: `edges` is not a tree"
    var adj: [V]List<V> = []
    var deg: [V]Int = 0
    for e in edges:
        adj[e.u].push(e.v)
        adj[e.v].push(e.u)
        deg[e.u] += 1
        deg[e.v] += 1
    var out: List<V> = []
    var at = 0
    while deg[V.nth(at)] != 1: at += 1
    var leaf = V.nth(at)
    for _ in 0..V.count - 2:
        deg[leaf] = 0
        var nb = leaf
        for x in adj[leaf]:
            if deg[x] > 0: nb = x
        out.push(nb)
        deg[nb] -= 1
        if deg[nb] == 1 and nb.to_idx < at: leaf = nb
        else:
            at += 1
            while deg[V.nth(at)] != 1: at += 1
            leaf = V.nth(at)
    out

/// The tree whose Prüfer code is `code`. Edges come in removal order as `(u: leaf, v:
/// neighbour)`, the last joining the two vertices left, smaller first.
///
/// cost: O(n)
/// empty: `[]` with two vertices gives the one edge `(u: 0, v: 1)` in `V` order
/// traps: when `code` does not have `V.count - 2` entries, or `V` has fewer than two vertices
/// example: `domain V = 0..4; from_prufer<V>([1, 1]) == [(u: 0, v: 1), (u: 2, v: 1), (u: 1, v: 3)]`
pub fn from_prufer<V>(code: List<V>) -> Tree<V>:
    assert V.count >= 2 and code.len == V.count - 2, "`from_prufer` needs `V.count - 2` entries"
    var deg: [V]Int = 1
    for x in code: deg[x] += 1
    var out: List<(u: V, v: V)> = []
    var at = 0
    while deg[V.nth(at)] != 1: at += 1
    var leaf = V.nth(at)
    for x in code:
        out.push((u: leaf, v: x))
        deg[leaf] = 0
        deg[x] -= 1
        if deg[x] == 1 and x.to_idx < at: leaf = x
        else:
            at += 1
            while deg[V.nth(at)] != 1: at += 1
            leaf = V.nth(at)
    deg[leaf] = 0
    var last = leaf
    for v in V:
        if deg[v] == 1: last = v
    if leaf.to_idx < last.to_idx: out.push((u: leaf, v: last)) else: out.push((u: last, v: leaf))
    [0..V.count - 1] i => out[i]
