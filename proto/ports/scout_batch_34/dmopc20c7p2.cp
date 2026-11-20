// DMOPC '20 Contest 7 P2 — Planar Square-Octagon Tiling Boundary (CP Lang v2 spec)
// original: corpus/submissions/3686612.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Each tile center `(x, y)` is a unit square when `(x + y) % 4 == 0` and an
// octagon otherwise. Insert each tile's clockwise boundary edges into a map
// from start vertex to `(next_vertex, tile_id)`, pick the lexicographically
// smallest degree-1 vertex, and trace the outer perimeter switching tiles at
// every degree-2 boundary junction.

let n: 1..=2e5 = read()
domain Tile = 0..n
let centers: [Tile]Pt<Int> = read()

let sq_offsets = [Pt(0, 0), Pt(0, 1), Pt(1, 1), Pt(1, 0)]
let oct_offsets = [
    Pt(-1, 0), Pt(-1, 1), Pt(0, 2), Pt(1, 2),
    Pt(2, 1), Pt(2, 0), Pt(1, -1), Pt(0, -1),
]

type Step = (to: Pt<Int>, tile: Tile)
// FRICTION UD12: `Pt<Int>` in v2 (§10) specifies vector ops (`+`, `-`, `dot`,
// `cross`, `norm2`, `turn`) and `ByAngle`, but does not explicitly guarantee
// lexicographic `Ord` and `Hash` for `Map<Pt<Int>, ...>` and `min(p for ...)`.
// In `stdlib/geometry.cp` today, `ByXY` is a separate `Order<Pt>`, confirming
// `Pt` lacks default `Ord`.
var nxt = Map<Pt<Int>, List<Step>>()

for (t, c) in centers.items:
    let shape = if (c.x + c.y) % 4 == 0: sq_offsets else: oct_offsets
    let m = shape.len
    for j in 0..m:
        let u = c + shape[j]
        let v = c + shape[(j + 1) % m]
        (nxt[u] ??= []).push((to: v, tile: t))

let start = min(p for (p, es) in nxt.items if es.len == 1)!
var cur = start
var cur_tile = nxt[start][0].tile
var perimeter: List<Pt<Int>> = []

// No `do ... while` in v2: `while true:` with trailing `if cur == start: break`.
while true:
    perimeter.push(cur)
    let es = nxt[cur]
    let step = if es.len == 1:
        es[0]
    elif es[0].tile != cur_tile:
        es[0]
    else:
        es[1]
    (cur, cur_tile) = (step.to, step.tile)
    if cur == start: break

print(perimeter.len)
for p in perimeter:
    print("{p.x} {p.y}")
