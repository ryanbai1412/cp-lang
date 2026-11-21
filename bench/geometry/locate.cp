use std.geometry.{Location, Pt, locate}

let (n, q): (Int, Int) = read()
var poly: List<Pt> = []
for _ in 0..n:
    let (x, y): (Int, Int) = read()
    poly.push(Pt(x, y))
var (ins, bnd, out) = (0, 0, 0)
for _ in 0..q:
    let (x, y): (Int, Int) = read()
    match locate(poly, Pt(x, y)):
        Location.Inside => ins += 1
        Location.Boundary => bnd += 1
        Location.Outside => out += 1
print(ins, bnd, out)
