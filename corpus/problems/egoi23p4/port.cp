// Source: EGOI 2023 Day 1 P4 (Bikes vs Cars), https://dmoj.ca/problem/egoi23p4
// Problem: n <= 500 places, streets of width k <= 1e6 split into a bike lane b and a car lane
//   k - b. Given, per pair, the widest car C and the widest bike B that can travel between them
//   (and no wider), print at most 2023 streets u v b that realise both, or NO.
// Solution: in bike-lane terms B is a widest-path (max, min) matrix and k - C a narrowest-path
//   (min, max) one, so both must be closed under their path algebra. A street u-v may carry only
//   b between the two, so offer both ends where that range is non-empty; if the offers close
//   back to both matrices, a minimum and a maximum spanning forest of them keep every
// original: corpus/submissions/5678052.cpp
// Bounds: n: 2..=500; k: 1..=1_000_000; width: 0..=k
//   bottleneck in at most 2(n - 1) streets. O(n^3).
let (n, k) = read<(2..=500, 1..=1_000_000)>()
domain V = [n]
// the matrices come as lower triangles, row by row; read them in bike-lane terms
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
fn only_fits(d: [V, V]?Int) -> [V, V]?Int = [V, V] (u, v) => if fits(u, v): d[u, v] else: None
fn closes_to<R>(d: [V, V]?Int, over: R, want: [V, V]?Int) -> Bool:
    let c = d.closure(over:)!
    (V, V).all((u, v) => u == v or c[u, v] == want[u, v])
let widest = Semiring(max, min)
let narrowest = Semiring(min, max)
if not (
    closes_to(wide, widest, wide) and closes_to(narrow, narrowest, narrow) and closes_to(
        only_fits(wide),
        widest,
        wide,
    ) and closes_to(
        only_fits(narrow),
        narrowest,
        narrow,
    )
):
    print("NO")
else:
    let edges = [
        (u:, v:, w: w!) for (u, v) in V.pairs() if fits(u, v) for w in [wide[u, v], narrow[u, v]]
    ]
    let low = kruskal(edges).take
    let high = kruskal(edges, by: e => desc(e.w)).take
    let keep = edges.keys.filter(e => low[e] or high[e]).collect()
    print(keep.len)
    for e in keep:
        print(edges[e].u, edges[e].v, edges[e].w)
