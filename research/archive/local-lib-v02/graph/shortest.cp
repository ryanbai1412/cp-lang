//! Shortest paths with predecessor pointers (`shortest`) and path reconstruction (`walk_back`).

/// `dijkstra` that also records how each vertex was reached: `via[v]` is the
/// vertex before `v` on a shortest path, `none` at a start. See `walk_back`.
/// O((n + m) log m) with a binary heap.
use std.graph.{Graph}
use std.heap.{MinHeap}
pub fn shortest<V, N, A, W>(next: N, from: List<(V, W)>, w: A -> W) -> (dist: [V]?W, via: [V]?V):
    var dist: [V]?W = none
    var via: [V]?V = none
    var heap = MinHeap<(W, V)>()
    for (s, d) in from:
        if dist[s] == none or d < dist[s]!:
            dist[s] = d
            heap.push((d, s))
    while let (d, u) = heap.pop():
        if dist[u]! < d: continue
        for a in next.out(u):
            let nd = d + w(a)
            if dist[a.to] == none or nd < dist[a.to]!:
                dist[a.to] = nd
                via[a.to] = u
                heap.push((nd, a.to))
    (dist: dist, via: via)

/// The vertices from a start to `t`, following `via` backwards. O(path length).
pub fn walk_back<V>(via: [V]?V, t: V) -> List<V>:
    var path = [t]
    while let p = via[path.last!]: path.push(p)
    path.reverse()
    path

@test
fn shortest_and_walk_back_match_floyd_warshall():
    domain V = 1..=7
    domain E = 1..=14
    for _ in 0..15:
        let edges = [E] e => (u: V.nth(rand(0, 6)), v: V.nth(rand(0, 6)), w: rand(0, 9))
        let g = Graph.directed(edges)
        var best: [V, V]?Int = none
        for v in V: best[v, v] = 0
        for e in E:
            let x = edges[e]
            if best[x.u, x.v] == none or x.w < best[x.u, x.v]!: best[x.u, x.v] = x.w
        for k in V:
            for i in V:
                for j in V:
                    if best[i, k] != none and best[k, j] != none:
                        let via = best[i, k]! + best[k, j]!
                        if best[i, j] == none or via < best[i, j]!: best[i, j] = via
        for s in V:
            let r = shortest(g, from: [(s, 0)], w: a => edges[a.e].w)
            for t in V:
                assert r.dist[t] == best[s, t]
                if r.dist[t] != none:
                    let path = walk_back(r.via, t)
                    assert path[0] == s and path.last! == t
