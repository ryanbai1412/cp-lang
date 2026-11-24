// expect: ok, output "ok"
// input: "1 300\n"
// model: Dir4
// model: Dir8
// model: up
// model: down
// model: left
// model: right
// model: step
// model: ray
// model: adj4
// model: adj8
// model: transposed
// model: rows
// model: cols
// model: GridRows
// model: GridCols
// Cells, directions and grid views against plain (Int, Int) arithmetic on a 5 x 4 board and a
// List<List<Int>>: seeded random cells, directions, turns, letters and cell writes, then every
// move, ray, neighbour list, transpose and row or column view. Reads `seed n`; prints the first
// op that differs.
use std.rand.{Rng}
use std.grid.{Dir4, Dir8, adj4, adj8, cols, down, left, ray, right, rows, step, transposed, up}
let seed0 = read<Int>()
let n = read<Int>()
domain R = 0..5
domain C = 0..4
const D4 = [(-1, 0), (0, 1), (1, 0), (0, -1)]
const D8 = [(-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1)]
const L4 = "URDL"
const L8 = "abcdefgh"
var rng = Rng(seed0)
fn cell(r: Int, c: Int) -> ?(R, C) = if 0 <= r < 5 and 0 <= c < 4: Some((R(r), C(c))) else: None
fn walk(r: Int, c: Int, d: (Int, Int)) -> List<(R, C)>:
    var out: List<(R, C)> = []
    var (a, b) = (r + d.0, c + d.1)
    while cell(a, b) != None:
        out.push(cell(a, b)!)
        a += d.0
        b += d.1
    out
fn near(r: Int, c: Int, ds: List<(Int, Int)>) -> List<(R, C)>:
    var out: List<(R, C)> = []
    for d in ds:
        if let q = cell(r + d.0, c + d.1): out.push(q)
    out
var g: [R, C]Int = [R, C] (r, c) => 0
var b = [[0 for _ in 0..4] for _ in 0..5]
var bad = ""
if Dir4.all() != [Dir4.up, Dir4.right, Dir4.down, Dir4.left] or Dir8.all().len != 8: bad = "op 0: all"
for k in 0..n:
    if bad != "": break
    let (r, c) = (rng.int(0, 4), rng.int(0, 3))
    let p = (R(r), C(c))
    let i = rng.int(0, 3)
    let j = rng.int(0, 7)
    let d4 = Dir4.all()[i]
    let d8 = Dir8.all()[j]
    if d4.delta != D4[i] or d4.cw != Dir4.all()[(i + 1) % 4] or d4.ccw != Dir4.all()[(i + 3) % 4] or d4.back != Dir4.all()[(i + 2) % 4]:
        bad = "op {k}: Dir4 {i}"
    elif d8.delta != D8[j] or d8.cw != Dir8.all()[(j + 1) % 8] or d8.ccw != Dir8.all()[(j + 7) % 8] or d8.back != Dir8.all()[(j + 4) % 8]:
        bad = "op {k}: Dir8 {j}"
    elif Dir4.from(L4, L4[i]) != Some(d4) or Dir4.from(L4, 'x') != None or Dir8.from(L8, L8[j]) != Some(d8) or Dir8.from(L8, 'z') != None:
        bad = "op {k}: from"
    if bad != "": break
    let (pu, pd, pl, pr) = (up(p), down(p), left(p), right(p))
    if pu != cell(r - 1, c) or pd != cell(r + 1, c) or pl != cell(r, c - 1) or pr != cell(r, c + 1):
        bad = "op {k}: up/down/left/right of ({r}, {c})"
    elif p.step(d4) != cell(r + D4[i].0, c + D4[i].1) or p.step(d8) != cell(r + D8[j].0, c + D8[j].1):
        bad = "op {k}: step ({r}, {c}) by {i}, {j}"
    elif p.ray(d4).collect() != walk(r, c, D4[i]) or p.ray(d8).collect() != walk(r, c, D8[j]):
        bad = "op {k}: ray ({r}, {c}) by {i}, {j}"
    elif p.adj4.collect() != near(r, c, D4) or p.adj8.collect() != near(r, c, D8):
        bad = "op {k}: adj ({r}, {c})"
    if bad != "": break
    let v = rng.int(-9, 9)
    g[p] = v
    b[r][c] = v
    let t = g.transposed()
    let rs = g.rows
    let cs = g.cols
    if rs.len != 5 or rs.is_empty or cs.len != 4 or cs.is_empty: bad = "op {k}: rows/cols len"
    var ri = 0
    for row in rs.each():
        if row.collect() != b[ri]: bad = "op {k}: rows.each row {ri}"
        ri += 1
    var ci = 0
    for col in cs.each():
        if col.collect() != [b[q][ci] for q in 0..5]: bad = "op {k}: cols.each col {ci}"
        ci += 1
    if rs.at(R(r)).collect() != b[r] or cs.at(C(c)).collect() != [b[q][c] for q in 0..5]: bad = "op {k}: at ({r}, {c})"
    for q in 0..5:
        for w in 0..4:
            if t[C(w), R(q)] != b[q][w]: bad = "op {k}: transposed [{w}, {q}]"
print(if bad == "": "ok" else: bad)
