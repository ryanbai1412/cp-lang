// Source: CCO 2019 P2 (Sirtet), https://dmoj.ca/problem/cco19p2
// limits: 1000 ms, 1024 MB
// Problem: an N x M grid (NM <= 1e6) of '.' and '#'; side-connected '#' cells form rigid objects
//   that all fall straight down at the same speed, each stopping on the bottom row or on top of
//   another object. Print the final grid.
// Solution: a Dsu over cells joins touching '#' cells into objects, named by their root cell. In
//   each column the lowest '#' starts its object at its height above the floor, and each next
//   '#' up gives an arc from the object below to its own, weighted by the empty cells between.
//   An object falls by its shortest distance (multi-source Dijkstra): O(NM log NM).
// original: corpus/submissions/3613676.cpp
// Bounds: n, m: 1..=1e6
let (n, m): (1..=1e6, 1..=1e6) = read()
let cells: 1..=1e6 = n * m
domain Row = 0..n
domain Col = 0..m
domain Cell = (Row, Col)
let g: [Row, Col]Char = read()

var objects = Dsu<Cell>()
for p in Cell if g[p] == '#':
    for q in p.adj4 if g[q] == '#': objects.unite(p, q)
fn object(p: Cell) -> Cell = objects.find(p)

// in each column the lowest '#' is its height above the floor, each next one `w` rows above the last
var floor: List<(Cell, Int)> = []
var rests: List<(u: Cell, v: Cell, w: Int)> = []
for c in Col:
    var below: ?Row = None
    for r in Row.rev if g[r, c] == '#':
        if let b = below: rests.push((u: object((b, c)), v: object((r, c)), w: b.ord - r.ord - 1))
        else: floor.push((object((r, c)), n - 1 - r.ord))
        below = Some(r)
domain Rest = 0..rests.len
let arcs: [Rest](u: Cell, v: Cell, w: Int) = [Rest] e => rests[e.ord]
let fall = dijkstra(Graph.directed(arcs), from: floor)

var out: [Row, Col]Char = '.'
for (r, c) in Cell if g[r, c] == '#': out[Row(r.ord + fall[object((r, c))]!), c] = '#'
for r in Row: print(out[r, ..])
