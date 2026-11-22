// p10 with a hand-rooted tree (row 9).
let n: 1..=2e5 = read()
domain V = 1..=n
let edges = read<[n - 1](u: V, v: V)>()
var adj: [V]List<V> = []
for e in edges:
    adj[e.u].push(e.v)
    adj[e.v].push(e.u)
var par: [V]?V = None
var order: List<V> = [V.first!]
var i = 0
while i < order.len:
    let u = order[i]
    i += 1
    for v in adj[u] if Some(v) != par[u]:
        par[v] = u
        order.push(v)
var free: [V]Int = 0
var used: [V]Int = 0
for v in order.rev:
    var gain = -inf
    for c in adj[v] if Some(c) != par[v]:
        let best = max(free[c], used[c])
        free[v] += best
        gain max= free[c] + 1 - best
    used[v] = if gain == -inf: 0 else: free[v] + gain
print(max(free[V.first!], used[V.first!]))
