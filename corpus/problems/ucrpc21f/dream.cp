// Source: UCRPC F21 F (Trip Navigator), https://dmoj.ca/problem/ucrpc21f
// limits: 1000 ms, 256 MB
// Problem: an n x n grid (n <= 1000) of empty '-' and banana 'b' cells; walk in 4 directions
//   from the top-left to the bottom-right cell, each move taking 1 second plus t <= 1000 more
//   when it enters a banana cell. Print the shortest time.
// Solution: Dijkstra over the n^2 cells, a move costing 1 or 1 + t by the cell it enters;
//   the cell's moves are its adj4 neighbours, so no graph is built. O(n^2 log n).
// Bounds: n: 1..=1000; t: 0..=1000
// original: corpus/submissions/3944579.cpp
let (n, t): (1..=1000, 0..=1000) = read()
domain Row = [n]
domain Col = [n]
let g: [Row, Col]Char = read()

let moves = (p: (Row, Col)) => [(to: q, w: if g[q] == 'b': 1 + t else: 1) for q in p.adj4]
print(dijkstra(moves, from: [((Row.first!, Col.first!), 0)])[(Row.last!, Col.last!)]!)
