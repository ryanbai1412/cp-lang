// NOI '07 P1 - Social Network (DMOJ: noi07p1, 15pt, TL 0.6s, ML 256MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Undirected weighted simple graph (n <= 100, m <= 4500). For every vertex v, compute
// betweenness centrality:
//   I(v) = sum_{s != v, t != v, s != t} C_{s,t}(v) / C_{s,t}
// where C_{s,t} is the exact number of shortest s-t paths (64-bit Int) and C_{s,t}(v)
// is the number passing through v. Print each I(v) to 3 decimal places.
let (n, m): (1..=100, 0..=4500) = read()
domain V = 1..=n
domain E = 0..m
// WISH MG11: simple undirected connected weighted graph constraint for cpl stress
let edges: [E](u: V, v: V, w: 1..=1000) = read()

// WISH MG12: MinPlus + path-count semiring `(dist: ?Int, ways: Int)` for Floyd-Warshall
var dist: [V, V]?Int = none
var ways: [V, V]Int = 0
for v in V:
    dist[v, v] = 0
    ways[v, v] = 1

for e in E:
    let (u, v, w) = (edges[e].u, edges[e].v, edges[e].w)
    dist[u, v] = w
    dist[v, u] = w
    ways[u, v] = 1
    ways[v, u] = 1

for k in V:
    for i in V:
        if let dik = dist[i, k]:
            for j in V:
                if let dkj = dist[k, j]:
                    let cand = dik + dkj
                    if let dij = dist[i, j]:
                        if cand < dij:
                            dist[i, j] = cand
                            ways[i, j] = ways[i, k] * ways[k, j]
                        elif cand == dij and k != i and k != j:
                            ways[i, j] += ways[i, k] * ways[k, j]
                    else:
                        dist[i, j] = cand
                        ways[i, j] = ways[i, k] * ways[k, j]

for v in V:
    var total = 0.0
    for s in V:
        if s == v: continue
        for t in V:
            if t == v or s == t: continue
            if let dsv = dist[s, v]:
                if let dvt = dist[v, t]:
                    if let dst = dist[s, t]:
                        if dsv + dvt == dst:
                            // WISH MG9: exact Int ratio -> F64 + `{total:.3f}` formatting
                            total += F64(ways[s, v] * ways[v, t]) / F64(ways[s, t])
    print("{total:.3f}")
