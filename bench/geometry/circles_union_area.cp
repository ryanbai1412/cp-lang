use std.geometry.{Vec, circles_union_area}

let n: Int = read()
var cs: List<(c: Vec, r: F64)> = []
for _ in 0..n:
    let (x, y, r): (F64, F64, F64) = read()
    cs.push((c: Vec(x, y), r: r))
let area = circles_union_area(cs)
print(round(area * 1000.0))
