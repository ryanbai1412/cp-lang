// NOIP 2014 Finding the Way: shortest path from s to t that only visits
// vertices all of whose out-neighbours can reach t.

let n: 1..=10000 = read()
let m: 0..=200000 = read()
domain V = 1..=n
let edges: [m](u: V, v: V) = read()
let s: V = read()
let t: V = read()

var out: [V]List<V> = []
var back: [V]List<V> = []
for (u, v) in edges:
    out[u].push(v)
    back[v].push(u)

let reaches = bfs(v => back[v], from: [t])
let safe = [V] u => all(reaches[v] != None for v in out[u])
let walk: [V]List<V> = [V] u => [v for v in out[u] if safe[u] and safe[v]]
print(if safe[s]: bfs(v => walk[v], from: [s])[t] ?? -1 else: -1)
