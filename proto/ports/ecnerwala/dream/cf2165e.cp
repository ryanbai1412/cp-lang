// Codeforces 2165 E: Rainbow Tree.
// For each edge count `k in 1..=n - 1`, find the minimum chromatic bound
// `colors` over all `k`-edge subforests obtained by peeling leaf layers from
// the tree:
//   - A `k`-edge subforest admits a `c`-coloring iff peeling `floor((c - 1) / 2)`
//     leaf layers leaves a forest whose remaining edges can be colored in the
//     remaining 1 (odd `c`) or 2 (even `c`) colors.
//   - Why a maximum-degree vertex is optimal on the even step (`colors = 2r`):
//     after `r - 1` peels, adding 1 color beyond `2r - 1` allows one vertex `v`
//     to absorb a star of `deg(v) - 1` extra incident edges (each using the new
//     color without creating a monochromatic path), so picking `v` with maximum
//     remaining degree `degs.last!` maximizes the edge count at `colors = 2r`.
//   - On the odd step (`colors = 2r + 1`), peeling the entire current leaf
//     layer covers all remaining `leaves.len - (max_deg - 1)` edges of the round.

use std.bag.SortedBag

@!multitest
let n: 2..=300_000 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](V, V)>())

var deg = [V] v => g[v].len
var degs = SortedBag.of(deg)
var leaves = [v for v in V if deg[v] == 1]
var alive: [V]Bool = true

fn peel_layer():
    var next_leaves: List<V> = []
    for v in leaves:
        alive[v] = false
        degs.remove(deg[v])
        for nb in g[v] if alive[nb]:
            degs.remove(deg[nb])
            deg[nb] -= 1
            degs.add(deg[nb])
            if deg[nb] == 1: next_leaves.push(nb)
    leaves = next_leaves

var edge_cnt = 1
var colors = 2
var ans = [1]

while ans.len < n - 1:
    let max_deg = degs.last!
    if colors % 2 == 0:
        edge_cnt += max_deg - 1
    else:
        edge_cnt += leaves.len - (max_deg - 1)
        peel_layer()
    while ans.len < edge_cnt: ans.push(colors)
    colors += 1

print(ans)
