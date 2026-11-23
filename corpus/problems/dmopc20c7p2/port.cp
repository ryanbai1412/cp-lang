// Runnable port (proto/cp.py lib-run) of dmopc20c7p2.cp
// original: corpus/submissions/3686612.cpp

let n: Int = read()
domain Tile = 0..n
let raw: [Tile](x: Int, y: Int) = read()

let sq = [Pt(0, 0), Pt(0, 1), Pt(1, 1), Pt(1, 0)]
let oct = [Pt(-1, 0), Pt(-1, 1), Pt(0, 2), Pt(1, 2), Pt(2, 1), Pt(2, 0), Pt(1, -1), Pt(0, -1)]

struct Step(to: Pt<Int>, tile: Tile)

var nxt = Map<Pt<Int>, List<Step>>()
var keys: List<Pt<Int>> = []

for t in Tile:
    let c = Pt(raw[t].x, raw[t].y)
    let shape = if (c.x + c.y) % 4 == 0: sq else: oct
    let m = shape.len
    for j in 0..m:
        let p = c + shape[j]
        let q = c + shape[(j + 1) % m]
        if not (p in nxt):
            nxt[p] = []
            keys.push(p)
        nxt[p].push(Step(q, t))

var start: ?Pt<Int> = none
for k in keys:
    if nxt[k].len == 1:
        if let cur = start:
            if k < cur: start = k
        else:
            start = k

let s0 = start!
var cur = s0
var cur_tile = nxt[s0][0].tile
var perimeter: List<Pt<Int>> = []

while true:
    perimeter.push(cur)
    let es = nxt[cur]
    let step = if es.len == 1: es[0] else: if es[0].tile != cur_tile: es[0] else: es[1]
    cur = step.to
    cur_tile = step.tile
    if cur == s0: break

print(perimeter.len)
for p in perimeter:
    print("{p.x} {p.y}")
