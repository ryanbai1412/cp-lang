// CF1941G Rudolf and Subway. 0-1 BFS over stations plus one node per line
// colour: boarding a line costs 1, leaving it costs 0.
use std.graph.{bfs01}

@!multitest
let n: 2..=2e5 = read()
let m: 1..=2e5 = read()
domain V = 1..=n
let es = read<[m](u: V, v: V, c: 1..=2e5)>()
let b, e: V = read()
let (domain C, comp) = compress([r.c for r in es])
domain N = 0..n + C.count
let st = (v: V) => N(v.ord)
let line = (c: C) => N(n + c.ord)
var adj: [N]List<(to: N, w: Int)> = []
for r in es:
    let l = line(comp.at(r.c))
    for x in [r.u, r.v]:
        adj[st(x)].push((to: l, w: 1))
        adj[l].push((to: st(x), w: 0))
let p = bfs01(adj, from: [st(b)])
print(p[st(e)]!)
