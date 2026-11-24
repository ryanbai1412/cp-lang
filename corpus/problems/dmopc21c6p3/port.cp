// Source: DMOPC '21 Contest 6 P3, An Art Problem, https://dmoj.ca/problem/dmopc21c6p3
// Problem: an N x M grid (N, M, K <= 1500) of colours c <= 1e9, 0 meaning blank. Every blank
//   pixel within K four-neighbour steps of a coloured one takes the colour of the closest
//   coloured pixel, the lowest colour on a tie. Print the grid after this expansion.
// Solution: a multi-source `bfs` from every coloured cell, entering cells at most K steps out.
//   In BFS order each claimed cell takes the lowest colour among its neighbours one step
//   closer, which is the lowest colour of the closest sources. O(NM).
// original: corpus/submissions/4342005.cpp
// Smaller colours start first, so they win every tie.

let n: 1..=1500 = read()
let m: 1..=1500 = read()
let k: 0..=3000 = read()
domain Row = 1..=n
domain Col = 1..=m
domain Cell = (Row, Col)
var paint: [Row, Col]0..=1e9 = read()

let reach = bfs(
    (p: Cell) => p.adj4,
    from: [p for p in Cell if paint[p] != 0],
    enter: (p, d) => d <= k,
)
for (v, d) in reach.iter() if d > 0:
    paint[v] = min(paint[u] for u in v.adj4 if reach[u] == Some(d - 1))!
print(paint)
