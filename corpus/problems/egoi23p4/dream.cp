// EGOI 2023 P4: rebuild a weighted graph from two matrices: for each pair, the best bottleneck
// over paths (max of path minimum) and k minus the worst (min of path maximum). Print its edges
// or NO. A pair can carry an edge only where the widest bound is at least the narrowest; the
// matrices are right exactly when those candidates alone close back to them, and then the max
// and min spanning trees of the candidates are the whole answer.
let (n, k) = read<(2..=500, 1..=1_000_000)>()
domain V = [n]
fn triangle(as_bike: Int -> Int) -> [V, V]?Int:
    var m: [V, V]?Int = None
    for v in V:
        for u in ..v:
            let width: 0..=k = read()
            (m[u, v], m[v, u]) = (Some(as_bike(width)), Some(as_bike(width)))
    m
let narrow = triangle(c => k - c)
let wide = triangle(b => b)
fn fits(u: V, v: V) -> Bool = u != v and wide[u, v]! >= narrow[u, v]!
fn rebuilds<R>(d: [V, V]?Int, over: R) -> Bool:
    let c = ([V, V] (u, v) => if fits(u, v): d[u, v] else: None).closure(over:)!
    V.pairs().all((u, v) => c[u, v] == d[u, v])

if not (rebuilds(wide, Semiring(max, min)) and rebuilds(narrow, Semiring(min, max))):
    print("NO")
else:
    let edges = [(u:, v:, w: w!) for (u, v) in V.pairs() if fits(u, v) for w in [wide[u, v], narrow[u, v]]]
    let (low, high) = (kruskal(edges).take, kruskal(edges, by: e => desc(e.w)).take)
    let keep = edges.keys.filter(e => low[e] or high[e]).collect()
    print(keep.len)
    for e in keep: print(edges[e].u, edges[e].v, edges[e].w)
