// noi07p1 Social Network: betweenness of every vertex: over pairs s != t avoiding v, the share
// of shortest s-t paths through v. Floyd-Warshall that also counts shortest paths.
let (n, m) = read<(1..=100, 0..=4_500)>()
domain V = 1..=n
var dist: [V, V]?Int = none
var ways: [V, V]Int = 0
for v in V:
    dist[v, v] = 0
    ways[v, v] = 1
for e in read<[m](u: V, v: V, w: 1..=1_000)>():
    for (x, y) in [(e.u, e.v), (e.v, e.u)]:
        dist[x, y] = e.w
        ways[x, y] = 1
for k in V:
    for i in V:
        for j in V:
            if let a = dist[i, k], let b = dist[k, j], i != k and j != k:
                let via = a + b
                if dist[i, j] == none or via < dist[i, j]!:
                    dist[i, j] = via
                    ways[i, j] = ways[i, k] * ways[k, j]
                elif via == dist[i, j]!:
                    ways[i, j] += ways[i, k] * ways[k, j]
for v in V:
    var share = 0.0
    for s in V:
        for u in V:
            if s != v and u != v and s != u and dist[s, v]! + dist[v, u]! == dist[s, u]!:
                share += F64(ways[s, v] * ways[v, u]) / F64(ways[s, u])
    print(share, precision: 3)
