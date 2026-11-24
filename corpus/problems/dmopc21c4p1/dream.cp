// DMOPC 2021 C4 P1: axis-aligned right triangles (corner first, then the ends of its two legs).
// For each query point, count the triangles containing it, boundary included.
let (n, q) = read<(1..=3_000, 1..=3_000)>()
let tris = read<[n](corner: Pt, up: Pt, side: Pt)>()
for p in read<[q]Pt>():
    print(tris.count(t => locate([t.corner, t.up, t.side], p) != Location.Outside))
