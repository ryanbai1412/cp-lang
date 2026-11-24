// fixed: rows
use std.grid.{Dir4, Dir8}
let n: Int = read()
let reps = 1 << 17
var s = 0
domain R = 0..n
domain C = 0..n

for i in 0..reps:
    s += Dir4.all().len // timed: grid.Dir4.all
    s += Dir8.all().len // timed: grid.Dir8.all
    let d = Dir4.all()[i % 4]
    let e = Dir8.all()[i % 8]
    if Dir4.up == d: s += 1 // timed: grid.Dir4.up
    if Dir4.down == d: s += 1 // timed: grid.Dir4.down
    if Dir4.left == d: s += 1 // timed: grid.Dir4.left
    if Dir4.right == d: s += 1 // timed: grid.Dir4.right
    if d.cw == Dir4.up: s += 1 // timed: grid.Dir4.cw
    if d.ccw == Dir4.up: s += 1 // timed: grid.Dir4.ccw
    if d.back == Dir4.up: s += 1 // timed: grid.Dir4.back
    s += d.delta.0 // timed: grid.Dir4.delta
    if e.cw == Dir8.all()[0]: s += 1 // timed: grid.Dir8.cw
    if e.ccw == Dir8.all()[0]: s += 1 // timed: grid.Dir8.ccw
    if e.back == Dir8.all()[0]: s += 1 // timed: grid.Dir8.back
    s += e.delta.1 // timed: grid.Dir8.delta
    if Dir4.from("URDL", "URDL"[i % 4]) == Some(d): s += 1 // timed: grid.Dir4.from
    if Dir8.from("abcdefgh", "abcdefgh"[i % 8]) == Some(e): s += 1 // timed: grid.Dir8.from
    let p = (R(i * 7919 % n), C(i * 13 % n))
    if p.up != None: s += 1 // timed: grid.up
    if p.down != None: s += 1 // timed: grid.down
    if p.left != None: s += 1 // timed: grid.left
    if p.right != None: s += 1 // timed: grid.right
    if p.step(d) != None: s += 1 // timed: grid.step
    if p.step(e) != None: s += 1 // timed: grid.step
    for (r, c) in p.adj4: // timed: grid.adj4
        s += r.ord + c.ord // timed: grid.adj4
    for (r, c) in p.adj8: // timed: grid.adj8
        s += r.ord + c.ord // timed: grid.adj8
for _ in 0..40:
    for (r, c) in (R(0), C(0)).ray(Dir8.all()[3]): // timed: grid.ray
        s += r.ord + c.ord // timed: grid.ray
    for (r, c) in (R(n - 1), C(0)).ray(Dir4.right): // timed: grid.ray
        s += r.ord + c.ord // timed: grid.ray

// a 64-row grid with n / 64 columns: rows fixed, columns grow
domain GR = 0..64
domain GC = 0..n / 64
let grid: [GR, GC]Int = [GR, GC] (r, c) => r.ord * 7 + c.ord
for _ in 0..200:
    s += grid.transposed()[GC(0), GR(1)] // timed: grid.transposed
for i in 0..reps:
    let rs = grid.rows // timed: grid.rows
    let cs = grid.cols // timed: grid.cols
    s += rs.len // timed: grid.GridRows.len
    s += cs.len // timed: grid.GridCols.len
    if rs.is_empty: s += 1 // timed: grid.GridRows.is_empty
    if cs.is_empty: s += 1 // timed: grid.GridCols.is_empty
print(s)
