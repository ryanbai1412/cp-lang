// Push-relabel on cf1404e's network for a 400 x 400 all-black grid: each cell takes
// a unit from above (or the source) and passes one left (or to the sink).
use std.flow.{FlowGraph}

domain Row = 0..400
domain Col = 0..400
enum Node:
    Src
    Snk
    Cell(Row, Col)

var g = FlowGraph<Node>()
for r in Row:
    for c in Col:
        let up = if let rp = r.prev: Node.Cell(rp, c) else: Node.Src
        let left = if let cp = c.prev: Node.Cell(r, cp) else: Node.Snk
        g.add(up, Node.Cell(r, c), 1)
        g.add(Node.Cell(r, c), left, 1)
print(g.push_relabel(Node.Src, Node.Snk))
