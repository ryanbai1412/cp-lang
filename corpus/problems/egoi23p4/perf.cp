// Tuned: each closure check is a plain Int Floyd-Warshall in its own (max, min) or (min, max)
// algebra on a narrow -1..=1e6 + 1 grid (-1 or k + 1 for a missing street), instead of the
// generic optional closure.
//
// Source: EGOI 2023 Day 1 P4 (Bikes vs Cars), https://dmoj.ca/problem/egoi23p4
// Problem: n <= 500 places, streets of width k <= 1e6 split into a bike lane b and a car lane
//   k - b. Given, per pair, the widest car C and the widest bike B that can travel between them
//   (and no wider), print at most 2023 streets u v b that realise both, or NO.
// Solution: in bike-lane terms B is a widest-path (max, min) matrix and k - C a narrowest-path
//   (min, max) one, so both must be closed under their path algebra. A street u-v may carry only
//   b between the two, so offer both ends where that range is non-empty; if the offers close
//   back to both matrices, a minimum and a maximum spanning forest of them keep every
//   bottleneck in at most 2(n - 1) streets. O(n^3).
// original: corpus/submissions/5678052.cpp
// Bounds: n: 2..=500; k: 1..=1_000_000; width: 0..=k
let (n, k) = read<(2..=500, 1..=1_000_000)>()
type Lane = -1..=1_000_001  // a lane width, or one past either end for a missing street
domain V = [n]
// the matrices come as lower triangles, row by row; read them in bike-lane terms
fn triangle(as_bike: Int -> Int) -> [V, V]Int:
    var m: [V, V]Int = 0
    for v in V:
        for u in ..v:
            let width: 0..=k = read()
            (m[u, v], m[v, u]) = (as_bike(width), as_bike(width))
    m
let narrow = triangle(c => k - c)
let wide = triangle(b => b)
fn fits(u: V, v: V) -> Bool = u != v and wide[u, v] >= narrow[u, v]
// widest paths (max of min) over the streets that fit, -1 where none
fn widest_ok(only: Bool) -> Bool:
    var c: [V, V]Lane = -1
    for (u, v) in (V, V) if u != v and (not only or fits(u, v)): c[u, v] = wide[u, v]
    for m in V:
        for u in V:
            let via = c[u, m]
            for v in V: c[u, v] max= min(via, c[m, v])
    (V, V).all((u, v) => u == v or c[u, v] == wide[u, v])
// narrowest paths (min of max) over the streets that fit, k + 1 where none
fn narrowest_ok(only: Bool) -> Bool:
    var c: [V, V]Lane = k + 1
    for (u, v) in (V, V) if u != v and (not only or fits(u, v)): c[u, v] = narrow[u, v]
    for m in V:
        for u in V:
            let via = c[u, m]
            for v in V: c[u, v] min= max(via, c[m, v])
    (V, V).all((u, v) => u == v or c[u, v] == narrow[u, v])
if not (widest_ok(false) and narrowest_ok(false) and widest_ok(true) and narrowest_ok(true)):
    print("NO")
else:
    let edges = [
        (u:, v:, w:) for (u, v) in V.pairs() if fits(u, v) for w in [wide[u, v], narrow[u, v]]
    ]
    let low = kruskal(edges).take
    let high = kruskal(edges, by: e => desc(e.w)).take
    let keep = edges.keys.filter(e => low[e] or high[e]).collect()
    print(keep.len)
    for e in keep:
        print(edges[e].u, edges[e].v, edges[e].w)
