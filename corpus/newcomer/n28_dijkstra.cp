let (n, m): (1..=200000, 1..=200000) = read()
var adj: List<List<(Int, Int)>> = [[] for _ in 0..=n]  // expect: missing manual-adjacency-list
for _ in 0..m:
    let (u, v, c): (1..=n, 1..=n, 1..=1000000000) = read()
    adj[u].push((v, c))
var dist: List<Int> = [1000000000000000000 for _ in 0..=n]  // expect: missing sentinel-instead-of-optional
var pq = MinHeap<(Int, Int)>()
dist[1] = 0
pq.push((0, 1))
while pq.len > 0:
    let (d, u) = pq.pop()!
    if d > dist[u]:  // expect: missing stale-heap-entry-check
        continue
    for (v, c) in adj[u]:
        if dist[u] + c < dist[v]:
            dist[v] = dist[u] + c
            pq.push((dist[v], v))
print(dist[n])
