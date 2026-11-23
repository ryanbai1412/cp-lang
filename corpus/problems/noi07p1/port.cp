// NOI 2007 P1 Social Network: n people, m weighted friendships. For each
// person v, print the sum over ordered pairs (s, t) of the share of shortest
// s-t paths that pass through v.
// Solution: Floyd-Warshall that also counts the shortest paths between every
// pair; v lies on a shortest s-t path iff dist(s, v) + dist(v, t) == dist(s, t),
// and then carries ways(s, v) * ways(v, t) of the ways(s, t) paths.
// original: corpus/submissions/3756474.cpp
let (n, m): (1..=100, 1..=4500) = read()
domain V = 1..=n
var dist: [V, V]?Int = None
var ways: [V, V]Int = 0
for v in V:
    dist[v, v] = Some(0)
    ways[v, v] = 1
for _ in 0..m:
    let (u, v, w): (V, V, 1..=1000) = read()
    dist[u, v] = Some(w)
    dist[v, u] = Some(w)
    ways[u, v] = 1
    ways[v, u] = 1

for k in V:
    for i in V:
        let dik = dist[i, k] else: continue
        for j in V:
            let dkj = dist[k, j] else: continue
            let (cand, via) = (dik + dkj, ways[i, k] * ways[k, j])
            let old = dist[i, j]
            if old == None or cand < old!:
                dist[i, j] = Some(cand)
                ways[i, j] = via
            elif cand == old! and k != i and k != j:
                ways[i, j] += via

for v in V:
    var share = 0.0
    for s in V:
        for t in V:
            if s == v or t == v or s == t: continue
            if let a = dist[s, v], let b = dist[v, t], a + b == dist[s, t]!:
                share += F64(ways[s, v] * ways[v, t]) / F64(ways[s, t])
    print(share, precision: 3)
