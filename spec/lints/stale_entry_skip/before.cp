let (n, m): (1..=200000, 1..=200000) = read()
var adj: [1..=n]List<(Int, Int)> = []
for _ in 0..m:
    let (u, v, c): (1..=n, 1..=n, 1..=1000000000) = read()
    adj[u].push((v, c))
var dist: [1..=n]?Int = None
var pq = MinHeap<(Int, Int)>()
dist[1] = 0
pq.push((0, 1))
while let (d, u) = pq.pop():
    if d != dist[u]!:
        continue
    for (v, c) in adj[u]:
        if dist[v] == None or d + c < dist[v]!:
            dist[v] = d + c
            pq.push((d + c, v))
print(dist[n] ?? -1)
