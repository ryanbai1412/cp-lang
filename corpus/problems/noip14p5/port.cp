// Source: NOIP 2014 Senior Day 2 P2 (Finding the Way), Luogu P2296 (not on DMOJ)
// Problem: a directed graph, n <= 1e4, m <= 2e5 unit edges (loops and multi-edges allowed), and
//   s != t. Print the length of the shortest s-t path through vertices all of whose
//   out-neighbours can reach t, or -1 if there is none.
// Solution: BFS from t over reversed edges marks the vertices that reach t; a vertex is safe when
//   all its out-neighbours are marked. If s is safe, BFS from s over the edges between safe
//   vertices gives the answer. O(n + m).

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
