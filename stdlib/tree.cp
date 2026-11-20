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
use std.algebra.{Monoid}
use std.segtree.{SegTree}
use std.lazy.{LazySegTree, AddMax}

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
    /// `Rooted(root: r, adj: u => lists[u])` for anything else. `adj` may list every neighbour
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
    pub fn weighted<N, A>(adj: N, root: V, len: A -> Int):
        domain P = 0..V.K
        var par: [V]?V = none
        var dep: [V]Int = 0
        var far: [V]Int = 0
        var kid: [V]List<V> = []
        var seen: [V]Bool = false
        var order = List<V>()                    // preorder of the depth-first search
        var stack = [(root, root, 0)]
        while not stack.is_empty:
            let (u, p, w) = stack[stack.len - 1]
            stack.pop()
            if seen[u]: continue
            seen[u] = true
            if u != root:
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
                let tmp = stack[l]
                stack[l] = stack[r]
                stack[r] = tmp
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

    /// The number of edges between `v` and the root.
    pub fn depth(self, v: V) -> Int = dep[v]

    /// The number of vertices in the subtree of `v`.
    pub fn size(self, v: V) -> Int = sz[v]

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
        var stack = [(root, 0)]
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
    pub fn chains(self, a: V, b: V, edges: Bool = false) -> yields (range: Range<Pre>, rising: Bool):
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
            while not self.is_anc(st[st.len - 1], u): st.pop()
            edges.push((u: st[st.len - 1], v: u))
            st.push(u)
        (nodes: nodes, edges: edges)

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
            assert not any(kept[w] and w != e.u and w != e.v and r.is_anc(e.u, w) and r.is_anc(w, e.v) for w in V)

@test
fn a_tree_from_a_stored_graph():
    domain V = 1..=6
    domain E = 1..=5
    let edges = [E] e => [(u: V(1), v: V(2)), (u: V(1), v: V(3)), (u: V(2), v: V(4)), (u: V(2), v: V(5)), (u: V(5), v: V(6))][e.to_idx]
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
    let raw = [(0, 1, 3), (0, 2, 5), (1, 2, 7), (1, 3, 2), (2, 4, 4), (3, 4, 6), (4, 5, 1), (0, 5, 9), (2, 2, 1)]
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
