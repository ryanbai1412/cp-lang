// Source: CCO 2016 P3 (Legends), https://dmoj.ca/problem/cco16p3
// Problem: subtask S in 1..=5 fixes a pattern: Flask (two vertices joined by three disjoint
//   paths), Moon (a cycle), Sun (a cycle of length >= 4), Talon (a claw), Fox (a triangle with
//   pendants on two corners). For each connected simple graph, print YES if it can grow from the
//   pattern by adding edges and vertices and splitting vertices. Sum of n, m <= 1000 (S = 3: 1e5).
// Solution: growing from a pattern means containing a subdivision of it, and every cycle lies in
//   one biconnected block. A block holds a Flask iff it is not a lone cycle (more edges than
//   vertices), a cycle iff it has 3 edges, a long cycle iff it has 4 vertices; a Talon is a vertex
//   of degree 3. A Fox is a cycle with two of its vertices leaving it to two distinct vertices:
//   a block of over 5 vertices has one iff two of its vertices have degree 3; a smaller block
//   tries each of its cycles. O(n + m) per graph, plus 5! cycles per small block.
// original: corpus/submissions/5568296.cpp
use std.graph.{blocks}

let pattern: 1..=5 = read()
let graphs: 1..=1000 = read()

fn grows_from(pattern: 1..=5) -> Bool:
    let n: 1..=100_000 = read()
    let m: 0..=100_000 = read()
    domain V = 1..=n
    domain E = 0..m
    let edges: [E](u: V, v: V) = read()
    let g = Graph<V, E>.undirected(edges)
    let (domain B, block) = blocks(g)
    let block_edges = block.buckets()
    let members = [B] b => [x for e in block_edges[b] for x in [edges[e].u, edges[e].v]].distinct()

    // Two vertices of `cycle` leave it to two distinct vertices.
    fn has_two_exits(cycle: List<V>) -> Bool:
        let outside = [a.to for u in cycle for a in g.out(u) if not cycle.contains(a.to)]
        cycle.count(u => g.out(u).any(a => not cycle.contains(a.to))) >= 2 and outside.distinct().len >= 2

    fn has_fox(vs: List<V>) -> Bool:
        if vs.len > 5: return vs.count(v => g.deg(v) >= 3) >= 2
        (3..=vs.len).any(k => vs.permutations(k).any(cycle =>
            (0..k).all(i => g.out(cycle[i]).any(a => a.to == cycle[(i + 1) % k])) and has_two_exits(cycle)))

    match pattern:
        1 => B.any(b => block_edges[b].len > members[b].len)
        2 => B.any(b => block_edges[b].len >= 3)
        3 => B.any(b => members[b].len >= 4)
        4 => V.any(v => g.deg(v) >= 3)
        _ => B.any(b => has_fox(members[b]))

repeat graphs:
    print(yes(grows_from(pattern)))
