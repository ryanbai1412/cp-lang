// Codeforces 1404 E: Bricks.
//
// Tile the black cells with 1-wide bricks. Give every black cell an
// orientation: horizontal (S side) or vertical (T side). A horizontal brick
// starts at a horizontal cell whose left neighbour is not a horizontal black
// cell; a vertical brick at a vertical cell whose upper neighbour is not a
// vertical black cell. Bricks = starts, and each start is one unit edge cut:
// up -> cell (cut when the cell is vertical and up is not), cell -> left (cut
// when the cell is horizontal and left is not), with a missing neighbour
// played by the source above and the sink to the left. So the fewest bricks
// is the min cut, which max flow finds.
use std.flow.FlowGraph

let (n, m): (1..=200, 1..=200) = read()
domain Row = 0..n
domain Col = 0..m
let grid: [Row, Col]Char = read()
fn black(r: Row, c: Col) -> Bool = grid[r, c] == '#'

enum Node:
    Src
    Snk
    Cell(Row, Col)

var g = FlowGraph<Node>()
for r in Row:
    for c in Col if black(r, c):
        let up = if let rp = r.prev and black(rp, c): Node.Cell(rp, c) else: Node.Src
        let left = if let cp = c.prev and black(r, cp): Node.Cell(r, cp) else: Node.Snk
        g.add(up, Node.Cell(r, c), 1)
        g.add(Node.Cell(r, c), left, 1)
print(g.max_flow(Node.Src, Node.Snk))
