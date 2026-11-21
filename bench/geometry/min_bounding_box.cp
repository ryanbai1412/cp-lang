use std.geometry.{min_bounding_box}

let n: Int = read()
var hull: List<Pt> = []
for _ in 0..n:
    let (x, y): (Int, Int) = read()
    hull.push(Pt(x, y))
var sum_a = 0.0
var sum_p = 0.0
for _ in 0..50:
    let b = min_bounding_box(hull)
    sum_a += b.area
    sum_p += b.perimeter
print(round(sum_a / 1e12), round(sum_p / 1e4))
