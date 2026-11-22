//! Centroid decomposition (`Centroids<V>`) and all-roots tree DP (`reroot`).

/// Centroid decomposition of a tree over `V`.
/// Every vertex belongs to at most `ceil(log2 n)` centroid subtrees, so `ancestors(v)` has
/// length O(log n) and gives `(centroid, dist(v, centroid))` for every centroid ancestor of `v`.
/// Building takes O(n log n).
use std.tree.{Rooted}
pub struct Centroids<V>:
    root: V
    par: [V]?V
    dep: [V]Int
    anc: [V]List<(V, Int)>

    pub fn new(adj: V -> List<V>) -> Centroids<V>:
        var dead: [V]Bool = false
        var sz: [V]Int = 0
        var cpar: [V]?V = none
        var cdep: [V]Int = 0
        var anc: [V]List<(V, Int)> = []
        var root = V.first
        var first = true

        // Iterative decomposition stack: (entry vertex, parent centroid, depth)
        var todo: List<(V, ?V, Int)> = [(V.first, none, 0)]
        while let item = todo.pop():
            let (entry, pcent, d) = item
            // 1. BFS component & compute subtree sizes bottom-up
            var order = [entry]
            var bpar: [V]?V = none
            var head = 0
            while head < order.len:
                let u = order[head]
                head += 1
                for w in adj(u):
                    if not dead[w] and w != bpar[u]:
                        bpar[w] = u
                        order.push(w)
            for u in order: sz[u] = 1
            for i in (1..order.len).rev:
                let u = order[i]
                sz[bpar[u]!] += sz[u]
            let total = sz[entry]
            // 2. Walk down to a centroid (every branch has size <= total / 2)
            var c = entry
            var moved = true
            while moved:
                moved = false
                for w in adj(c):
                    if not dead[w] and w != bpar[c] and sz[w] * 2 > total:
                        c = w
                        moved = true
                        break
            if first:
                root = c
                first = false
            cpar[c] = pcent
            cdep[c] = d
            dead[c] = true
            // 3. BFS from centroid `c` to record `(c, dist)` for every vertex in its component
            var q = [(c, 0)]
            var qpar: [V]?V = none
            var qh = 0
            while qh < q.len:
                let (u, du) = q[qh]
                qh += 1
                anc[u].push((c, du))
                for w in adj(u):
                    if not dead[w] and w != qpar[u]:
                        qpar[w] = u
                        q.push((w, du + 1))
            for w in adj(c):
                if not dead[w]: todo.push((w, c, d + 1))

        Centroids(root, cpar, cdep, anc)

    /// Parent of `v` in the centroid tree; `none` at the root centroid.
    pub fn parent(self, v: V) -> ?V = par[v]

    /// Depth of `v` in the centroid tree (always `<= log2(V.count)`).
    pub fn depth(self, v: V) -> Int = dep[v]

    /// All `(centroid, dist(v, centroid))` pairs from root centroid down to `(v, 0)`. O(log n) items.
    pub fn ancestors(self, v: V) -> List<(V, Int)> = anc[v]

/// All-roots tree DP without requiring an inverse operation (uses prefix + suffix folds).
///
/// - `id`: identity for combining sibling contributions.
/// - `merge(a, b)`: associative combination of two sibling contributions.
/// - `lift(val, child, parent)`: transform `child`'s finished subtree value across edge `(child, parent)`.
/// - `finish(sum, v)`: turn the combined contributions of all neighbors of `v` into `v`'s answer.
///
/// O(n) calls to `merge`, `lift`, and `finish`.
pub fn reroot<V, T>(
    adj: V -> List<V>,
    id: T,
    merge: (T, T) -> T,
    lift: (T, V, V) -> T,
    finish: (T, V) -> T,
) -> [V]T:
    let r = Rooted(root: V.first!, next: adj)
    var down = [V] _ => id
    var sub_val = [V] _ => id
    // 1. Bottom-up pass
    for u in r.at.rev:
        var acc = id
        for c in r.children(u): acc = merge(acc, lift(down[c], c, u))
        sub_val[u] = acc
        down[u] = finish(acc, u)
    // 2. Top-down pass with prefix/suffix sibling folds (works for non-invertible monoids like max)
    var up = [V] _ => id
    var ans = [V] _ => id
    for u in r.at:
        let kids = r.children(u).collect()
        let k = kids.len
        let contrib = [lift(down[c], c, u) for c in kids]
        var pref = List.filled(k + 1, id)
        for i in 0..k: pref[i + 1] = merge(pref[i], contrib[i])
        var suff = List.filled(k + 1, id)
        for i in (0..k).rev: suff[i] = merge(contrib[i], suff[i + 1])
        let total = if r.parent(u) == none: pref[k] else: merge(pref[k], up[u])
        ans[u] = finish(total, u)
        for i in 0..k:
            let c = kids[i]
            var without = merge(pref[i], suff[i + 1])
            if r.parent(u) != none: without = merge(without, up[u])
            up[c] = lift(finish(without, u), u, c)
    ans

// ---- tests

fn rand_tree<V>() -> [V]List<V>:
    var adj: [V]List<V> = []
    for i in 1..V.count:
        let p = V.nth(rand(0, i - 1))
        let u = V.nth(i)
        adj[p].push(u)
        adj[u].push(p)
    adj

@test
fn centroid_tree_depth_and_nearest_marked_vertex():
    domain V = 0..16
    for _ in 0..15:
        let adj = rand_tree<V>()
        let r = Rooted(root: V(0), next: u => adj[u])
        let cd = Centroids<V>(u => adj[u])
        // Centroid tree depth is O(log n) and distances match true tree distances
        for v in V:
            assert cd.depth(v) <= 5
            for (c, d) in cd.ancestors(v): assert d == r.dist(v, c)
        // Classic dynamic "paint vertex / query nearest painted vertex" against brute force
        var best: [V]?Int = none
        var painted: List<V> = []
        for _ in 0..12:
            let v = V.nth(rand(0, 15))
            painted.push(v)
            for (c, d) in cd.ancestors(v):
                if best[c] == none or d < best[c]!: best[c] = d
            for q in V:
                var got: ?Int = none
                for (c, d) in cd.ancestors(q):
                    if best[c] != none:
                        let cand = best[c]! + d
                        if got == none or cand < got!: got = cand
                let want = painted.map(p => r.dist(q, p)).min()
                assert got == want

@test
fn reroot_computes_eccentricity_and_distance_sums():
    domain V = 0..14
    for _ in 0..15:
        let adj = rand_tree<V>()
        let r = Rooted(root: V(0), next: u => adj[u])
        // Non-invertible monoid: maximum distance from every vertex (eccentricity)
        let ecc = reroot<V, Int>(
            u => adj[u],
            0,
            (a, b) => max(a, b),
            (val, _c, _p) => val + 1,
            (s, _v) => s,
        )
        for v in V:
            assert ecc[v] == V.map(u => r.dist(v, u)).max()!
        // Sum of distances from every vertex using (sum, size) pairs
        let sums = reroot<V, (Int, Int)>(
            u => adj[u],
            (0, 0),
            (a, b) => (a.0 + b.0, a.1 + b.1),
            (val, _c, _p) => (val.0 + val.1, val.1),
            (s, _v) => (s.0, s.1 + 1),
        )
        for v in V:
            assert sums[v].0 == sum(r.dist(v, u) for u in V)
            assert sums[v].1 == V.count
