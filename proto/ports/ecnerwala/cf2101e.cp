// Codeforces 2101 E (Longest Doubling Walk on a Tree):
// For each '1'-vertex v, find the longest sequence v = v_1, v_2, ..., v_k of
// '1'-vertices whose consecutive tree distances double:
//   dist(v_{i+1}, v_{i+2}) >= 2 * dist(v_i, v_{i+1}).
//
// Work backwards by round r = 1, 2, ... (at most O(log n) rounds since the
// first step doubles each time):
//   - min_dist[v] is the largest first-step budget D for which a valid walk of
//     length r - 1 exists starting from v (initialized to n for '1', 0 for '0').
//   - A '1'-vertex u extends to length r via w if dist(u, w) <= min_dist[w],
//     giving next_dist[u] = max dist(u, w) and new budget next_dist[u] / 2.
//
// On a prebuilt centroid decomposition `cd = Centroids.new(adj)`, every path
// (u, w) splits at its highest centroid c into two different branches with
// dist(u, w) = depth(u) + depth(w). Rewriting the condition as
// depth(u) <= min_dist[w] - depth(w), we bucket each w in c's piece at
// `min_dist[w] - depth(w)`, take suffix maxima of `Best` (deepest vertex and
// deepest vertex from another branch), and query at `depth(u)`.

use std.tree.Centroids

type Far<V> = (depth: Int, branch: V)

// Deepest vertex in a centroid piece and deepest vertex in a different branch.
struct Best<V>:
    far: ?Far<V>
    other: ?Int

fn deeper(a: ?Int, b: ?Int) -> ?Int:
    if let x = a, let y = b: return max(x, y)
    a ?? b

fn merge<V>(a: Best<V>, o: Best<V>) -> Best<V>:
    let x = a.far ?? return o
    let y = o.far ?? return a
    if x.branch == y.branch: Best((max(x.depth, y.depth), x.branch), deeper(a.other, o.other))
    elif y.depth > x.depth: Best(y, deeper(x.depth, o.other))
    else: Best(x, deeper(a.other, y.depth))

@!multitest
let n: 1..=70_000 = read()
domain V = 1..=n
let s: [V]Char = read()
let edges: [0..n - 1](u: V, v: V) = read()
let adj = Graph.undirected(edges)
let cd = Centroids<V>.new(adj)

var ans: [V]Int = [V] v => if s[v] == '1': 1 else: -1
var min_dist: [V]Int = [V] v => if s[v] == '1': n else: 0
for round in 2..:
    var next_dist: [V]Int = 0
    for c in V:
        let max_d = max(d for (_, d, _) in cd.piece(c)) ?? 0
        var bests = List.filled(max_d + 1, Best<V>(none, none))
        for (u, d, br) in cd.piece(c) if min_dist[u] > 0 and min_dist[u] >= d:
            let idx = min(max_d, min_dist[u] - d)
            bests[idx] = merge(bests[idx], Best((d, br), none))
        for i in (0..max_d).rev:
            bests[i] = merge(bests[i], bests[i + 1])
        for (u, d, br) in cd.piece(c) if s[u] == '1':
            if let top = bests[d].far:
                let far_opt = if top.branch == br: bests[d].other else: some(top.depth)
                if let far = far_opt: next_dist[u] max= d + far

    let reached = [v for v in V if next_dist[v] > 0]
    if reached.is_empty: break
    for v in reached: ans[v] = round
    min_dist = [V] v => next_dist[v] / 2
print(ans)
