//! Rooted trees: parents, subtree ranges, LCA, and path folds by heavy-light decomposition.
//!
//! `Rooted<V>` owns a second domain, `Pre`: the positions of an Euler order
//! in which every subtree and every heavy chain is one contiguous range.
//! `tin[v]` is a `Pre`, not a `V`, so indexing a per-position structure
//! with a vertex (the classic Euler tour bug) does not type-check.
use std.algebra.{Monoid, Sum, Max, flip}
use std.segtree.{SegTree}

/// A tree rooted at `root`. Built in O(n) without recursion; memory is
/// seven `[V]` arrays, one `[Pre]` array and the child lists.
pub struct Rooted<V>:
    /// Positions in heavy-first preorder.
    domain Pre = 0..V.K

    root: V
    /// The parent; `none` at the root.
    up: [V]?V
    depth: [V]Int
    /// The total edge length from the root; equal to `depth` when the edges have no lengths.
    far: [V]Int
    /// The number of vertices in the subtree.
    size: [V]Int
    kids: [V]List<V>
    /// The child with the largest subtree; `none` at a leaf.
    heavy: [V]?V
    /// The highest vertex of the heavy chain through `v`.
    top: [V]V
    tin: [V]Pre
    /// The inverse of `tin`.
    at: [Pre]V

    /// Root the tree given by `adj` (arcs in both directions, or parent to child) at `root`.
    /// Stops if some vertex is not reachable from the root.
    pub fn new<A>(adj: V -> Iter<A>, root: V) -> Rooted<V> = Rooted.weighted(adj, root, a => 1)

    /// The same for edges with lengths: `len(arc)` is the length of the edge an arc runs along,
    /// and `dist` adds lengths up instead of counting edges.
    pub fn weighted<A>(adj: V -> Iter<A>, root: V, len: A -> Int) -> Rooted<V>:
        var up: [V]?V = none
        var depth: [V]Int = 0
        var far: [V]Int = 0
        var kids: [V]List<V> = []
        var seen: [V]Bool = false
        var order = [root]
        seen[root] = true
        var i = 0
        while i < order.len:
            let u = order[i]
            i += 1
            for a in adj(u):
                let c = a.to
                if not seen[c]:
                    seen[c] = true
                    up[c] = u
                    depth[c] = depth[u] + 1
                    far[c] = far[u] + len(a)
                    kids[u].push(c)
                    order.push(c)
        assert order.len == V.K
        var size: [V]Int = 1
        for u in order.rev:
            if let p = up[u]: size[p] += size[u]
        var heavy: [V]?V = none
        for u in order:
            for c in kids[u]:
                if heavy[u] == none or size[c] > size[heavy[u]!]: heavy[u] = c
        var top = [V] v => v
        var tin = [V] v => Pre.first
        for u in order:
            var next = tin[u].ord + 1
            if let h = heavy[u]:
                top[h] = top[u]
                tin[h] = Pre.nth(next)
                next += size[h]
            for c in kids[u]:
                if c != heavy[u]:
                    tin[c] = Pre.nth(next)
                    next += size[c]
        var at = [Pre] p => root
        for v in V: at[tin[v]] = v
        Rooted(root, up, depth, far, size, kids, heavy, top, tin, at)

    /// The positions of `v`'s subtree, `v` first.
    pub fn sub(self, v: V) -> Range<Pre> = tin[v].l..tin[v].r(size[v])

    /// Is `a` an ancestor of `b`? A vertex is its own ancestor. O(1).
    pub fn is_anc(self, a: V, b: V) -> Bool = tin[b] in self.sub(a)

    /// The lowest common ancestor. O(log n).
    pub fn lca(self, a: V, b: V) -> V:
        var (x, y) = (a, b)
        while top[x] != top[y]:
            if depth[top[x]] < depth[top[y]]: swap(mut x, mut y)
            x = up[top[x]]!
        if depth[x] < depth[y]: x else: y

    /// The length of the path between `a` and `b`: its number of edges, or their total length in a
    /// tree built with `weighted`. O(log n).
    pub fn dist(self, a: V, b: V) -> Int = far[a] + far[b] - 2 * far[self.lca(a, b)]

    /// The ancestor `k` edges above `v`, or `none` if that is above the root. O(log n).
    pub fn ancestor(self, v: V, k: Int) -> ?V:
        if k > depth[v]: return none
        var (x, left) = (v, k)
        while depth[x] - depth[top[x]] < left:
            left -= depth[x] - depth[top[x]] + 1
            x = up[top[x]]!
        at[tin[x].offset(-left)!]

    /// The vertex `k` edges from `a` on the path to `b`, or `none` if the path is shorter. O(log n).
    pub fn step(self, a: V, b: V, k: Int) -> ?V:
        let c = self.lca(a, b)
        let (da, db) = (depth[a] - depth[c], depth[b] - depth[c])
        if k <= da: return self.ancestor(a, k)
        if k > da + db: return none
        self.ancestor(b, da + db - k)

    /// The path from `a` to `b` as O(log n) position ranges. `rising` ranges
    /// are walked from their high end to their low end. With `skip_top`, the
    /// highest vertex of the path is left out (for values stored per edge, at
    /// its lower end).
    pub fn chains(self, a: V, b: V, skip_top: Bool) -> yields (range: Range<Pre>, rising: Bool):
        var (x, y) = (a, b)
        var falling = List<Range<Pre>>()
        while top[x] != top[y]:
            if depth[top[x]] >= depth[top[y]]:
                yield (range: tin[top[x]]..=tin[x], rising: true)
                x = up[top[x]]!
            else:
                falling.push(tin[top[y]]..=tin[y])
                y = up[top[y]]!
        let skip = if skip_top: 1 else: 0
        if depth[x] >= depth[y]:
            yield (range: tin[y].l(1 - skip)..tin[x].r, rising: true)
        else:
            yield (range: tin[x].l(1 - skip)..tin[y].r, rising: false)
        for r in falling.rev: yield (range: r, rising: false)

/// Path and subtree folds over a tree with a value on every vertex.
/// `path` and `set` are O(log^2 n) and O(log n); `sub` is O(log n).
/// Paths are folded in order from `a` to `b`, so the monoid does not have
/// to be commutative; the price is a second segment tree for the upward
/// direction (2 * 2 * bit_ceil(n) values in total).
pub struct Hld<V, T>(m: Monoid<T>):
    r: Rooted<V>
    down: SegTree<Rooted<V>.Pre, T>
    rise: SegTree<Rooted<V>.Pre, T>

    pub fn new(r: Rooted<V>, m: Monoid<T>, val: [V]T) -> Hld<V, T>:
        let by_pos = [Rooted<V>.Pre] p => val[r.at[p]]
        Hld(m, r, SegTree.from(m, by_pos), SegTree.from(flip(m), by_pos))

    pub fn get(self, v: V) -> T = down.get(r.tin[v])

    pub fn set(mut self, v: V, x: T):
        down.set(r.tin[v], x)
        rise.set(r.tin[v], x)

    /// The fold of the values on the path, from `a` to `b`, both ends included.
    pub fn path(self, a: V, b: V) -> T = self.walk(a, b, false)

    /// `path` without the highest vertex: for a tree with a value per edge,
    /// stored at the edge's lower end.
    pub fn path_edges(self, a: V, b: V) -> T = self.walk(a, b, true)

    /// The fold over the subtree of `v`, in `Pre` order.
    pub fn sub(self, v: V) -> T = down.fold(r.sub(v))

    fn walk(self, a: V, b: V, skip_top: Bool) -> T:
        var acc = m.id
        for c in r.chains(a, b, skip_top):
            acc = m.op(acc, if c.rising: rise.fold(c.range) else: down.fold(c.range))
        acc

// ---- tests

/// A random tree on `V` as adjacency lists, with arcs in both directions.
fn random_tree<V>(any: V) -> [V]List<(to: V)>:
    var adj: [V]List<(to: V)> = []
    var label = [v for v in V]
    shuffle(mut label)
    for i in 1..V.K:
        let (c, p) = (label[i], label[rand(0, i - 1)])
        adj[c].push((to: p))
        adj[p].push((to: c))
    adj

/// The vertices from `a` to `b`, by climbing parents.
fn slow_path<V>(r: Rooted<V>, a: V, b: V) -> List<V>:
    var (x, y) = (a, b)
    var (left, right) = (List<V>(), List<V>())
    while x != y:
        if r.depth[x] >= r.depth[y]:
            left.push(x)
            x = r.up[x]!
        else:
            right.push(y)
            y = r.up[y]!
    left.push(x)
    left + right.rev

@test
fn subtrees_and_chains_are_contiguous():
    domain V = 1..=14
    for _ in 0..10:
        let adj = random_tree(V.first)
        let r = Rooted(v => adj[v], V.nth(rand(0, 13)))
        assert r.size[r.root] == 14 and r.up[r.root] == none
        for v in V:
            assert r.at[r.tin[v]] == v
            assert r.sub(v).len == r.size[v]
            for u in V:
                var x = u
                while x != v and r.up[x] != none: x = r.up[x]!
                assert r.is_anc(v, u) == (x == v)
            if let h = r.heavy[v]:
                assert r.tin[h].ord == r.tin[v].ord + 1 and r.top[h] == r.top[v]

@test
fn lca_dist_ancestor_and_step_match_climbing():
    domain V = 0..13
    for _ in 0..10:
        let adj = random_tree(V.first)
        let r = Rooted(v => adj[v], V.nth(rand(0, 12)))
        for a in V:
            for b in V:
                let path = slow_path(r, a, b)
                assert r.dist(a, b) == path.len - 1
                let top = r.lca(a, b)
                assert top in path and min(r.depth[v] for v in path) == r.depth[top]
                for k in 0..path.len: assert r.step(a, b, k) == path[k]
                assert r.step(a, b, path.len) == none
            assert r.ancestor(a, r.depth[a]) == r.root
            assert r.ancestor(a, r.depth[a] + 1) == none

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
    assert [r.far[v] for v in V] == [0, 3, 7, 13, 7] and r.depth[V(3)] == 2
    assert r.dist(V(2), V(3)) == 14 and r.dist(V(4), V(2)) == 14 and r.dist(V(3), V(3)) == 0
    let plain = Rooted(v => adj[v], V(0))
    assert plain.dist(V(2), V(3)) == 2 and plain.far[V(3)] == plain.depth[V(3)]

@test
fn path_folds_keep_the_order_from_a_to_b():
    domain V = 0..12
    let cat = Monoid<Str>("", (x, y) => x + y)
    for _ in 0..8:
        let adj = random_tree(V.first)
        let r = Rooted(v => adj[v], V.nth(rand(0, 11)))
        var name = [V] v => [Char(Int('a') + v.to_idx)]
        var h = Hld(r, cat, name)
        for round in 0..2:
            for a in V:
                for b in V:
                    let path = slow_path(r, a, b)
                    var want = ""
                    for v in path: want = want + name[v]
                    assert h.path(a, b) == want
                    var edges = ""
                    for v in path:
                        if v != r.lca(a, b): edges = edges + name[v]
                    assert h.path_edges(a, b) == edges
            let v = V.nth(rand(0, 11))
            name[v] = "Z"
            h.set(v, "Z")
            assert h.get(v) == "Z"

@test
fn subtree_sums_and_path_maxima():
    domain V = 1..=10
    let adj = random_tree(V.first)
    let r = Rooted(v => adj[v], V(1))
    let val = [V] v => rand(-20, 20)
    let sums = Hld(r, Sum<Int>, val)
    let tops = Hld(r, Max<Int>, val)
    for v in V:
        assert sums.sub(v) == sum(val[u] for u in V if r.is_anc(v, u))
        for u in V:
            assert tops.path(u, v) == max(val[x] for x in slow_path(r, u, v))
