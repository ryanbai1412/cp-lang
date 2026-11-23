// CF 1063B Labyrinth: 0-1 BFS on cells, left moves cost 1; rights follow from the column.
use std.graph.{bfs01}
let n: 1..=2000 = read()
let m: 1..=2000 = read()
domain R = 1..=n
domain C = 1..=m
let r0: R = read()
let c0: C = read()
let x: 0..=1e9 = read()
let y: 0..=1e9 = read()
let grid = read<[R, C]Char>()
fn arcs(p: (R, C)) -> yields (to: (R, C), w: Int):
    for (q, w) in [(p.up, 0), (p.down, 0), (p.left, 1), (p.right, 0)]:
        if let t = q:
            if grid[t] == '.': yield (to: t, w: w)
let lefts = bfs01(arcs, from: [(r0, c0)])
var reached = 0
for r in R:
    for c in C:
        if let l = lefts[(r, c)]:
            if l <= x and l + c.ord - c0.ord <= y: reached += 1
print(reached)
