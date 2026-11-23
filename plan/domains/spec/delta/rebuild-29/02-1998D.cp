// CF 1998D Determine Winning Islands in Race.
// dist[v]: Elsie's fewest moves to v. Islands in index order are a topological order of the DAG.
// Bessie (moving first from s) loses iff some alternative bridge u -> v with u < s < v
// gets Elsie to v strictly before her: dist[u] + 1 < v - s.

@!multitest
let (n, m): (2..=2e5, 0..=2e5) = read()
domain V = 1..=n
let alt = read<[m](u: V, v: V)>()
let g = Graph.directed(alt)
var dist: [V]Int = [V] v => Int(v) - 1
for u in V:
    if let w = u.next: dist[w] min= dist[u] + 1
    for a in g.out(u): dist[a.to] min= dist[u] + 1
// lose[s] for s in lo..=hi via a difference array over island numbers
var diff: [n + 2]Int = 0
for e in alt:
    let lo = Int(e.u) + 1
    let hi = Int(e.v) - dist[e.u] - 2
    if lo <= hi:
        diff[lo] += 1
        diff[hi + 1] -= 1
var out = ""
var cover = 0
for s in 1..n:
    cover += diff[s]
    out.push(if cover > 0: '0' else: '1')
print(out)
