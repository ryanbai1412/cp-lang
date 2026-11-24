// Tuned: the blocks by hand over flat Int arc lists instead of `blocks(g)`, which builds a list
//   of arcs per vertex.
//
// Source: CCO 2016 P3 (Legends), https://dmoj.ca/problem/cco16p3
// limits: 1000 ms, 1024 MB
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
// Bounds: pattern: 1..=5; graphs: 1..=1000; n: 1..=100_000; m: 0..=100_000; edges.u, edges.v: 1..=n

let pattern: 1..=5 = read()
let graphs: 1..=1000 = read()

fn grows_from(pattern: 1..=5) -> Bool:
    let n: 1..=100_000 = read()
    let m: 0..=100_000 = read()
    domain V = 1..=n
    domain E = 0..m
    let edges: [E](u: V, v: V) = read()
    let g = Graph<V, E>.undirected(edges)
    // `blocks(g)` by hand over flat arc lists: arc 2e runs u -> v and arc 2e + 1 back
    var head: List<Int> = [-1 for v in 0..n]
    var link: List<Int> = [-1 for a in 0..2 * m]
    var to: List<Int> = [0 for a in 0..2 * m]
    for e in E:
        let (u, v) = (edges[e].u.ord, edges[e].v.ord)
        to[2 * e.ord] = v
        link[2 * e.ord] = head[u]
        head[u] = 2 * e.ord
        to[2 * e.ord + 1] = u
        link[2 * e.ord + 1] = head[v]
        head[v] = 2 * e.ord + 1
    var tin: List<Int> = [-1 for v in 0..n]
    var low: List<Int> = [0 for v in 0..n]
    var arc: List<Int> = [-1 for v in 0..n]
    var tree_edge: List<Int> = [-1 for v in 0..n]
    var raw: List<Int> = [0 for e in 0..m]
    var path: List<Int> = []
    var open_edges: List<Int> = []
    var timer = 0
    var count = 0
    for r in 0..n:
        if tin[r] >= 0: continue
        tin[r] = timer
        low[r] = timer
        timer += 1
        arc[r] = head[r]
        path.push(r)
        while path.len > 0:
            let u = path.last!
            let a = arc[u]
            if a >= 0:
                arc[u] = link[a]
                let v = to[a]
                if a >> 1 == tree_edge[u]: continue
                if tin[v] < 0:
                    open_edges.push(a >> 1)
                    tree_edge[v] = a >> 1
                    tin[v] = timer
                    low[v] = timer
                    timer += 1
                    arc[v] = head[v]
                    path.push(v)
                elif tin[v] < tin[u]:
                    low[u] min= tin[v]
                    open_edges.push(a >> 1)
            else:
                let _ = path.pop()
                if path.len > 0:
                    let p = path.last!
                    low[p] min= low[u]
                    if low[u] >= tin[p]:
                        while let x = open_edges.pop():
                            raw[x] = count
                            if x == tree_edge[u]: break
                        count += 1
    domain B = 0..count
    let block = [E] e => B.nth(raw[e.ord])
    var length: [B]Int = 0
    for e in E: length[block[e]] += 1
    var size: [B]Int = 0  // vertices per block: each vertex counts once in each of its blocks
    var counted: [B]?V = None
    for v in V:
        for a in g.out(v):
            if counted[block[a.e]] != Some(v):
                counted[block[a.e]] = Some(v)
                size[block[a.e]] += 1
    fn members(b: B) -> List<V>: [x for e in E if block[e] == b for x in [edges[e].u, edges[e].v]].distinct()

    // Two vertices of `cycle` leave it to two distinct vertices.
    fn has_two_exits(cycle: List<V>) -> Bool:
        let outside = [a.to for u in cycle for a in g.out(u) if not cycle.contains(a.to)]
        cycle.count(u => g.out(u).any(a => not cycle.contains(a.to))) >= 2 and outside.distinct().len >= 2

    fn has_fox(vs: List<V>) -> Bool:
        if vs.len > 5: return vs.count(v => g.deg(v) >= 3) >= 2
        (3..=vs.len).any(k => vs.permutations(k).any(cycle =>
            (0..k).all(i => g.out(cycle[i]).any(a => a.to == cycle[(i + 1) % k])) and has_two_exits(cycle)))

    match pattern:
        1 => B.any(b => length[b] > size[b])
        2 => B.any(b => length[b] >= 3)
        3 => B.any(b => size[b] >= 4)
        4 => V.any(v => g.deg(v) >= 3)
        _ => B.any(b => has_fox(members(b)))

repeat graphs:
    print(yes(grows_from(pattern)))
