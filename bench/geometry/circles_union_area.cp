use std.geometry.{Vec, Circle, circles_union_area}

let n: Int = read()
var cs: List<Circle> = []
for _ in 0..n:
    let (x, y, r): (F64, F64, F64) = read()
    cs.push(Circle(Vec(x, y), r))
let area = circles_union_area(cs)
print(round(area * 1000.0))
