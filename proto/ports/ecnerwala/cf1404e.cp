// Orient every black cell as horizontal (S-side) or vertical (T-side) in an S-T cut.
// A vertical brick starts at a T-cell whose upper neighbor is missing or on the S-side;
// a horizontal brick starts at an S-cell whose left neighbor is missing or on the T-side.
// Charging 1 to each such start makes cut capacity equal the exact brick count, so min_cut = max_flow.
use std.flow.FlowGraph

let n: 1..=200 = read()
let m: 1..=200 = read()
domain Row = 0..n
domain Col = 0..m
let rows: [Row]Str = read()

domain Node = 0..n * m + 2
let src = Node(n * m)
let snk = Node(n * m + 1)
fn cell(r: Row, c: Col) -> Node = Node(r.ord * m + c.ord)

var fg = FlowGraph<Node>()
for r in Row:
    for c in Col if rows[r][c.ord] == '#':
        let u = cell(r, c)
        var up = src
        if let rp = r.prev:
            if rows[rp][c.ord] == '#': up = cell(rp, c)
        var left = snk
        if let cp = c.prev:
            if rows[r][cp.ord] == '#': left = cell(r, cp)
        fg.add(up, u, 1)
        fg.add(u, left, 1)

print(fg.max_flow(src, snk))
