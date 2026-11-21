@!multitest
let n: Int = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)

var cur_deg = [V] v => g[v].len
var deg_freq: [0..n]Int = 0
for v in V: deg_freq[cur_deg[v]] += 1

var edge_cnt = 1
var ans = [1]
var leaves = [v for v in V if cur_deg[v] == 1]
var alive: [V]Bool = true
var max_deg = n - 1
var colors = 2
while ans.len < n - 1:
    if colors % 2 == 0:
        while deg_freq[max_deg] == 0: max_deg -= 1
        edge_cnt += max_deg - 1
    else:
        edge_cnt += leaves.len - (max_deg - 1)
        var nleaves: List<V> = []
        for v in leaves:
            alive[v] = false
            deg_freq[cur_deg[v]] -= 1
            for nb in g[v] if alive[nb]:
                deg_freq[cur_deg[nb]] -= 1
                cur_deg[nb] -= 1
                deg_freq[cur_deg[nb]] += 1
                if cur_deg[nb] == 1: nleaves.push(nb)
        leaves = nleaves
    while ans.len < edge_cnt: ans.push(colors)
    colors += 1
print(ans)
