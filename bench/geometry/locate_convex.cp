use std.geometry.{Location, Pt, locate_convex}

let (n, q): (Int, Int) = read()
var hull: List<Pt> = []
for _ in 0..n:
    let (x, y): (Int, Int) = read()
    hull.push(Pt(x, y))
var (ins, bnd, out) = (0, 0, 0)
for _ in 0..q:
    let (x, y): (Int, Int) = read()
    match locate_convex(hull, Pt(x, y)):
        Location.Inside => ins += 1
        Location.Boundary => bnd += 1
        Location.Outside => out += 1
print(ins, bnd, out)
