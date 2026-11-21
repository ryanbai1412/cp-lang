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
use std.algebra.{Monoid, MaxSub}
use std.segtree.{SegTree}
use std.lazy.{LazySegTree, AddMax}
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
    kid: [V]List<V>
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
    /// depth-first spanning tree, so on a graph with extra edges every other edge joins a
    /// vertex to one of its ancestors. Stops if some vertex is not reachable from the root.
    pub fn new<N>(adj: N, root: V) = Rooted.weighted(adj, root, a => 1)

    /// Build a rooted tree directly from a parent array `par: [V]?V` where the unique vertex
    /// with `par[v] == none` is the root. O(n).
    pub fn from_parents(par: [V]?V):
        var root: ?V = none
        var kids: [V]List<V> = []
        for v in V:
            if let p = par[v]: kids[p].push(v)
            else:
                assert root == none
                root = v
        assert root != none
        Rooted.weighted(u => kids[u], root!, a => 1)

    /// The same for edges with lengths: `len(arc)` is the length of the edge an arc runs along,
    /// and `dist` adds lengths up instead of counting edges.
    pub fn weighted<N, A>(adj: N, root: V, len: A -> Int) = Rooted.build(adj, [root], len)

    /// One tree per root: `adj` must reach every vertex from exactly one of `roots`. The trees
    /// sit one after another in `t.Pre`, in the order of `roots`; `t.root` is the first root.
    /// Paths and lowest common ancestors exist only within one tree (`root_of` tells). O(n).
    pub fn forest<N>(adj: N, roots: List<V>) = Rooted.build(adj, roots, a => 1)

    fn build<N, A>(adj: N, roots: List<V>, len: A -> Int):
        domain P = 0..V.K
        let root = roots[0]
        var par: [V]?V = none
        var dep: [V]Int = 0
        var far: [V]Int = 0
        var kid: [V]List<V> = []
        var seen: [V]Bool = false
        var order = List<V>()  // preorder of the depth-first search
        var stack = [(r, r, 0) for r in roots.rev]
        while not stack.is_empty:
            let (u, p, w) = stack[stack.len - 1]
            stack.pop()
            if seen[u]: continue
            seen[u] = true
            if u != p:
                par[u] = p
                dep[u] = dep[p] + 1
                far[u] = far[p] + w
                kid[p].push(u)
            order.push(u)
            let base = stack.len
            for a in adj.out(u):
                let c = a.to
                if not seen[c]: stack.push((c, u, len(a)))
            var (l, r) = (base, stack.len - 1)
            while l < r:
                stack.swap(l, r)
                l += 1
                r -= 1
        assert order.len == V.K
        var sz: [V]Int = 1
        for u in order.rev:
            if let p = par[u]: sz[p] += sz[u]
        var heavy: [V]?V = none
        for u in order:
            for c in kid[u]:
                if heavy[u] == none or sz[c] > sz[heavy[u]!]: heavy[u] = c
        var top = [V] v => v
        var tin = [V] v => P.first
        var start = 0
        for r in roots:
            tin[r] = P.nth(start)
            start += sz[r]
        for u in order:
            var next = tin[u].ord + 1
            if let h = heavy[u]:
                top[h] = top[u]
                tin[h] = P.nth(next)
                next += sz[h]
            for c in kid[u]:
                if c != heavy[u]:
                    tin[c] = P.nth(next)
                    next += sz[c]
        var at = [P] p => root
        for v in V: at[tin[v]] = v
        Rooted<V, P>(root, par, dep, far, sz, kid, heavy, top, tin, at)

    /// The position of `v` in this tree's value-owned preorder domain `t.Pre`. Inverse of `at`. O(1).
    pub fn pos(self, v: V) -> Pre = tin[v]

    /// The parent of `v`; `none` at the root.
    pub fn parent(self, v: V) -> ?V = par[v]

    /// Reorder a vertex-indexed array `[V]T` into preorder `[t.Pre]T` (`out[t.pos(v)] == a[v]`). O(n).
    pub fn reorder<T>(self, a: [V]T) -> [Pre]T = [Pre] p => a[at[p]]

    /// The children of `v`, in the order `adj` listed them.
    pub fn children(self, v: V) -> List<V> = kid[v]

    /// For each child `c` of `v`, in child order, the fold with `m` of `f` over `v`'s other
    /// children: the step of a rerooting DP, with no division or inverse needed. The fold keeps
    /// child order (earlier siblings, then later ones), so `m` need not commute. O(k) calls of
    /// `f` and of `m` for `k` children.
    ///
    ///     for (c, rest) in t.excl_fold(v, mul, c => down[c] + 1):
    ///         up[c] = (up[v] * rest + 1) % m
    pub fn excl_fold<T, M>(self, v: V, m: M, f: V -> T) -> yields (c: V, rest: T):
        let cs = kid[v]
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
        for c in kid[v]:
            if some(c) != heavy[v]: yield c

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
            for c in kid[v]: acc = k.op(acc, lift(out[c]!, c, v))
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
            let cs = kid[v]
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
            var key = [of[c] for c in kid[v]]
            sort(mut key)
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

    /// Every vertex, each before its descendants (heavy child first): `at[p] for p in Pre`.
    pub @get fn preorder(self) -> List<V> = [at[p] for p in Pre]

    /// Every vertex, each after its descendants: the order to fold children into parents.
    pub @get fn postorder(self) -> List<V>:
        // v leaves after the sz[v] vertices of its subtree and after every
        // vertex entered before it, except its dep[v] - dep[root] open ancestors.
        var out = [at[p] for p in Pre]
        for p in Pre:
            let v = at[p]
            out[p.ord + sz[v] - 1 - (dep[v] - dep[root])] = v
        out

    /// Iterative enter/exit Euler tour: yields `(v, true)` when entering `v` and
    /// `(v, false)` when leaving `v`'s subtree, without recursion. O(n).
    pub @get fn tour(self) -> yields (v: V, entering: Bool):
        var stack = [(at[p], 0) for p in Pre.rev if par[at[p]] == none]
        while not stack.is_empty:
            let (u, i) = stack[stack.len - 1]
            if i == 0: yield (v: u, entering: true)
            if i < kid[u].len:
                stack[stack.len - 1] = (u, i + 1)
                stack.push((kid[u][i], 0))
            else:
                stack.pop()
                yield (v: u, entering: false)

    /// The positions of `v`'s subtree, `v` first.
    pub fn sub(self, v: V) -> Range<Pre> = tin[v].l..tin[v].r(sz[v])

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
        if k > dep[v]: return none
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
        sort(mut all)
        for i in 0..keys.len - 1:
            let l = self.lca(all[i].1, all[i + 1].1)
            all.push((tin[l].ord, l))
        sort(mut all)
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
    assert a.preorder == b.preorder and a.par == b.par and a.sz == b.sz

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
        for v in t.postorder:
            for c in t.children(v): down[v] = down[v] * (down[c] + 1) % mod
        var up: [V]Int = 1
        for v in t.preorder:
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
        for v in r.postorder:
            for c in r.children(v): assert seen[c]
            seen[v] = true
        seen.fill(false)
        for v in r.preorder:
            assert r.parent(v) == none or seen[r.parent(v)!]
            seen[v] = true
        for v in V:
            let cap = rand(0, 4)
            let got = r.lift_while(v, u => r.depth(v) - r.depth(u) <= cap)
            assert got == r.ancestor(v, min(cap, r.depth(v)))
            assert r.lift_while(v, u => false) == none
        var keys: List<V> = []
        for v in V:
            if rand(0, 2) == 0: keys.push(v)
        let vt = r.virtual_tree(keys)
        var kept: [V]Bool = false
        for a in keys:
            for b in keys: kept[r.lca(a, b)] = true
        assert sorted(vt.nodes) == [v for v in V if kept[v]]
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
fn weighted_rooted_dfs_tree_on_cyclic_graph():
    domain V = 0..6
    // Cycle + chords + self-loop: DFS tree must have every non-tree edge connect ancestor/descendant
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
        assert r.is_anc(V(u), V(v)) or r.is_anc(V(v), V(u))
    // DFS explores first neighbor 1 -> 2 -> 4 -> 3, 5; so 0's only tree child is 1
    assert r.children(V(0)) == [V(1)] and r.dist(V(0), V(2)) == 10 and r.dist(V(0), V(5)) == 15

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
            assert sorted(cs) == sorted(r.children(v))
            let sub = [u for u in r.subtree(v)]
            assert sub[0] == v and sorted(sub) == [u for u in V if r.is_anc(v, u)]
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
    assert all(f.parent(r) == none for r in roots) and f.preorder.len == 10
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
        for v in r.postorder:
            var parts = [canon[c] for c in r.children(v)]
            sort(mut parts)
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
