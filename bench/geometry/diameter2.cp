use std.geometry.{Pt, diameter2}

let n: Int = read()
var hull: List<Pt> = []
for _ in 0..n:
    let (x, y): (Int, Int) = read()
    hull.push(Pt(x, y))
var ans = 0
for _ in 0..50:
    ans = diameter2(hull)
print(ans)
