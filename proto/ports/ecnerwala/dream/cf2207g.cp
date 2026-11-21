// Codeforces 2207 G: Toothless.
// A set of cells can be colored iff it induces a forest: color each tree outward from a root.
// Cells with (r + c) % 3 != k form disjoint staircases, and some k keeps 2/3 P (pigeonhole).
// Force that class's needy cells in and, while one closes a cycle, drop a plain cell of it;
// by Euler's formula on the planar grid the best k still reaches 2/3 P (editorial's bound).

use std.grid.{adj4}
use std.graph.{cycle_through, dfs_order}

@!multitest
let (n, m): (1..=2000, 1..=2000) = read()
let (a, b): (1..=500_000, 1..=500_000) = read()
domain Row = 0..n
domain Col = 0..m
type Cell = (Row, Col)
let grid: [Row, Col]Char = read()

let needy = [Row, Col] (r, c) => grid[r, c] == '#'
let class = [Row, Col] (r, c) => (r + c) % 3
fn value(p: Cell) -> Int = match grid[p]:
    'x' => b
    '.' => a
    _ => 0

fn forest(k: 0..3) -> [Row, Col]Bool:
    var keep = [Row, Col] (r, c) => needy[r, c] or class[r, c] != k
    let inside = (p: Cell) => p.adj4.filter(q => keep[q])
    for s in grid.keys.filter(s => needy[s] and class[s] == k):
        // Each search is O(nm) and drops a cell: O((nm)^2) per test, 4e6 over all tests.
        while let cycle = cycle_through(inside, s):
            keep[cycle.find(p => grid[p] == '.')!] = false
    keep

let best = (0..3).map(forest).max(by: keep => grid.keys.sum(p => if keep[p]: value(p) else: 0))
let order = dfs_order(p => p.adj4.filter(q => best[q]), within: best)
print(order.len)
for (r, c) in order: print(r + 1, c + 1)
