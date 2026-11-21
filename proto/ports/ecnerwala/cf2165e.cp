// Codeforces 2165 E: Rainbow Tree.
// For each edge count `k in 1..=n - 1`, find the minimum chromatic bound
// `colors` over all `k`-edge subforests obtained by repeatedly peeling leaf
// layers from the tree:
//   - Starting at `edge_cnt = 1` (1 color), each peeling round adds 2 colors
//     and removes all `leaves.len` current leaves, advancing `edge_cnt` by
//     `leaves.len` total across the even and odd steps.
//   - On the even step (`colors % 2 == 0`), saturating a maximum-degree vertex
//     of the remaining tree covers `max_deg - 1` additional edges before
//     needing another color.
//   - On the odd step (`colors % 2 == 1`), peeling the full leaf layer covers
//     the remaining `leaves.len - (max_deg - 1)` edges of the round.

@!multitest
let n: 2..=300_000 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](V, V)>())

var deg = [V] v => g.deg(v)
var deg_freq: [0..n]Int = 0
for v in V: deg_freq[deg[v]] += 1
var top_deg = n - 1

fn max_remaining_deg() -> Int:
    while deg_freq[top_deg] == 0: top_deg -= 1
    top_deg

fn dec_deg(v: V):
    deg_freq[deg[v]] -= 1
    deg[v] -= 1
    deg_freq[deg[v]] += 1

var edge_cnt = 1
var colors = 2
var ans = [1]
var leaves = [v for v in V if deg[v] == 1]
var alive: [V]Bool = true

while ans.len < n - 1:
    if colors % 2 == 0:
        edge_cnt += max_remaining_deg() - 1
    else:
        edge_cnt += leaves.len - (max_remaining_deg() - 1)
        var next_leaves: List<V> = []
        for v in leaves:
            alive[v] = false
            deg_freq[deg[v]] -= 1
            for nb in g[v] if alive[nb]:
                dec_deg(nb)
                if deg[nb] == 1: next_leaves.push(nb)
        leaves = next_leaves
    while ans.len < edge_cnt: ans.push(colors)
    colors += 1

print(ans)
