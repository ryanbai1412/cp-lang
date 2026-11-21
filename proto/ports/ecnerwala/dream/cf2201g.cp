// Codeforces 2201 G: Codeforces Heuristic Contest 1001.
// Cells with (r - c) % 9 in {0, 2, 5, 7} have exactly two Zebra neighbours: zigzag paths along the
// diagonal pairs (d, d + 5), density 4/9 > 1/e. Clip them to an inner square, chain consecutive
// paths' ends through alternate margins and close the chain; each link is a BFS path through cells
// touching nothing chosen but its two ends, and a shortest path has no chords, so the whole stays
// one induced cycle. Not proven: simulated for n = 1001 (425,510 cells); n = 5 is the sample ring.

use std.grid.{moves}
use std.graph.{bfs, components}
use std.seq.{buckets}

let n: 5..=1001 = read()
domain Row = 0..n
domain Col = 0..n
type Cell = (Row, Col)
let zebra = [(dr, dc) for dr in -3..=3 for dc in -3..=3 if dr * dr + dc * dc == 13]

fn ring() -> [Row, Col]Bool = [Row, Col] (r, c) => abs(r - 2) + abs(c - 2) in 2..=3

fn snake() -> [Row, Col]Bool:
    let margin = 12
    var on = [Row, Col] (r, c) =>
        margin <= min(r, c) and max(r, c) < n - margin and (r - c) % 9 in [0, 2, 5, 7]
    let near = (p: Cell) => p.moves(zebra).filter(q => on[q])
    for p in on.keys.filter(p => near(p).len == 0): on[p] = false

    // Each path is its two ends, top-left first, in diagonal order.
    let (_, part) = components(near)
    let ends = buckets(part).map(cells => cells.filter(p => near(p).len == 1))
    let paths = [e.sorted(by: p => p.0 + p.1) for e in ends if e.len == 2].sorted(by: e => e[0].0 - e[0].1)

    // About 430 links: each clears O(n^2) BFS state but searches only near its ends.
    fn link(a: Cell, b: Cell):
        let open = (q: Cell) => not on[q] and q.moves(zebra).all(x => not on[x] or x == a or x == b)
        let ways = bfs(p => p.moves(zebra).filter(q => q == b or open(q)), from: [a], until: b)
        for p in ways.path(b)!: on[p] = true

    for i in 0..paths.len - 1:
        let side = 1 - i % 2
        link(paths[i][side], paths[i + 1][side])
    link(paths.first[0], paths.last[paths.len % 2])
    on

let chosen = if n == 5: ring() else: snake()
for r in Row: print(Col.map(c => if chosen[r, c]: '1' else: '0').join())
