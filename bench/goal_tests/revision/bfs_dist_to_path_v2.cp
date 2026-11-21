let (n, m): (Int, Int) = read()
domain V = 0..n
var adj: [V]List<V> = []
for _ in 0..m:
    let (u, v): (V, V) = read()
    adj[u].push(v)
    adj[v].push(u)
var dist: [V]?Int = none
var par: [V]?V = none
dist[V(0)] = 0
var q: List<V> = [V(0)]
var head = 0
while head < q.len:
    let u = q[head]
    head += 1
    let d = dist[u]!
    for v in adj[u]:
        if dist[v] == none:
            dist[v] = d + 1
            par[v] = u
            q.push(v)
var path: List<Int> = []
var cur: ?V = V(n - 1)
while let u = cur:
    path.push(u.ord)
    cur = par[u]
path.reverse()
print(path)
