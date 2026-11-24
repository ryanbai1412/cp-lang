// Source: Codeforces 1404E (Bricks), https://codeforces.com/contest/1404/problem/E
// Problem: an n x m grid (n, m <= 200) of black '#' and white '.' cells, at least one black.
//   Cover the black cells exactly with 1-wide bricks (1 x k or k x 1); print the fewest bricks.
// Solution: give each black cell an orientation, horizontal (source side) or vertical (sink
//   side). Bricks = starts: a vertical cell whose upper neighbour is not a vertical black cell,
//   or a horizontal cell whose left neighbour is not a horizontal black cell. Each start is one
//   cut unit edge, up -> cell or cell -> left, with a missing upper neighbour played by the source
//   and a missing left one by the sink, so the answer is the min cut: push-relabel max flow on
//   n m + 2 nodes with 2 edges per black cell.

let (n, m): (1..=200, 1..=200) = read()
domain Row = 0..n
domain Col = 0..m
domain Square = (Row, Col)
let grid: [Row, Col]Char = read()
let black = [Row, Col] (r, c) => grid[r, c] == '#'

enum Node:
    Src
    Snk
    Cell(Row, Col)
var g = FlowGraph<Node>()
for (r, c) in Square if black[r, c]:
    let up = if let rp = r.prev and black[rp, c]: Node.Cell(rp, c) else: Node.Src
    let left = if let cp = c.prev and black[r, cp]: Node.Cell(r, cp) else: Node.Snk
    g.add(up, Node.Cell(r, c), 1)
    g.add(Node.Cell(r, c), left, 1)
print(g.push_relabel(Node.Src, Node.Snk))
