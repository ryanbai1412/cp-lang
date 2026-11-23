// dmopc21c4p1: right triangles with legs along the axes from corner (x1, y1) through (x2, y1)
// and (x1, y3); each query counts the triangles containing the point, borders included.
let (n, q) = read<(1..=3_000, 1..=3_000)>()
let tris = read<[n](a: Pt, b: Pt, c: Pt)>()
for p in read<[q]Pt>():
    print(tris.count(t => locate([t.a, t.b, t.c], p) != Location.Outside))
