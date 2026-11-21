// Orient every black cell as horizontal (S-side) or vertical (T-side) in an S-T cut.
// A vertical brick starts at a T-cell whose upper neighbor is missing or on the S-side;
// a horizontal brick starts at an S-cell whose left neighbor is missing or on the T-side.
// Charging 1 to each such start makes cut capacity equal the exact brick count, so min_cut = max_flow.
use std.flow.FlowGraph

let (n, m): (1..=200, 1..=200) = read()
domain Row = 0..n
domain Col = 0..m
let grid: [Row, Col]Char = read()

enum Node { Src, Snk, Cell(Row, Col) }
var fg = FlowGraph<Node>()
for (r, c) in (Row, Col) if grid[r, c] == '#':
    let u = Node.Cell(r, c)
    let up = if let rp = r.prev if grid[rp, c] == '#': Node.Cell(rp, c) else: Node.Src
    let left = if let cp = c.prev if grid[r, cp] == '#': Node.Cell(r, cp) else: Node.Snk
    fg.add(up, u, 1)
    fg.add(u, left, 1)

print(fg.min_cut(Node.Src, Node.Snk))
