// Codeforces 2165 E: alternate taking the maximum remaining vertex degree and
// peeling the current leaf layer to compute the minimum edge count for each
// chromatic bound.
@!multitest
let n: Int = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)

var deg = [V] v => g.deg(v)
var deg_freq: [0..n]Int = 0
for v in V: deg_freq[deg[v]] += 1

var (edge_cnt, max_deg, colors) = (1, n - 1, 2)
var ans = [1]
var leaves = [v for v in V if deg[v] == 1]
var alive: [V]Bool = true
while ans.len < n - 1:
    if colors % 2 == 0:
        while deg_freq[max_deg] == 0: max_deg -= 1
        edge_cnt += max_deg - 1
    else:
        edge_cnt += leaves.len - (max_deg - 1)
        var next_leaves: List<V> = []
        for v in leaves:
            alive[v] = false
            deg_freq[deg[v]] -= 1
            for nb in g[v] if alive[nb]:
                deg_freq[deg[nb]] -= 1
                deg[nb] -= 1
                deg_freq[deg[nb]] += 1
                if deg[nb] == 1: next_leaves.push(nb)
        leaves = next_leaves
    while ans.len < edge_cnt: ans.push(colors)
    colors += 1
print(ans)
