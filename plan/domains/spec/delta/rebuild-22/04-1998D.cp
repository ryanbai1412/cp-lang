// cf1998D Determine Winning Islands in Race: Elsie's fastest arrival d[u] by BFS. An
// alternative bridge u -> v lets her overtake Bessie from s when u < s and she lands on v
// (after d[u] + 1 moves) ahead of Bessie at s + d[u] + 1: s in u+1 ..= v - d[u] - 2.
// Islands are 1-based; the answer string is indexed from 0 by s - 1.
@!multitest
let (n, m): (2..=2e5, 0..=2e5) = read()
domain V = 1..=n
let alts = read<[m](u: V, v: V)>()

var adj: [V]List<V> = []
for v in V: if let w = v.next: adj[v].push(w)
for e in alts: adj[e.u].push(e.v)
let d = bfs(adj, from: [V.first!])

var diff: [1..=n]Int = 0
for e in alts:
    let lo = e.u + 1
    let hi = e.v - d[e.u]! - 2
    if lo <= hi:
        diff[lo] += 1
        diff[hi + 1] -= 1
var beaten = 0
var out = List<Char>()
for s in 1..n:
    beaten += diff[s]
    out.push(if beaten > 0: '0' else: '1')
print(out, sep: "")
