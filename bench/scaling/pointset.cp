// n: 4096
use std.geometry.{PointSet, Pt}
let n: Int = read()
let pts = [Pt(i, i * 7 % 101) for i in 0..n]
let s = PointSet.of(pts)
var acc = 0
for i in 0..1 << 18:
    if s.is_empty: acc += i // timed: geometry.PointSet.is_empty
    acc += s.len // timed: geometry.PointSet.len
for r in 0..64:
    for k in s.each(): acc += k // timed: geometry.PointSet.each
for r in 0..4:
    let t = PointSet.of(pts) // timed: geometry.PointSet.of
    acc += t.len
for r in 0..8:
    var d = PointSet.of(pts)
    for i in 0..n:
        let p = pts[i]
        acc += d.drain(p.x..p.x + 1, p.y..p.y + 1).len // timed: geometry.PointSet.drain [fixed: k]
for r in 0..16:
    var e = PointSet.of(pts)
    for i in 0..n:
        if e.contains(i): acc += 1 // timed: geometry.PointSet.contains
        if e.remove(i): acc += 1 // timed: geometry.PointSet.remove
print(acc)
