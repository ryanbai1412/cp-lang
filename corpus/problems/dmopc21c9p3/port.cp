// Runnable port (proto/cp.py lib-run) of dmopc21c9p3.cp
// original: corpus/submissions/4564027.cpp

let (n, m): (Int, Int) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()

var adj: [V]List<V> = [V] _ => []
for e in E:
    adj[edges[e].u].push(edges[e].v)
    adj[edges[e].v].push(edges[e].u)

// All-pairs distances by one BFS per source (the graph is connected).
var dist: [V][V]Int = [V] _ => [V] _ => -1
for s in V:
    var queue: List<V> = [s]
    var head = 0
    dist[s][s] = 0
    while head < queue.len:
        let u = queue[head]
        head += 1
        for v in adj[u]:
            if dist[s][v] == -1:
                dist[s][v] = dist[s][u] + 1
                queue.push(v)

let q: Int = read()
for _ in 0..q:
    let (a, b, c): (V, V, Int) = read()
    let (d, e, f): (V, V, Int) = read()
    var ok = dist[a][b] <= c or dist[d][e] <= f
    if not ok:
        // Swap targets instead: both walkers get one step less.
        let swap = min(dist[a][d] + dist[b][e], dist[a][e] + dist[b][d])
        ok = swap <= c + f - 2
    print(if ok: "YES" else: "NO")
