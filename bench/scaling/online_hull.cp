// The hull's corners are the 2n points of two parabolas, added in a scattered order, so it holds
// about 2n corners.
use std.geometry.{Location, OnlineHull, Pt}
let n: Int = read()
var acc = 0
let top = 2 * n * n
fn low(i: Int) -> Pt = Pt(i, i * i)
fn high(i: Int, top: Int) -> Pt = Pt(i, top - i * i)
var h = OnlineHull()
for r in 0..1:
    h = OnlineHull()
    for k in 0..n:
        let i = k * 40503 % n
        if h.add(low(i)): acc += 1 // timed: geometry.OnlineHull.add
        if h.add(high(i, top)): acc += 1 // timed: geometry.OnlineHull.add
for i in 0..8 * n:
    let e = OnlineHull() // timed: geometry.OnlineHull.new
    acc += e.area
    acc += h.area2 % 7 // timed: geometry.OnlineHull.area2
    acc += h.len // timed: geometry.OnlineHull.len
    if h.is_empty: acc += 1 // timed: geometry.OnlineHull.is_empty
for i in 0..2 * n:
    let q = Pt(i % n, (i * 7919) % top)
    if h.locate(q) == Location.Inside: acc += 1 // timed: geometry.OnlineHull.locate
    acc += (h.extreme(Pt(i % 7 - 3, i % 5 - 2)) ?? Pt(0, 0)).x // timed: geometry.OnlineHull.extreme
for r in 0..16:
    acc += h.vertices().len // timed: geometry.OnlineHull.vertices
    for p in h.iter(): acc += p.x % 3 // timed: geometry.OnlineHull.iter
var c = OnlineHull()
for i in 0..4 * n:
    c.add(Pt(i, 0))
    c.add(Pt(0, i + 1))
    c.clear() // timed: geometry.OnlineHull.clear
print(acc)
