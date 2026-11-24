# Friction report: cco19p2

## Dream

The statement bounds only the cell count (NM <= 1e6), so the grid is
`[Row, Col]` with a checked joint bound:

```cp
let (n, m): (1..=1e6, 1..=1e6) = read()
let cells: 1..=1e6 = n * m
domain Row = 0..n
domain Col = 0..m
type Cell = (Row, Col)
let g: [Row, Col]Char = read()

let touching: [Row, Col]List<Cell> = [Row, Col] p => if g[p] == '#': [q for q in p.adj4 if g[q] == '#'] else: []
let (Shape, shape) = components(touching)
domain Node = 0..=Shape.count  // 0 is the floor
fn node(s: Shape) -> Node = Node(1 + s.ord)

// in each column, every '#' rests `w` empty rows above the next '#' (or the floor) below it
var rests: List<(u: Node, v: Node, w: Int)> = []
for c in Col:
    var (below, top) = (Node(0), n)
    for r in Row.rev if g[r, c] == '#':
        rests.push((u: below, v: node(shape[r, c]), w: top - r.ord - 1))
        (below, top) = (node(shape[r, c]), r.ord)
domain Rest = 0..rests.len
let arcs: [Rest](u: Node, v: Node, w: Int) = [Rest] e => rests[e.ord]
let fall = dijkstra(Graph.directed(arcs), from: [(Node(0), 0)])

var out: [Row, Col]Char = '.'
for (r, c) in Cell if g[r, c] == '#': out[Row(r.ord + fall[node(shape[r, c])]!), c] = '#'
for r in Row: print(out[r, ..])
```

## Blocked: a grid's peak ignores the joint cell bound

`[Row, Col]` tables get a static peak of 1e6 * 1e6 cells (E0904, "takes
1000 GB"), even after `let cells: 1..=1e6 = n * m`. Spec R03.1.1 narrows a
one-dimensional `domain Val = 1..=nm` from a joint bound, but a product of
two domains still multiplies their separate bounds. The v0.2 port fails
`cpl check` for the same reason. The only spelling that checks today is a
flat `0..cells` domain with hand row/column arithmetic and a hand `adj4`,
which is what the dream replaces, so the port waits for a fix. Repro sent
to Core.
