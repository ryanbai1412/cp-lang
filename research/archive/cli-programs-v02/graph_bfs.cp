// Shortest paths by breadth-first search on an undirected graph.
let n: 1..=1000 = read()
let m: 0..=5000 = read()
domain V = 1..=n
var adj: [V]List<V> = []
for e in 0..m:
    let u: V = read()
    let v: V = read()
    adj[u].push(v)
    adj[v].push(u)
var dist: [V]?Int = none
var queue: List<V> = [V.first]
dist[V.first] = 0
var head = 0
while head < queue.len:
    let u = queue[head]
    head += 1
    for v in adj[u]:
        if dist[v].is_none:
            dist[v] = (dist[u] ?? 0) + 1
            queue.push(v)
for v in V:
    if let d = dist[v]:
        print(v, d)
    else:
        print(v, "unreachable")
print(queue)
