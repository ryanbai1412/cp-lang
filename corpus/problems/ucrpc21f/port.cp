// UCRPC 2021 F: cheapest walk from the top-left to the bottom-right cell of
// an n x n board; entering any cell costs 1, entering a 'b' cell costs t more.
// Runnable on proto/cpc today.
// original: corpus/submissions/3944579.cpp
let n: 1..=1000 = read()
let t: 0..=1e9 = read()
domain Row = 0..n
domain Col = 0..n
type Cell = (Row, Col)
let g: [Row, Col]Char = read()

let src: Cell = (Row.first!, Col.first!)
var dist: [Row, Col]?Int = None
var pq = MinHeap<(Int, Cell)>()
dist[src] = 0
pq.push((0, src))
while let (d, p) = pq.pop():
    if dist[p] != d: continue
    for q in p.adj4:
        let nd = d + (if g[q] == 'b': 1 + t else: 1)
        if dist[q] == None or nd < dist[q]!:
            dist[q] = nd
            pq.push((nd, q))
print(dist[(Row.last!, Col.last!)]!)
