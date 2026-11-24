// aac5p3, noi19p1: a domain field beside ranged ones keeps the ranged ones narrow, 12 B a read
// edge and 8 B an arc as in the matched C++.
let n: 1..=4_000_000 = read()
domain V = 1..=n
let hills: [0..n](u: V, v: V, d: 1..=1e9) = read()
var adj: [V]List<(to: V, d: 1..=1e9)> = []
for (u, v, d) in hills:
    adj[u].push((to: v, d:))
    adj[v].push((to: u, d:))
var s = 0
for u in V: for (to, d) in adj[u]: s += to.ord + d
print(s)
