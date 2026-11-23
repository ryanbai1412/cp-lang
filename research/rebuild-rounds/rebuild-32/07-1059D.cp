// cf1059d Nature Reserve: with every lair on one side (flip to y > 0), a circle tangent
// to y = 0 at x0 must have radius max((xi - x0)^2 + yi^2) / (2 yi), convex in x0: ternary search.
let n: 1..=1e5 = read()
let p = read<[n](x: -1e7..=1e7, y: -1e7..=1e7)>()

let above = p.count(q => q.y > 0)
if above != 0 and above != n:
    print(-1)
else:
    let radius = (x0: F64) => p.map(q =>
        ((F64(q.x) - x0) * (F64(q.x) - x0) + F64(q.y) * F64(q.y)) / (2.0 * F64(abs(q.y)))
    ).max()!
    var lo: F64 = F64(-1e7)
    var hi: F64 = F64(1e7)
    var best: F64 = F64(1e18)
    for _ in 0..200:
        let m1 = lo + (hi - lo) / 3.0
        let m2 = hi - (hi - lo) / 3.0
        let f1 = radius(m1)
        let f2 = radius(m2)
        best min= f1
        best min= f2
        if f1 < f2: hi = m2
        else: lo = m1
    print(best, precision: 9)
