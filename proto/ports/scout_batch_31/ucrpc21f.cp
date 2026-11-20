// UCRPC 2021 F: cheapest walk from the top-left to the bottom-right cell of
// an n x n board; entering any cell costs 1, entering a 'b' cell costs t more.
// original: corpus/submissions/3944579.cpp
let (n, t): (1..=1000, 0..=1e9) = read()
domain Row = 0..n
domain Col = 0..n
type Cell = (Row, Col)
let g: [Row, Col]Char = read()

let dist = dijkstra(p => (q, if g[q] == 'b': 1 + t else: 1) for q in p.adj4,
                    from: [(Row.first, Col.first)])
print(dist[Row.last, Col.last]!)
