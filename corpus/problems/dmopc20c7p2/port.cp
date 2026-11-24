// Source: DMOPC '20 Contest 7 P2 (Alice and Tiles), https://dmoj.ca/problem/dmopc20c7p2
// Problem: N <= 2e5 tiles of the square-octagon tiling, given by coordinates |x|, |y| <= 1e9,
//   form one polygon without holes. Print its vertices clockwise, starting from the one with
//   least x, then least y.
// Solution: each tile lists its own edges clockwise, square or octagon by its coordinates;
//   sorted by start corner, a corner's edges sit together. Start at the least corner on one tile and
//   walk; where two tiles meet, leave by the tile that was not just walked, skipping the shared
//   edge, until back at the start. O(N log N).
// original: corpus/submissions/3686612.cpp
// Bounds: n: 1..=2e5; center.x, center.y: -1e9..=1e9
let n: 1..=2e5 = read()
domain Tile = 0..n
let center: [Tile](x: -1e9..=1e9, y: -1e9..=1e9) = read()

let square = [Pt(0, 0), Pt(0, 1), Pt(1, 1), Pt(1, 0)]
let octagon = [Pt(-1, 0), Pt(-1, 1), Pt(0, 2), Pt(1, 2), Pt(2, 1), Pt(2, 0), Pt(1, -1), Pt(0, -1)]

// every tile lists its edges clockwise; sorted by start, a corner's edges sit together
var edges: List<(from: Pt<Int>, to: Pt<Int>, tile: Tile)> = []
for t in Tile:
    let c = Pt(center[t].x, center[t].y)
    let shape = if (c.x + c.y) % 4 == 0: square else: octagon
    let corners = [c + d for d in shape]
    for (p, q) in (corners + [corners[0]]).adjacent: edges.push((p, q, t))
edges.sort(by: e => e.from)
fn leaving(p: Pt<Int>) -> List<(from: Pt<Int>, to: Pt<Int>, tile: Tile)> = [edges[i] for i in edges.equal_range(p, by: e => e.from)]
let start = min(e.from for e in edges if leaving(e.from).len == 1)!
var at = start
var by = leaving(start)[0].tile
var boundary: List<Pt<Int>> = []
loop:
    boundary.push(at)
    let out = leaving(at)
    let step = out.find(e => e.tile != by) ?? out[0]
    at = step.to
    by = step.tile
    if at == start: break
print(boundary.len)
for p in boundary: print(p.x, p.y)
