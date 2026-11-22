// p11 with a hand-rooted tree and two passes (row 9).
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
var size: [V]Int = 1
var down: [V]Int = 0
for v in order.rev:
    if let p = par[v]:
        size[p] += size[v]
        down[p] += down[v] + size[v]
var all: [V]Int = 0
for v in order:
    all[v] = if let p = par[v]: all[p] - size[v] + (n - size[v]) else: down[v]
print([all[v] for v in V])
