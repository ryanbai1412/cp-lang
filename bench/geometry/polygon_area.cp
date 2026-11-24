use std.geometry.{Vec}

let n: Int = read()
var ipoly: List<Pt> = []
var fpoly: List<Vec> = []
for _ in 0..n:
    let (x, y): (Int, Int) = read()
    ipoly.push(Pt(x, y))
    fpoly.push(Vec(F64(x), F64(y)))
var acc_i = 0
var acc_f = 0.0
for _ in 0..100:
    acc_i += area2(ipoly)
    acc_f += polygon_area(fpoly)
print("{acc_i} {acc_f:.2f}")
