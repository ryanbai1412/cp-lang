// DMOPC 2021 C4 P1: axis-aligned right triangles (corner first, then the vertical and horizontal
// legs' ends). For each query point, count triangles containing it, boundary included.
let (n, q) = read (Int, Int)
let tris = read [n](corner: Pt, up: Pt, side: Pt)
for p in read [q]Pt:
    print(tris.count(t => Polygon([t.corner, t.up, t.side]).contains(p, boundary: true)))
