// Source: CCO 2019 P2 (Sirtet), https://dmoj.ca/problem/cco19p2
// Problem: an N x M grid (NM <= 1e6) of '.' and '#'; side-connected '#' cells form rigid objects
//   that all fall straight down at the same speed, each stopping on the bottom row or on top of
//   another object. Print the final grid.
// Solution: label objects by flood fill. In each column, consecutive '#' cells from the bottom
//   give an edge from the object below (the floor is node 0) to the one above, weighted by the
//   empty cells between them. An object falls by its shortest distance from the floor
//   (Dijkstra): O(NM log NM).
// original: corpus/submissions/3613676.cpp
// Bounds: n, m: 1..=1e6
let n: 1..=1e6 = read()
let m: 1..=1e6 = read()
domain Row = 0..n
domain Col = 0..m
type Cell = (Row, Col)
let g: [Row, Col]Char = read()

// Label the shapes 1, 2, ...; 0 is the floor.
var shape: [Row, Col]Int = 0
var shapes = 0
for r in Row:
    for c in Col:
        if g[r, c] != '#' or shape[r, c] != 0: continue
        shapes += 1
        shape[r, c] = shapes
        var stack: List<Cell> = [(r, c)]
        while let p = stack.pop():
            for q in p.adj4:
                if g[q] == '#' and shape[q] == 0:
                    shape[q] = shapes
                    stack.push(q)

domain S = 0..=shapes
var gaps: [S]List<(S, Int)> = []
for c in Col:
    var below = S(0)
    var below_row = n
    for i in 0..n:
        let r = Row(n - 1 - i)
        if g[r, c] == '#':
            let s = S(shape[r, c])
            gaps[below].push((s, below_row - (n - 1 - i) - 1))
            below = s
            below_row = n - 1 - i

var drop: [S]?Int = None
var pq = MinHeap<(Int, S)>()
drop[S(0)] = 0
pq.push((0, S(0)))
while let (d, u) = pq.pop():
    if drop[u] != d: continue
    for (v, w) in gaps[u]:
        if drop[v] == None or d + w < drop[v]!:
            drop[v] = d + w
            pq.push((d + w, v))

var out: [Row, Col]Char = '.'
for r in Row:
    for c in Col:
        if g[r, c] == '#':
            out[Row.at(r + drop[S(shape[r, c])]!)!, c] = '#'
for r in Row: print(out[r, ..])
