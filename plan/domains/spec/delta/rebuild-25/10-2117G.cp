// Codeforces 2117G Omg Graph: https://codeforces.com/problemset/problem/2117/G
// Fix the edge (u, v, w) that is the minimum of the path. Walks may repeat, so the best walk
// through it reaches the nearer end from 1 and leaves from the nearer end to n by widest paths:
// cost w + max(w, min(from1[u], from1[v]), min(fromn[u], fromn[v])).
@!multitest
let (n, m): (2..=2e5, 1..=2e5) = read()
domain V = 1..=n
let es = read<[m](u: V, v: V, w: 1..=1e9)>()
let g = Graph.undirected(es)

// smallest possible largest edge on a walk from s (dijkstra with max in place of +)
fn widest(s: V) -> [V]Int:
    var best: [V]?Int = None
    var h = MinHeap<(Int, V)>()
    best[s] = 0
    h.push((0, s))
    while let (d, u) = h.pop():
        if d > best[u]!: continue
        for a in g.out(u):
            if best[a.to] min= max(d, a.w): h.push((max(d, a.w), a.to))
    best!

let from1 = widest(V.first!)
let fromn = widest(V.last!)
print(es.map(e => e.w + max(e.w, max(min(from1[e.u], from1[e.v]), min(fromn[e.u], fromn[e.v])))).min()!)
