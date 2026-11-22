domain V = 1..=3
var adj: [V]List<V> = []
let (u, v) = (V(1), V(2))
for (a, b) in [(u, v), (v, u)]:
    adj[a].push(b)
print(adj[u].len)
