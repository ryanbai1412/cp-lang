// CCO 2019 P2, Sirtet: rigid shapes of '#' cells (connected by sides) fall
// straight down and come to rest on the floor or on each other. Print the
// final picture.
//
// A shape can fall as far as the floor lets it, and no further than the
// shape directly below any of its cells plus the gap between them: shortest
// distances from the floor over those gap edges.
// original: corpus/submissions/3613676.cpp
let (n, m): (1..=1e6, 1..=1e6) = read()
domain Row = 0..n
domain Col = 0..m
type Cell = (Row, Col)
let g: [Row, Col]Char = read()

var shapes = Dsu<Cell>()
for p in Cell:
    for q in p.adj4:
        if g[p] == '#' and g[q] == '#': shapes.unite(p, q)

enum Node:
    Floor
    Shape(root: Cell)
fn node(p: Cell) -> Node: Node.Shape(shapes.find(p))

// One edge per vertically consecutive pair of blocks in a column.
var gaps: List<(Node, Node, Int)> = []
for c in Col:
    var (below, below_row) = (Node.Floor, n)
    for r in Row.rev:
        if g[r, c] == '#':
            gaps.push((below, node((r, c)), below_row - r.to_idx - 1))
            (below, below_row) = (node((r, c)), r.to_idx)

let drop = Graph.directed(gaps).dijkstra(from: [Node.Floor])
var out: [Row, Col]Char = '.'
for (r, c) in Cell:
    if g[r, c] == '#': out[(r + drop[node((r, c))]!)!, c] = '#'
for r in Row: print(out.row(r))
