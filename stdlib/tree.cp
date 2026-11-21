//! Rooted trees: parents, subtree ranges, LCA, and path folds over one range structure.
//!
//! Each `Rooted` tree value `t` owns a fresh domain, `t.Pre`: the positions of an
//! Euler order in which every subtree and every heavy chain is one contiguous range.
//! `t.pos(v)` is a `t.Pre`, not a `V` (and never mixes with another tree `t2.Pre`), so
//! indexing a per-position structure with a vertex or another tree's position does not type-check.
//!
//! `Rooted` is the one tree abstraction. A value per vertex lives in any
//! structure indexed by `t.Pre` (a `SegTree`, a `LazySegTree`, a `Fenwick`), and
//! `fold_path` / `apply_path` walk a path over it, so there is no second
//! "tree with values" type and no second copy of the data.
use std.algebra.{Reversible}
use std.graph.{Neighbours, Target, Graph}
use std.algebra.{Monoid, MaxSub, Action}
use std.segtree.{SegTree}
use std.lazy.{LazySegTree, AddMax, AddSum}
use std.seq.{excl_fold}
use std.rand.{Rng}

/// A tree rooted at `root`, owning its own fresh heavy-first preorder domain `t.Pre`.
/// Built in O(n) without recursion; memory is seven `[V]` arrays, one `[Pre]` array and child lists.
pub struct Rooted<V, Pre>:
    root: V
    /// The parent; `none` at the root.
    par: [V]?V
    dep: [V]Int
    /// The total edge length from the root; equal to the depth when the edges have no lengths.
    far: [V]Int
    /// The number of vertices in the subtree.
    sz: [V]Int
    /// Breadth-first order: the children of `v` are `bfs[k0[v]..k1[v]]`, so no vertex owns a list.
    bfs: List<V>
    k0: [V]Int
    k1: [V]Int
    /// The child with the largest subtree; `none` at a leaf.
    heavy: [V]?V
    /// The highest vertex of the heavy chain through `v`.
    top: [V]V
    tin: [V]Pre
    /// The inverse of `tin` / `pos`.
    at: [Pre]V

    /// Root the tree given by `adj` at `root`: `Rooted(g, root: r)` for a `Graph`,
    /// `Rooted(lists, root: r)` for a table of neighbour lists `[V]List<V>`, and
    /// `Rooted(root: r, adj: u => ...)` for anything else. `adj` may list every neighbour
    /// (the parent is skipped) or only the children, as vertices or as arcs. The tree is the
    /// breadth-first spanning tree, so on a graph with extra edges `depth` is the distance
    /// from the root. Stops if some vertex is not reachable from the root.
    pub fn new<N>(adj: N, root: V) = Rooted.weighted(adj, root, a => 1)

    /// Build a rooted tree directly from a parent array `par: [V]?V` where the unique vertex
    /// with `par[v] == none` is the root. O(n).
    pub fn from_parents(par: [V]?V):
        var root: ?V = none
        var start = List.filled(V.K + 1, 0)
        for v in V:
            if let p = par[v]: start[p.to_idx + 1] += 1
            else:
                assert root == none, "from_parents needs exactly one vertex without a parent"
                root = v
        assert root != none, "from_parents needs exactly one vertex without a parent"
        for i in 0..V.K: start[i + 1] += start[i]
        var flat = List.filled(V.K - 1, root!)
        var fill = copy(start)
        for v in V:
            if let p = par[v]:
                flat[fill[p.to_idx]] = v
                fill[p.to_idx] += 1
        var dep: [V]Int = 0
        var k0: [V]Int = 0
        var k1: [V]Int = 0
        var order = [root!]
        var i = 0
        while i < order.len:
            let u = order[i]
            k0[u] = order.len
            for j in start[u.to_idx]..start[u.to_idx + 1]:
                let c = flat[j]
                dep[c] = dep[u] + 1
                order.push(c)
            k1[u] = order.len
            i += 1
        assert order.len == V.K, "from_parents found a cycle among the parents"
        Rooted.finish([root!], par, dep, copy(dep), order, k0, k1)

    /// The same for edges with lengths: `len(arc)` is the length of the edge an arc runs along,
    /// and `dist` adds lengths up instead of counting edges.
    pub fn weighted<N, A>(adj: N, root: V, len: A -> Int) = Rooted.build(adj, [root], len)

    /// One tree per root: `adj` must reach every vertex from exactly one of `roots`. The trees
    /// sit one after another in `t.Pre`, in the order of `roots`; `t.root` is the first root.
    /// Paths and lowest common ancestors exist only within one tree (`root_of` tells). O(n).
    pub fn forest<N>(adj: N, roots: List<V>) = Rooted.build(adj, roots, a => 1)

    fn build<N, A>(adj: N, roots: List<V>, len: A -> Int):
        var par: [V]?V = none
        var dep: [V]Int = 0
        var far: [V]Int = 0
        var seen: [V]Bool = false
        var k0: [V]Int = 0
        var k1: [V]Int = 0
        var order = List<V>()
        for r in roots:
            seen[r] = true
            order.push(r)
        var i = 0
        while i < order.len:
            let u = order[i]
            k0[u] = order.len
            for a in adj.out(u):
                let c = a.to
                if not seen[c]:
                    seen[c] = true
                    par[c] = u
                    dep[c] = dep[u] + 1
                    far[c] = far[u] + len(a)
                    order.push(c)
            k1[u] = order.len
            i += 1
        assert order.len == V.K
        Rooted.finish(roots, par, dep, far, order, k0, k1)

    fn finish(
        roots: List<V>,
        par: [V]?V,
        dep: [V]Int,
        far: [V]Int,
        bfs: List<V>,
        k0: [V]Int,
        k1: [V]Int,
    ):
        domain P = 0..V.K
        let root = roots[0]
        var sz: [V]Int = 1
        for u in bfs.rev:
            if let p = par[u]: sz[p] += sz[u]
        var heavy: [V]?V = none
        for u in bfs:
            for i in k0[u]..k1[u]:
                let c = bfs[i]
                if heavy[u] == none or sz[c] > sz[heavy[u]!]: heavy[u] = c
        var top: [V]V = root
        var tin: [V]P = P.first
        var start = 0
        for r in roots:
            top[r] = r
            tin[r] = P.nth(start)
            start += sz[r]
        for u in bfs:
            var next = tin[u].ord + 1
            if let h = heavy[u]:
                top[h] = top[u]
                tin[h] = P.nth(next)
                next += sz[h]
            for i in k0[u]..k1[u]:
                let c = bfs[i]
                if some(c) != heavy[u]:
                    top[c] = c
                    tin[c] = P.nth(next)
                    next += sz[c]
        var at: [P]V = root
        for v in V: at[tin[v]] = v
        Rooted<V, P>(root, par, dep, far, sz, bfs, k0, k1, heavy, top, tin, at)

    /// The position of `v` in this tree's value-owned preorder domain `t.Pre`. Inverse of `at`. O(1).
    pub fn pos(self, v: V) -> Pre = tin[v]

    /// The parent of `v`; `none` at the root.
    pub fn parent(self, v: V) -> ?V = par[v]

    /// Reorder a vertex-indexed array `[V]T` into preorder `[t.Pre]T` (`out[t.pos(v)] == a[v]`). O(n).
    pub fn reorder<T>(self, a: [V]T) -> [Pre]T = [Pre] p => a[at[p]]

    /// The children of `v`, in the order `adj` listed them.
    pub fn children(self, v: V) -> List<V> = [bfs[i] for i in k0[v]..k1[v]]

    /// For each child `c` of `v`, in child order, the fold with `m` of `f` over `v`'s other
    /// children: the step of a rerooting DP, with no division or inverse needed. The fold keeps
    /// child order (earlier siblings, then later ones), so `m` need not commute. O(k) calls of
    /// `f` and of `m` for `k` children.
    ///
    ///     for (c, rest) in t.excl_fold(v, mul, c => down[c] + 1):
    ///         up[c] = (up[v] * rest + 1) % m
    pub fn excl_fold<T, M>(self, v: V, m: M, f: V -> T) -> yields (c: V, rest: T):
        let cs = self.children(v)
        var i = 0
        for (_, rest) in [f(c) for c in cs].excl_fold(m):
            yield (c: cs[i], rest: rest)
            i += 1

    /// The number of edges between `v` and the root.
    pub fn depth(self, v: V) -> Int = dep[v]

    /// The number of vertices in the subtree of `v`.
    pub fn size(self, v: V) -> Int = sz[v]

    /// The children of `v` other than `heavy[v]`, in child order. Every vertex has O(log n)
    /// light edges above it, which is what small-to-large merging pays for. O(children).
    pub fn light(self, v: V) -> yields V:
        for i in k0[v]..k1[v]:
            if some(bfs[i]) != heavy[v]: yield bfs[i]

    /// The root of the tree that holds `v` (always `t.root` unless built with `forest`). O(log n).
    pub fn root_of(self, v: V) -> V:
        var x = top[v]
        while let p = par[x]: x = top[p]
        x

    /// The vertices of `v`'s subtree in preorder, `v` first: `t.at[p] for p in t.sub(v)`.
    /// O(size(v)).
    pub fn subtree(self, v: V) -> yields V:
        for p in self.sub(v): yield at[p]

    /// A top-down pass: `out[v] = f(out[parent], v)`, and `f(init, root)` at a root. The
    /// state on the path from the root, such as a prefix maximum or a distance. O(n) calls of `f`.
    pub fn down<T>(self, init: T, f: (T, V) -> T) -> [V]T:
        var out = [V] v => init
        for p in Pre:
            let v = at[p]
            if let q = par[v]: out[v] = f(out[q], v)
            else: out[v] = f(init, v)
        out

    /// A bottom-up tree DP: `out[v] = finish(fold with m of lift(out[c], c, v) over children c, v)`.
    /// `lift` sees both ends of the edge, child first. Children fold in child order. O(n) calls.
    pub fn up<T, U, M>(self, m: M, lift: (T, V, V) -> U, finish: (U, V) -> T) -> [V]T:
        let k = Monoid<U>.of(m)
        var out: [V]?T = none
        for p in Pre.rev:
            let v = at[p]
            var acc = k.id
            for i in k0[v]..k1[v]: acc = k.op(acc, lift(out[bfs[i]]!, bfs[i], v))
            out[v] = some(finish(acc, v))
        [V] v => out[v]!

    /// The same DP as `up`, for every root at once: `out[v]` is what `up` would give at `v` if the
    /// tree were rooted at `v`. Each vertex folds its parent's side first, then its children; the
    /// sibling folds use prefixes and suffixes, so `m` needs no inverse. O(n) calls.
    pub fn reroot<T, U, M>(self, m: M, lift: (T, V, V) -> U, finish: (U, V) -> T) -> [V]T:
        let k = Monoid<U>.of(m)
        let below = self.up(m, lift, finish)
        var above = [V] v => k.id
        var out: [V]?T = none
        for p in Pre:
            let v = at[p]
            let cs = self.children(v)
            let parts = [lift(below[c], c, v) for c in cs]
            var after = List.filled(cs.len + 1, k.id)
            for i in (0..cs.len).rev: after[i] = k.op(parts[i], after[i + 1])
            var before = above[v]
            for i in 0..cs.len:
                above[cs[i]] = lift(finish(k.op(before, after[i + 1]), v), v, cs[i])
                before = k.op(before, parts[i])
            out[v] = some(finish(before, v))
        [V] v => out[v]!

    /// The isomorphism class of every subtree: `of[a] == of[b]` exactly when the subtrees of `a`
    /// and `b` are the same rooted tree up to reordering children. Classes are numbered by first
    /// appearance from the leaves up. O(n log n).
    pub fn shapes(self) -> (domain Shape, of: [V]Shape):
        var known = Map<List<Int>, Int>()
        var of: [V]Int = 0
        for p in Pre.rev:
            let v = at[p]
            var key = [of[bfs[i]] for i in k0[v]..k1[v]]
            key.sort()
            of[v] = known.get_or_insert(key, known.len)
        domain Shape = 0..known.len
        (Shape: Shape, of: [V] v => Shape(of[v]))

    /// The size of every subtree, by vertex: `t.subtree_sizes[v]` is `t.size(v)`.
    pub @get fn subtree_sizes(self) -> [V]Int = sz

    /// A centroid: a vertex whose removal leaves no piece larger than half the tree. When
    /// there are two, this is the one nearer the root. O(depth).
    pub @get fn centroid(self) -> V:
        var v = root
        while let h = heavy[v], 2 * sz[h] > sz[root]: v = h
        v

    /// Every centroid, one or two: the one nearer the root first. Two happen only when the
    /// number of vertices is even and one edge splits the tree in halves. O(depth).
    pub @get fn centroids(self) -> List<V>:
        let c = self.centroid
        if let h = heavy[c], 2 * sz[h] == sz[root]: [c, h] else: [c]

    /// Iterative enter/exit Euler tour: yields `(v, true)` when entering `v` and
    /// `(v, false)` when leaving `v`'s subtree, without recursion. O(n).
    pub @get fn tour(self) -> yields (v: V, entering: Bool):
        var stack = [(at[p], 0) for p in Pre.rev if par[at[p]] == none]
        while not stack.is_empty:
            let (u, i) = stack[stack.len - 1]
            if i == 0: yield (v: u, entering: true)
            if k0[u] + i < k1[u]:
                stack[stack.len - 1] = (u, i + 1)
                stack.push((bfs[k0[u] + i], 0))
            else:
                stack.pop()
                yield (v: u, entering: false)

    /// The positions of `v`'s subtree, `v` first.
    pub fn sub(self, v: V) -> Range<Pre> = tin[v].l..tin[v].r(sz[v])

    /// The positions of `v`'s subtree without `v`: its descendants. O(1).
    pub fn below(self, v: V) -> Range<Pre> = tin[v].r..tin[v].r(sz[v])

    /// Is `a` an ancestor of `b`? A vertex is its own ancestor. O(1).
    pub fn is_anc(self, a: V, b: V) -> Bool = tin[b] in self.sub(a)

    /// The child end of the tree edge between `u` and `v`: the one below. O(1).
    pub fn lower(self, u: V, v: V) -> V:
        assert par[u] == v or par[v] == u
        if par[u] == v: u else: v

    /// The lowest common ancestor. O(log n).
    pub fn lca(self, a: V, b: V) -> V:
        var (x, y) = (a, b)
        while top[x] != top[y]:
            if dep[top[x]] < dep[top[y]]: swap(mut x, mut y)
            x = par[top[x]]!
        if dep[x] < dep[y]: x else: y

    /// The length of the path between `a` and `b`: its number of edges, or their total length in a
    /// tree built with `weighted`. O(log n).
    pub fn dist(self, a: V, b: V) -> Int = far[a] + far[b] - 2 * far[self.lca(a, b)]

    /// The simple path of vertices from `a` to `b`, inclusive. O(|path|).
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

    /// A longest simple path in the tree: `(len, u, v)` with `self.dist(u, v) == len`. O(n).
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

    /// The ancestor `k` edges above `v`, or `none` if that is above the root. O(log n).
    pub fn ancestor(self, v: V, k: Int) -> ?V:
        if k < 0 or k > dep[v]: return none
        var (x, left) = (v, k)
        while dep[x] - dep[top[x]] < left:
            left -= dep[x] - dep[top[x]] + 1
            x = par[top[x]]!
        at[tin[x].offset(-left)!]

    /// The vertex `k` edges from `a` on the path to `b`, or `none` if the path is shorter. O(log n).
    pub fn step(self, a: V, b: V, k: Int) -> ?V:
        let c = self.lca(a, b)
        let (da, db) = (dep[a] - dep[c], dep[b] - dep[c])
        if k <= da: return self.ancestor(a, k)
        if k > da + db: return none
        self.ancestor(b, da + db - k)

    /// The highest ancestor `u` of `v` such that `pred` holds for `u`, or `none` when it does not
    /// hold for `v` itself. `pred` must be true and then false on the way up. O(log^2 n).
    pub fn lift_while(self, v: V, pred: V -> Bool) -> ?V:
        if not pred(v): return none
        var (lo, hi) = (0, dep[v])
        while lo < hi:
            let mid = (lo + hi + 1) / 2
            if pred(self.ancestor(v, mid)!): lo = mid else: hi = mid - 1
        self.ancestor(v, lo)

    /// The path from `a` to `b` as O(log n) position ranges. `rising` ranges are walked from
    /// their high end to their low end. With `edges`, the highest vertex of the path is left
    /// out: for a value per edge, stored at the edge's lower end.
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
    /// structure over `Pre` with `fold(range)` and a `monoid` (`SegTree`, `LazySegTree`).
    /// A stretch of the path that climbs is folded top-down by `ds` and then turned round
    /// with `rev()`: a value that is not its own reverse implements `Reversible`. One
    /// structure, O(log^2 n).
    pub fn fold_path<S, T>(self, a: V, b: V, ds: S, edges: Bool = false) -> T:
        let m = ds.monoid
        var acc = m.id
        for c in self.chains(a, b, edges):
            let part = ds.fold(c.range)
            acc = m.op(acc, if c.rising: part.rev() else: part)
        acc

    /// Every vertex on the path from `a` to `b` takes the update `tag`, through `ds.apply`.
    /// O(log^2 n).
    pub fn apply_path<S, F>(self, a: V, b: V, mut ds: S, tag: F, edges: Bool = false):
        for c in self.chains(a, b, edges): ds.apply(c.range, tag)

    /// The tree squeezed down to `keys`: those vertices and the lowest common ancestors
    /// between them, each with its nearest kept ancestor as parent. `nodes` is in preorder
    /// and `edges` go from parent to child. O(k log k + k log n).
    pub fn virtual_tree(self, keys: List<V>) -> (nodes: List<V>, edges: List<(u: V, v: V)>):
        if keys.is_empty: return (nodes: [], edges: [])
        var all = [(tin[v].ord, v) for v in keys]
        all.sort()
        for i in 0..keys.len - 1:
            let l = self.lca(all[i].1, all[i + 1].1)
            all.push((tin[l].ord, l))
        all.sort()
        var nodes = List<V>()
        for p in all:
            if nodes.is_empty or nodes[nodes.len - 1] != p.1: nodes.push(p.1)
        var edges: List<(u: V, v: V)> = []
        var st = [nodes[0]]
        for i in 1..nodes.len:
            let u = nodes[i]
            while not self.is_anc(st.last!, u): st.pop()
            edges.push((u: st.last!, v: u))
            st.push(u)
        (nodes: nodes, edges: edges)

/// A centroid decomposition: the centroid of the tree, then recursively of each piece left when
/// it is removed. Every path passes through the centroid ancestor that first separates its
/// ends, and each vertex has at most log2 n + 1 centroid ancestors, so a question about all
/// paths through `v` is a loop over `ancestors(v)`.
/// Built in O(n log n) time and memory without recursion.
pub struct Centroids<V>:
    root: V
    cpar: [V]?V
    cdep: [V]Int
    /// For each vertex, every centroid whose piece holds it, nearest first, with the
    /// distance and the neighbour of the centroid it hangs off.
    anc: [V]List<(c: V, dist: Int, branch: V)>
    /// For each centroid, the vertices of its piece in breadth-first order from it.
    held: [V]List<V>

    /// Decompose the tree given by `adj` (as for `Rooted`: a `Graph`, neighbour lists or a fn).
    pub fn new<N>(adj: N) -> Centroids<V>:
        var dead: [V]Bool = false
        var sz: [V]Int = 0
        var cpar: [V]?V = none
        var cdep: [V]Int = 0
        var anc: [V]List<(c: V, dist: Int, branch: V)> = []
        var held: [V]List<V> = []
        var bpar: [V]?V = none
        var from: [V]?V = none
        var root = V.first
        var todo: List<(V, ?V, Int)> = [(V.first, none, 0)]
        while let item = todo.pop():
            let (entry, above, d) = item
            var order = [entry]
            bpar[entry] = none
            var head = 0
            while head < order.len:
                let u = order[head]
                head += 1
                for a in adj.out(u):
                    let w = a.to
                    if not dead[w] and some(w) != bpar[u] and w != entry:
                        bpar[w] = u
                        order.push(w)
            for u in order: sz[u] = 1
            for i in (1..order.len).rev: sz[bpar[order[i]]!] += sz[order[i]]
            let total = order.len
            var c = entry
            var moved = true
            while moved:
                moved = false
                for a in adj.out(c):
                    let w = a.to
                    if not dead[w] and some(w) != bpar[c] and w != entry and 2 * sz[w] > total:
                        c = w
                        moved = true
                        break
            if above == none: root = c
            cpar[c] = above
            cdep[c] = d
            dead[c] = true
            var q: List<(V, Int, V)> = [(c, 0, c)]
            from[c] = none
            var qh = 0
            while qh < q.len:
                let (u, du, b) = q[qh]
                qh += 1
                anc[u].push((c: c, dist: du, branch: b))
                held[c].push(u)
                for a in adj.out(u):
                    let w = a.to
                    if not dead[w] and some(w) != from[u]:
                        from[w] = u
                        let nb = if u == c: w else: b
                        q.push((w, du + 1, nb))
            for a in adj.out(c):
                if not dead[a.to]: todo.push((a.to, some(c), d + 1))
        for v in V: anc[v].reverse()
        Centroids(root, cpar, cdep, anc, held)

    /// The parent of centroid `c` in the centroid tree; `none` at `cd.root`. O(1).
    pub fn parent(self, c: V) -> ?V = cpar[c]

    /// The depth of `c` in the centroid tree, at most log2 n. O(1).
    pub fn depth(self, c: V) -> Int = cdep[c]

    /// Every centroid whose piece holds `v`, nearest first (`v` itself, with distance 0), with
    /// the tree distance from it. At most log2 n + 1 entries, O(1) each, no allocation.
    pub fn ancestors(self, v: V) -> yields (c: V, dist: Int):
        for x in anc[v]: yield (c: x.c, dist: x.dist)

    /// The piece of centroid `c`: each vertex `v`, its distance from `c`, and `branch`, the
    /// neighbour of `c` on the way to `v` (`c` itself for `v == c`). Pairs in different branches
    /// are exactly the paths whose highest centroid is `c`. O(size of the piece).
    pub fn piece(self, c: V) -> yields (v: V, dist: Int, branch: V):
        for v in held[c]:
            let x = anc[v][cdep[v] - cdep[c]]
            yield (v: v, dist: x.dist, branch: x.branch)

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
    domain V = 1..=14
    for _ in 0..10:
        let adj = random_tree(V.first)
        let r = Rooted(root: V.nth(rand(0, 13)), adj: v => adj[v])
        assert r.size(r.root) == 14 and r.parent(r.root) == none
        for v in V:
            assert r.at[r.tin[v]] == v
            assert r.sub(v).len == r.size(v)
            for u in V:
                var x = u
                while x != v and r.parent(x) != none: x = r.parent(x)!
                assert r.is_anc(v, u) == (x == v)
            if let h = r.heavy[v]:
                assert r.tin[h].ord == r.tin[v].ord + 1 and r.top[h] == r.top[v]

@test
fn a_table_of_neighbour_lists_roots_like_the_fn_that_reads_it():
    domain V = 0..9
    let adj = random_tree(V.first)
    let (a, b) = (Rooted(adj, V.first), Rooted(v => adj[v], V.first))
    assert [v for v in a.at] == [v for v in b.at] and a.par == b.par and a.sz == b.sz

@test
fn lca_dist_ancestor_and_step_match_climbing():
    domain V = 0..13
    for _ in 0..10:
        let adj = random_tree(V.first)
        let r = Rooted(root: V.nth(rand(0, 12)), adj: v => adj[v])
        for a in V:
            for b in V:
                let path = slow_path(r, a, b)
                assert r.dist(a, b) == path.len - 1
                let top = r.lca(a, b)
                assert top in path and min(r.depth(v) for v in path) == r.depth(top)
                for k in 0..path.len: assert r.step(a, b, k) == path[k]
                assert r.step(a, b, path.len) == none
            assert r.ancestor(a, r.depth(a)) == r.root
            assert r.ancestor(a, r.depth(a) + 1) == none

@test
fn weighted_dist_adds_the_lengths_on_the_path():
    domain V = 0..5
    // 0 -3- 1 -4- 2, 1 -10- 3, 0 -7- 4
    let roads = [(V(0), V(1), 3), (V(1), V(2), 4), (V(1), V(3), 10), (V(0), V(4), 7)]
    var adj: [V]List<(to: V, len: Int)> = []
    for (u, v, len) in roads:
        adj[u].push((to: v, len: len))
        adj[v].push((to: u, len: len))
    let r = Rooted.weighted(v => adj[v], V(0), a => a.len)
    assert [r.far[v] for v in V] == [0, 3, 7, 13, 7] and r.depth(V(3)) == 2
    assert r.dist(V(2), V(3)) == 14 and r.dist(V(4), V(2)) == 14 and r.dist(V(3), V(3)) == 0
    let plain = Rooted(root: V(0), adj: v => adj[v])
    assert plain.dist(V(2), V(3)) == 2 and plain.far[V(3)] == plain.depth(V(3))

@test
fn path_folds_keep_the_order_from_a_to_b():
    domain V = 0..12
    for _ in 0..12:
        let adj = random_tree(V(0))
        let r = Rooted(root: V.nth(rand(0, 11)), adj: v => adj[v])
        var name = [V] v => Word(Str([Char(97 + v.to_idx)]))
        var st = SegTree(Monoid<Word>(Word(""), (a, b) => Word(a.s + b.s)), r.reorder(name))
        for _ in 0..12:
            let (a, b) = (V.nth(rand(0, 11)), V.nth(rand(0, 11)))
            let path = slow_path(r, a, b)
            assert r.fold_path(a, b, st).s == Str([name[v].s[0] for v in path])
            let lower = [v for v in path if v != r.lca(a, b)]
            assert r.fold_path(a, b, st, edges: true).s == Str([name[v].s[0] for v in lower])
            let v = V.nth(rand(0, 11))
            name[v] = Word(Str([Char(65 + rand(0, 25))]))
            st.set(r.pos(v), name[v])

struct Word(s: Str)

impl Reversible for Word:
    fn rev(self) -> Word = Word(Str([s[s.len - 1 - i] for i in 0..s.len]))

@test
fn path_updates_and_subtree_folds_over_one_lazy_tree():
    domain V = 0..11
    for _ in 0..10:
        let adj = random_tree(V(0))
        let r = Rooted(root: V(0), adj: v => adj[v])
        var val = [V] v => rand(0, 9)
        var st = LazySegTree(AddMax, r.reorder(val))
        for _ in 0..20:
            let (a, b, x) = (V.nth(rand(0, 10)), V.nth(rand(0, 10)), rand(-3, 3))
            for v in slow_path(r, a, b): val[v] += x
            r.apply_path(a, b, mut st, x)
            let (c, d) = (V.nth(rand(0, 10)), V.nth(rand(0, 10)))
            assert r.fold_path(c, d, st) == max(val[v] for v in slow_path(r, c, d))
            assert st.fold(r.sub(c)) == max(val[v] for v in V if r.is_anc(c, v))

@test
fn centroids_leave_no_piece_over_half():
    domain V = 0..9
    for _ in 0..40:
        let adj = random_tree(V(0))
        let r = Rooted(root: V(0), adj: v => adj[v])
        let good = [
            v for v in V if 2 * (V.len - r.size(v)) <= V.len and all(
                2 * r.size(c) <= V.len for c in r.children(v)
            )
        ]
        assert r.centroid in good
        var got = r.centroids
        got.sort()
        assert got == good
        assert r.centroids[0] == r.centroid
        assert all(r.subtree_sizes[v] == r.size(v) for v in V)

@test
fn excl_fold_folds_the_other_children():
    domain V = 0..12
    for _ in 0..10:
        let adj = random_tree(V(0))
        let r = Rooted(root: V(0), adj: v => adj[v])
        let w = [V] v => rand(0, 9)
        for v in V:
            let cs = r.children(v)
            var seen = List<V>()
            for (c, rest) in r.excl_fold(v, +, c => w[c]):
                seen.push(c)
                assert rest == sum(w[x] for x in cs if x != c)
            assert seen == cs
            // order is kept for a non-commutative fold: earlier siblings, then later ones
            let cat = Monoid<Str>("", (a, b) => a + b)
            for (c, rest) in r.excl_fold(v, cat, c => "{c.to_idx},"):
                assert rest == Str(ch for x in cs if x != c for ch in "{x.to_idx},")

@test
fn rerooting_counts_connected_sets_through_every_vertex():
    // AtCoder DP V: the connected vertex sets containing v, for every v, against a brute force.
    domain V = 0..7
    let mod = 1_000_000_007
    let mul = Monoid<Int>(1, (a, b) => a * b % mod)
    for _ in 0..5:
        let adj = random_tree(V(0))
        let t = Rooted(root: V(0), adj: v => adj[v])
        var down: [V]Int = 1
        for v in t.at.rev:
            for c in t.children(v): down[v] = down[v] * (down[c] + 1) % mod
        var up: [V]Int = 1
        for v in t.at:
            for (c, rest) in t.excl_fold(v, mul, c => down[c] + 1):
                up[c] = (up[v] * rest + 1) % mod
        for v in V:
            var brute = 0
            for mask in 0..(1 << V.K):
                if (mask >> v.to_idx) & 1 == 0: continue
                // connected iff every chosen vertex but the top one has a chosen parent
                var tops = 0
                for x in V:
                    if (mask >> x.to_idx) & 1 == 1:
                        let p = t.parent(x)
                        if p == none or (mask >> p!.to_idx) & 1 == 0: tops += 1
                if tops == 1: brute += 1
            assert down[v] * up[v] % mod == brute

@test
fn orders_lifting_and_the_virtual_tree():
    domain V = 0..13
    for _ in 0..15:
        let adj = random_tree(V(0))
        let r = Rooted(root: V.nth(rand(0, 12)), adj: v => adj[v])
        var seen: [V]Bool = false
        for v in r.at.rev:
            for c in r.children(v): assert seen[c]
            seen[v] = true
        seen.fill(false)
        for v in r.at:
            assert r.parent(v) == none or seen[r.parent(v)!]
            seen[v] = true
        for v in V:
            let cap = rand(0, 4)
            let got = r.lift_while(v, u => r.depth(v) - r.depth(u) <= cap)
            assert got == r.ancestor(v, min(cap, r.depth(v)))
            assert r.lift_while(v, u => false) == none
        let keys: List<V> = [v for v in V if rand(0, 2) == 0]
        let vt = r.virtual_tree(keys)
        var kept: [V]Bool = false
        for a in keys:
            for b in keys: kept[r.lca(a, b)] = true
        assert vt.nodes.sorted() == [v for v in V if kept[v]]
        assert vt.edges.len == max(vt.nodes.len - 1, 0)
        for e in vt.edges:
            assert r.is_anc(e.u, e.v) and e.u != e.v
            // nothing kept lies strictly between a node and its parent
            assert not any(
                kept[w] and w != e.u and w != e.v and r.is_anc(e.u, w) and r.is_anc(
                    w,
                    e.v,
                ) for w in V
            )

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
    domain V = 1..=6
    let par = [V] v => [none, V(1), V(1), V(2), V(2), V(5)][v.to_idx]
    let t1 = Rooted.from_parents(par)
    let t2 = Rooted.from_parents([V] v => [V(2), none, V(2), V(1), V(3), V(5)][v.to_idx])
    assert t1.root == V(1) and t2.root == V(2)
    assert t1.lca(V(4), V(6)) == V(2) and t1.dist(V(4), V(6)) == 3
    for v in V: assert t1.at[t1.pos(v)] == v
    // Both trees have independent value-owned Pre domains
    var a1: [t1.Pre]Int = 0
    for p in t1.Pre: a1[p] = t1.at[p].to_idx
    var a2: [t2.Pre]Int = 0
    for p in t2.Pre: a2[p] = t2.at[p].to_idx
    assert a1[t1.pos(V(6))] == 5 and a2[t2.pos(V(6))] == 5
    // Iterative enter/exit tour balances parentheses and matches subtree nesting
    var entered: List<V> = []
    var exited: List<V> = []
    var stack: List<V> = []
    for (v, entering) in t1.tour:
        if entering:
            entered.push(v)
            stack.push(v)
        else:
            assert stack.pop() == v
            exited.push(v)
    assert stack.is_empty and entered.len == V.K and exited.len == V.K
    assert entered[0] == t1.root and exited[exited.len - 1] == t1.root
    // Every vertex is entered before its children and exited after its children
    var ent_pos: [V]Int = 0
    var ext_pos: [V]Int = 0
    for i in 0..V.K:
        ent_pos[entered[i]] = i
        ext_pos[exited[i]] = i
    for v in V:
        for c in t1.children(v):
            assert ent_pos[v] < ent_pos[c] and ext_pos[c] < ext_pos[v]

@test
fn weighted_rooted_bfs_tree_on_cyclic_graph():
    domain V = 0..6
    // Cycle + chords + self-loop: the breadth-first tree puts the ends of every edge at most one level apart
    let raw = [
        (0, 1, 3),
        (0, 2, 5),
        (1, 2, 7),
        (1, 3, 2),
        (2, 4, 4),
        (3, 4, 6),
        (4, 5, 1),
        (0, 5, 9),
        (2, 2, 1),
    ]
    var g: [V]List<(to: V, w: Int)> = []
    for (u, v, w) in raw:
        g[V(u)].push((to: V(v), w: w))
        if u != v: g[V(v)].push((to: V(u), w: w))
    let r = Rooted.weighted(u => g[u], root: V(0), len: a => a.w)
    assert r.parent(V(0)) == none
    for (u, v, _) in raw:
        assert abs(r.depth(V(u)) - r.depth(V(v))) <= 1
    assert r.children(V(0)) == [V(1), V(2), V(5)] and r.children(V(2)) == [V(4)]
    assert r.dist(V(0), V(2)) == 5 and r.dist(V(0), V(5)) == 9 and r.dist(V(0), V(4)) == 9

@test
fn lower_is_the_child_end_of_an_edge():
    domain V = 0..9
    let adj = random_tree(V.first)
    let r = Rooted(adj, V.first)
    for v in V:
        if let p = r.parent(v):
            assert r.lower(v, p) == v and r.lower(p, v) == v

@test
fn path_and_diameter_match_pairwise_distances():
    domain V = 0..12
    let adj = random_tree(V.first)
    let r = Rooted(adj, V.first)
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
    domain V = 0..12
    for _ in 0..10:
        let adj = random_tree(V(0))
        let r = Rooted(root: V.nth(rand(0, 11)), adj: v => adj[v])
        for v in V:
            var cs = [c for c in r.light(v)]
            if let h = r.heavy[v]:
                assert all(r.size(c) <= r.size(h) for c in r.children(v))
                cs.push(h)
            assert cs.sorted() == r.children(v).sorted()
            let sub = [u for u in r.subtree(v)]
            assert sub[0] == v and sub.sorted() == [u for u in V if r.is_anc(v, u)]
            assert [r.at[p] for p in r.below(v)].sorted() == sub[1..].sorted()
            assert r.root_of(v) == r.root

@test
fn a_forest_keeps_each_tree_in_one_range():
    domain V = 0..10
    // 0-1-2, 3-4, 5-6-7-8, 9 alone
    let pairs = [(0, 1), (1, 2), (3, 4), (5, 6), (6, 7), (6, 8)]
    var adj: [V]List<V> = []
    for (a, b) in pairs:
        adj[V(a)].push(V(b))
        adj[V(b)].push(V(a))
    let roots = [V(0), V(3), V(5), V(9)]
    let f = Rooted.forest(v => adj[v], roots)
    assert f.root == V(0) and [f.root_of(v).to_idx for v in V] == [0, 0, 0, 3, 3, 5, 5, 5, 5, 9]
    assert [f.pos(r).ord for r in roots] == [0, 3, 5, 9]
    assert f.size(V(5)) == 4 and f.lca(V(7), V(8)) == V(6) and f.dist(V(2), V(0)) == 2
    assert all(f.parent(r) == none for r in roots) and f.at.len == 10
    let depth = f.down(0, (d, v) => if f.parent(v) == none: 0 else: d + 1)
    assert all(depth[v] == f.depth(v) for v in V)

@test
fn down_and_up_match_depth_size_and_height():
    domain V = 0..12
    for _ in 0..10:
        let adj = random_tree(V(0))
        let r = Rooted(root: V.nth(rand(0, 11)), adj: v => adj[v])
        let w = [V] v => rand(0, 9)
        let best = r.down(0, (m, v) => max(m, w[v]))
        let size = r.up(+, (s, c, p) => s, (s, v) => s + 1)
        let height = r.up(max, (h, c, p) => h + 1, (h, v) => max(h, 0))
        for v in V:
            assert best[v] == max(w[u] for u in slow_path(r, r.root, v))
            assert size[v] == r.size(v)
            assert height[v] == max(r.depth(u) - r.depth(v) for u in r.subtree(v))

@test
fn reroot_matches_up_from_every_root():
    domain V = 0..10
    let cat = Monoid<Str>("", (a, b) => a + b)
    for _ in 0..6:
        let adj = random_tree(V(0))
        let r = Rooted(root: V.nth(rand(0, 9)), adj: v => adj[v])
        // far: the farthest vertex; shape: a child-order-sensitive fold, to check the order
        let far = r.reroot(max, (h, c, p) => h + 1, (h, v) => max(h, 0))
        let shape = r.reroot(cat, (s, c, p) => s, (s, v) => "(" + s + ")")
        let label = r.reroot(cat, (s, c, p) => s, (s, v) => "{v.to_idx}" + s)
        assert shape[r.root] == r.up(cat, (s, c, p) => s, (s, v) => "(" + s + ")")[r.root]
        for v in V:
            let at = Rooted(root: v, adj: u => adj[u])
            assert far[v] == max(r.dist(v, u) for u in V)
            assert at.up(max, (h, c, p) => h + 1, (h, u) => max(h, 0))[v] == far[v]
            assert shape[v].len == 2 * V.K and label[v].len == at.up(
                cat,
                (s, c, p) => s,
                (s, u) => "{u.to_idx}" + s,
            )[v].len

@test
fn shapes_match_the_canonical_string():
    domain V = 0..14
    for _ in 0..10:
        let adj = random_tree(V(0))
        let r = Rooted(root: V.nth(rand(0, 13)), adj: v => adj[v])
        var canon: [V]Str = ""
        for v in r.at.rev:
            var parts = [canon[c] for c in r.children(v)]
            parts.sort()
            canon[v] = "(" + Str(ch for s in parts for ch in s) + ")"
        let sh = r.shapes()
        for a in V:
            for b in V: assert (sh.of[a] == sh.of[b]) == (canon[a] == canon[b])

@test
fn centroid_ancestors_split_every_path():
    domain V = 0..16
    for _ in 0..10:
        let adj = random_tree(V(0))
        let r = Rooted(adj, V(0))
        let cd = Centroids<V>(adj)
        let same = Centroids<V>(v => adj[v])
        assert all(same.parent(v) == cd.parent(v) for v in V)
        assert cd.parent(cd.root) == none and cd.depth(cd.root) == 0
        for v in V:
            let anc = [x for x in cd.ancestors(v)]
            assert anc[0] == (c: v, dist: 0) and anc.len == cd.depth(v) + 1
            assert (1 << (anc.len - 1)) <= V.K
            for i in 0..anc.len:
                assert r.dist(v, anc[i].c) == anc[i].dist
                if i + 1 < anc.len: assert cd.parent(anc[i].c) == some(anc[i + 1].c)
        for c in V:
            var count = 0
            for (v, d, b) in cd.piece(c):
                count += 1
                assert r.dist(c, v) == d and (b == c) == (v == c)
                assert v == c or (r.dist(c, b) == 1 and r.dist(b, v) == d - 1)
            assert count == [v for v in V if any(x.c == c for x in cd.ancestors(v))].len

@test
fn a_jump_tree_answers_what_the_rooted_tree_does():
    domain V = 0..40
    for _ in 0..20:
        let par = [V] v => if v == V.first: none else: some(V(rand(0, v.ord - 1)))
        let r = Rooted.from_parents(par)
        var j = JumpTree<V>()
        for v in V:
            if let p = par[v]: j.add(v, p)
        for v in V:
            assert j.parent(v) == r.parent(v) and j.depth(v) == r.depth(v) and j.root_of(v) == V.first
            for k in -1..=r.depth(v) + 1: assert j.ancestor(v, k) == r.ancestor(v, k)
            let cap = rand(0, 8)
            assert j.lift_while(
                v,
                u => r.depth(v) - r.depth(u) <= cap,
            ) == r.ancestor(v, min(cap, r.depth(v)))
            assert j.lift_while(v, u => false) == none
            for u in V: assert j.lca(u, v) == some(r.lca(u, v))
    var f = JumpTree<V>()
    f.add(V(1), V(0))
    f.add(V(3), V(2))
    assert f.lca(V(1), V(0)) == some(V(0)) and f.lca(V(1), V(3)) == none and f.root_of(V(3)) == V(2)

/// A rooted forest grown one leaf at a time, by skew-binary jump pointers: `add` is O(1),
/// queries are O(log n), and it keeps three arrays. Every vertex starts as its own root.
pub struct JumpTree<V>:
    par: [V]?V
    jump: [V]V
    dep: [V]Int

    /// A forest of lone vertices, each its own root. O(n).
    pub fn new() -> JumpTree<V> = JumpTree([V] _ => none, [V] v => v, [V] _ => 0)

    /// Hang `v`, a vertex not yet in any tree, below `p`. O(1).
    pub fn add(mut self, v: V, p: V):
        assert par[v] == none and jump[v] == v and v != p
        par[v] = some(p)
        dep[v] = dep[p] + 1
        let j = jump[p]
        jump[v] = if dep[p] - dep[j] == dep[j] - dep[jump[j]]: jump[j] else: p

    /// The parent of `v`, or `none` at a root. O(1).
    pub fn parent(self, v: V) -> ?V = par[v]

    /// The number of edges from `v` up to its root. O(1).
    pub fn depth(self, v: V) -> Int = dep[v]

    /// The root of `v`'s tree. O(log n).
    pub fn root_of(self, v: V) -> V:
        var x = v
        while jump[x] != x: x = jump[x]
        x

    /// The ancestor `k` edges above `v`, or `none` if that is above the root. O(log n).
    pub fn ancestor(self, v: V, k: Int) -> ?V:
        if k < 0 or k > dep[v]: return none
        let d = dep[v] - k
        var x = v
        while dep[x] > d: x = if dep[jump[x]] >= d: jump[x] else: par[x]!
        x

    /// The lowest common ancestor of `a` and `b`, or `none` in different trees. O(log n).
    pub fn lca(self, a: V, b: V) -> ?V:
        var x = if dep[a] > dep[b]: self.ancestor(a, dep[a] - dep[b])! else: a
        var y = if dep[b] > dep[a]: self.ancestor(b, dep[b] - dep[a])! else: b
        while x != y:
            if par[x] == none: return none
            if jump[x] != jump[y]:
                x = jump[x]
                y = jump[y]
            else:
                x = par[x]!
                y = par[y]!
        x

    /// The highest ancestor `u` of `v` such that `pred` holds for `u`, or `none` when it does not
    /// hold for `v` itself. `pred` must be true and then false on the way up. O(log n) calls.
    pub fn lift_while(self, v: V, pred: V -> Bool) -> ?V:
        if not pred(v): return none
        var x = v
        while true:
            if jump[x] != x and pred(jump[x]): x = jump[x]
            elif par[x] != none and pred(par[x]!): x = par[x]!
            else: return x
        x

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

    /// `V.K` single-vertex trees holding `a` (the identity of `op` when `a` is absent). O(n).
    pub fn new<M>(op: M, a: ?[V]T = none) -> LinkCut<V, T>:
        let m = Monoid<T>.of(op)
        let val = if let init = a: init else: [V] _ => m.id
        LinkCut(m, [V] _ => none, [V] _ => none, [V] _ => none, [V] _ => false, val, val, [])

    fn top(self, x: V) -> Bool:
        if let p = pa[x]: lc[p] != some(x) and rc[p] != some(x) else: true

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
        if let c = lc[x]: s = m.op(agg[c], s)
        if let c = rc[x]: s = m.op(s, agg[c])
        agg[x] = s

    fn rotate(mut self, x: V):
        let p = pa[x]!
        let g = pa[p]
        let p_top = self.top(p)
        if lc[p] == some(x):
            lc[p] = rc[x]
            if let c = rc[x]: pa[c] = some(p)
            rc[x] = some(p)
        else:
            rc[p] = lc[x]
            if let c = lc[x]: pa[c] = some(p)
            lc[x] = some(p)
        pa[p] = some(x)
        pa[x] = g
        if not p_top, let q = g:
            if lc[q] == some(p): lc[q] = some(x) else: rc[q] = some(x)
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
                if (lc[g] == some(p)) == (lc[p] == some(x)): self.rotate(p) else: self.rotate(x)
            self.rotate(x)

    // Makes root -> x the preferred path with x at the root of its splay tree; returns the
    // last vertex where the walk joined the old preferred path (the lca with the previous access).
    fn access(mut self, x: V) -> V:
        var last: ?V = none
        var joined = x
        var y: ?V = some(x)
        while let u = y:
            self.splay(u)
            rc[u] = last
            self.pull(u)
            last = some(u)
            joined = u
            y = pa[u]
        self.splay(x)
        joined

    /// Make `v` the root of its tree. Amortized O(log n).
    pub fn make_root(mut self, v: V):
        let _ = self.access(v)
        self.turn(v)

    /// The root of the tree holding `v`. Amortized O(log n).
    pub fn root_of(mut self, v: V) -> V:
        let _ = self.access(v)
        var x = v
        self.push(x)
        while let c = lc[x]:
            x = c
            self.push(x)
        self.splay(x)
        x

    /// Whether `u` and `v` are in the same tree. Amortized O(log n).
    pub fn connected(mut self, u: V, v: V) -> Bool = self.root_of(u) == self.root_of(v)

    /// Add the edge `u`-`v`; they must be in different trees, which costs one extra `root_of`. Amortized O(log n).
    pub fn link(mut self, u: V, v: V):
        self.make_root(u)
        assert self.root_of(v) != u, "link joins two trees; these vertices are already connected"
        pa[u] = some(v)

    /// Remove the edge `u`-`v`, which must exist. Amortized O(log n).
    pub fn cut(mut self, u: V, v: V):
        self.make_root(u)
        let _ = self.access(v)
        assert lc[v] == some(u) and rc[u] == none, "cut needs an edge between these vertices"
        lc[v] = none
        pa[u] = none
        self.pull(v)

    /// The lowest common ancestor of `u` and `v` when their tree is rooted at `root`, which
    /// becomes its root; `none` unless all three are connected. Amortized O(log n).
    pub fn lca(mut self, u: V, v: V, root: V) -> ?V:
        self.make_root(root)
        if self.root_of(u) != root or self.root_of(v) != root: return none
        let _ = self.access(u)
        some(self.access(v))

    /// The value at `v`. O(1).
    pub fn get(self, v: V) -> T = val[v]

    /// Set the value at `v` to `x`. Amortized O(log n).
    pub fn set(mut self, v: V, x: T):
        let _ = self.access(v)
        val[v] = x
        self.pull(v)

    /// The fold of the values on the path from `u` to `v`, in that order; `u` becomes the root.
    /// Amortized O(log n).
    pub fn fold_path(mut self, u: V, v: V) -> T:
        self.make_root(u)
        let _ = self.access(v)
        agg[v]

/// `LinkCut` with updates along a path: `LazyLinkCut.new(AddSum, a)` adds to every value on
/// a path and folds paths, under the same `Action` as `LazySegTree`. Reversing a path must not
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
    cnt: [V]Int
    lz: [V]F
    stk: List<V>

    /// `V.K` single-vertex trees holding `a` (the identity of `act.over` when `a` is absent). O(n).
    pub fn new(act: Action<T, F>, a: ?[V]T = none) -> LazyLinkCut<V, T, F>:
        let val = if let init = a: init else: [V] _ => act.over.id
        LazyLinkCut(
            act,
            [V] _ => none,
            [V] _ => none,
            [V] _ => none,
            [V] _ => false,
            val,
            val,
            [V] _ => 1,
            [V] _ => act.id,
            [],
        )

    fn top(self, x: V) -> Bool:
        if let p = pa[x]: lc[p] != some(x) and rc[p] != some(x) else: true

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
        if lc[p] == some(x):
            lc[p] = rc[x]
            if let c = rc[x]: pa[c] = some(p)
            rc[x] = some(p)
        else:
            rc[p] = lc[x]
            if let c = lc[x]: pa[c] = some(p)
            lc[x] = some(p)
        pa[p] = some(x)
        pa[x] = g
        if not p_top, let q = g:
            if lc[q] == some(p): lc[q] = some(x) else: rc[q] = some(x)
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
                if (lc[g] == some(p)) == (lc[p] == some(x)): self.rotate(p) else: self.rotate(x)
            self.rotate(x)

    fn access(mut self, x: V) -> V:
        var last: ?V = none
        var joined = x
        var y: ?V = some(x)
        while let u = y:
            self.splay(u)
            rc[u] = last
            self.pull(u)
            last = some(u)
            joined = u
            y = pa[u]
        self.splay(x)
        joined

    /// Make `v` the root of its tree. Amortized O(log n).
    pub fn make_root(mut self, v: V):
        let _ = self.access(v)
        self.turn(v)

    /// The root of the tree holding `v`. Amortized O(log n).
    pub fn root_of(mut self, v: V) -> V:
        let _ = self.access(v)
        var x = v
        self.push(x)
        while let c = lc[x]:
            x = c
            self.push(x)
        self.splay(x)
        x

    /// Whether `u` and `v` are in the same tree. Amortized O(log n).
    pub fn connected(mut self, u: V, v: V) -> Bool = self.root_of(u) == self.root_of(v)

    /// Add the edge `u`-`v`; they must be in different trees, which costs one extra `root_of`. Amortized O(log n).
    pub fn link(mut self, u: V, v: V):
        self.make_root(u)
        assert self.root_of(v) != u, "link joins two trees; these vertices are already connected"
        pa[u] = some(v)

    /// Remove the edge `u`-`v`, which must exist. Amortized O(log n).
    pub fn cut(mut self, u: V, v: V):
        self.make_root(u)
        let _ = self.access(v)
        assert lc[v] == some(u) and rc[u] == none, "cut needs an edge between these vertices"
        lc[v] = none
        pa[u] = none
        self.pull(v)

    /// The lowest common ancestor of `u` and `v` when their tree is rooted at `root`, which
    /// becomes its root; `none` unless all three are connected. Amortized O(log n).
    pub fn lca(mut self, u: V, v: V, root: V) -> ?V:
        self.make_root(root)
        if self.root_of(u) != root or self.root_of(v) != root: return none
        let _ = self.access(u)
        some(self.access(v))

    /// The value at `v`, with every update along paths through it applied. Amortized O(log n).
    pub fn get(mut self, v: V) -> T:
        let _ = self.access(v)
        val[v]

    /// Set the value at `v` to `x`. Amortized O(log n).
    pub fn set(mut self, v: V, x: T):
        let _ = self.access(v)
        val[v] = x
        self.pull(v)

    /// The fold of the values on the path from `u` to `v`, in that order; `u` becomes the root.
    /// Amortized O(log n).
    pub fn fold_path(mut self, u: V, v: V) -> T:
        self.make_root(u)
        let _ = self.access(v)
        agg[v]

    /// Apply `f` to every value on the path from `u` to `v`; `u` becomes the root. Amortized O(log n).
    pub fn apply_path(mut self, u: V, v: V, f: F):
        self.make_root(u)
        let _ = self.access(v)
        self.put(v, f)

@test
fn link_cut_follows_a_changing_forest():
    domain V = 1..=6
    var lct = LinkCut<V, Int>.new(+, [V] v => v.to_idx + 1)
    lct.link(V(1), V(2))
    lct.link(V(2), V(3))
    lct.link(V(4), V(3))
    assert lct.connected(V(1), V(4)) and not lct.connected(V(1), V(5))
    assert lct.fold_path(V(1), V(4)) == 10
    assert lct.lca(V(3), V(4), V(1)) == some(V(3)) and lct.lca(V(2), V(4), V(1)) == some(V(2))
    assert lct.lca(V(1), V(4), V(3)) == some(V(3)) and lct.lca(V(1), V(5), V(1)) == none
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
        var from: [V]?V = none
        var seen: [V]Bool = false
        seen[u] = true
        var st = [u]
        while let x = st.pop():
            for y in adj[x]:
                if not seen[y]:
                    seen[y] = true
                    from[y] = some(x)
                    st.push(y)
        assert lct.connected(u, v) == seen[v]
        if seen[v] and u != v:
            var acc = MaxSub.of(w[v])
            var x = v
            while let p = from[x]:
                acc = MaxSub.of(w[p]) + acc
                x = p
            assert lct.fold_path(u, v) == acc
            if rng.int(0, 2) == 0 and from[v] != none:
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
    var lct = LazyLinkCut<V, Int, Int>.new(AddSum, w)
    var adj: [V]List<V> = []
    for _ in 0..600:
        let u = V(rng.int(1, 9))
        let v = V(rng.int(1, 9))
        var from: [V]?V = none
        var seen: [V]Bool = false
        seen[u] = true
        var st = [u]
        while let x = st.pop():
            for y in adj[x]:
                if not seen[y]:
                    seen[y] = true
                    from[y] = some(x)
                    st.push(y)
        assert lct.connected(u, v) == seen[v]
        if not seen[v]:
            lct.link(u, v)
            for (x, y) in [(u, v), (v, u)]: adj[x].push(y)
            continue
        var path = [v]
        while let p = from[path[path.len - 1]]: path.push(p)
        let k = rng.int(0, 3)
        if k == 0:
            let d = rng.int(-3, 3)
            lct.apply_path(u, v, d)
            for x in path: w[x] += d
        elif k == 1:
            assert lct.fold_path(u, v) == [w[x] for x in path].sum()
        elif k == 2:
            assert lct.get(v) == w[v]
        elif from[v] != none:
            let p = from[v]!
            lct.cut(v, p)
            adj[v] = [y for y in adj[v] if y != p]
            adj[p] = [y for y in adj[p] if y != v]

@test
fn lazy_link_cut_keeps_path_order_under_assignment():
    domain V = 1..=6
    let paint = Action<MaxSub, ?Int>(
        none,
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
    lct.apply_path(V(2), V(3), some(-2))
    assert lct.fold_path(V(1), V(6)).best == 5 and lct.fold_path(
        V(6),
        V(1),
    ).total == 3 - 2 - 2 - 1 + 5 - 9
    lct.apply_path(V(5), V(4), some(2))
    assert lct.fold_path(V(1), V(5)).best == 4 and lct.fold_path(V(4), V(2)).pre == 2
    assert lct.get(V(3)) == MaxSub.of(-2) and lct.lca(V(2), V(6), V(4)) == some(V(4))

/// How a permutation-tree node joins its two children `l` (earlier positions) and `r`.
/// `Incr`: `r`'s values sit just above `l`'s; `Decr`: just below; `Partial`: the union is
/// not an interval on its own and only closes further up; `Full`: `r` fills the gaps of a
/// `Partial` `l`, closing an interval in neither order.
pub enum PermKind: Leaf, Incr, Decr, Partial, Full

/// The binary permutation (interval-decomposition) tree of a bijection `p: [I]V`: every node
/// covers a run of positions, leaves are single positions, a node's children split its run in
/// two, and the values of every node but a `Partial` one form an interval. Nodes have their own
/// domain `t.Node` of 2n - 1 members; `kind`, `first`, `last`, `lo` and `hi` are arrays over it. Built in O(n) by one stack pass.
pub struct PermTree<I, V, Node>:
    root: Node
    kind: [Node]PermKind
    l: [Node]Node
    r: [Node]Node
    first: [Node]I
    last: [Node]I
    lo: [Node]V
    hi: [Node]V

    /// The tree of `p`, which must be a bijection between two domains of equal size. O(n).
    pub fn of(p: [I]V):
        let n = I.K
        assert n == V.K and n > 0, "PermTree needs a bijection between equal non-empty domains"
        domain N = 0..2 * n - 1
        var prv = [0..n] v => v - 1
        var nxt = [0..n] v => v + 1
        for i in I.rev:
            let x = p[i].ord
            let (a, b) = (prv[x], nxt[x])
            if a >= 0: nxt[a] = b
            if b < n: prv[b] = a
        var kind: [N]PermKind = Leaf
        var l: [N]N = N.first
        var r: [N]N = N.first
        var first: [N]I = I.first
        var last: [N]I = I.first
        var lo: [N]V = V.first
        var hi: [N]V = V.first
        type Open = (left: Int, lo: Int, lo_gap: Int, hi: Int, hi_gap: Int, node: Int)
        var stk = List<Open>()
        fn join(u: Open, t: Open, k: PermKind, end: Int) -> Open:
            let id = N(2 * t.left - 1)
            kind[id] = k
            l[id] = N(u.node)
            r[id] = N(t.node)
            first[id] = I.nth(u.left)
            last[id] = I.nth(end)
            let (a, b) = (min(u.lo, t.lo), max(u.hi, t.hi))
            lo[id] = V.nth(a)
            hi[id] = V.nth(b)
            (left: u.left, lo: a, lo_gap: u.lo_gap, hi: b, hi_gap: u.hi_gap, node: id.ord)
        for i in I:
            let (j, x) = (i.ord, p[i].ord)
            while let top = stk.last, x < top.lo_gap or x > top.hi_gap:
                let t = stk.pop()!
                stk.push(join(stk.pop()!, t, Partial, j - 1))
            let leaf = N(2 * j)
            first[leaf] = i
            last[leaf] = i
            lo[leaf] = p[i]
            hi[leaf] = p[i]
            var t = (left: j, lo: x, lo_gap: prv[x] + 1, hi: x, hi_gap: nxt[x] - 1, node: 2 * j)
            while let u = stk.last, max(u.hi, t.hi) - min(u.lo, t.lo) == j - u.left:
                let k = if t.lo < u.lo: Decr elif t.hi > u.hi: Incr else: Full
                t = join(stk.pop()!, t, k, j)
            stk.push(t)
        assert stk.len == 1
        PermTree<I, V, N>(N(stk[0].node), kind, l, r, first, last, lo, hi)

    /// The leaf of position `i`. O(1).
    pub fn leaf(self, i: I) -> Node = Node(2 * i.ord)

    /// The earlier child of `v`, an inner node. O(1).
    pub fn left(self, v: Node) -> Node:
        assert kind[v] != Leaf, "a leaf has no children"
        l[v]

    /// The later child of `v`, an inner node. O(1).
    pub fn right(self, v: Node) -> Node:
        assert kind[v] != Leaf, "a leaf has no children"
        r[v]

    /// The positions `v` covers. O(1).
    pub fn span(self, v: Node) -> Range<I> = first[v].l..last[v].r

    /// From the least to the greatest value `v` covers: exactly its values unless `v` is `Partial`. O(1).
    pub fn vals(self, v: Node) -> Range<V> = lo[v].l..hi[v].r

@test
fn perm_tree_nodes_cover_intervals_of_values():
    var rng = Rng(5)
    for _ in 0..200:
        let n = rand(1, 12)
        domain I = 0..n
        var vs = [i for i in 0..n]
        rng.shuffle(mut vs)
        let p = [I] i => I(vs[i.ord])
        let t = PermTree.of(p)
        assert t.span(t.root) == I.first.l..I.last.r
        var leaves = 0
        for v in t.Node:
            let s = t.span(v)
            let w = t.vals(v)
            assert (s.len == w.len) == (t.kind[v] != Partial)
            for i in s: assert w.contains(p[i])
            if t.kind[v] == Leaf:
                leaves += 1
                assert t.leaf(t.first[v]) == v and s.len == 1
            else:
                let (a, b) = (t.left(v), t.right(v))
                assert t.span(a).r == t.span(b).l and t.span(a).l == s.l and t.span(b).r == s.r
                if t.kind[v] == Incr: assert t.vals(a).r == t.vals(b).l
                if t.kind[v] == Decr: assert t.vals(b).r == t.vals(a).l
        assert leaves == n

@test
fn perm_tree_of_a_small_permutation():
    domain I = 0..4
    let p: [I]I = [I] i => I([1, 0, 3, 2][i.ord])
    let t = PermTree.of(p)
    assert t.kind[t.root] == Incr
    assert t.kind[t.left(t.root)] == Decr and t.span(t.left(t.root)) == I(0).l..I(1).r
