// SHOI 2016 P2, lattice squares avoiding k deleted points
// original: corpus/submissions/5445692.cpp
const MOD = 100000007
let n: 1..=1e6 = read()
let m: 1..=1e6 = read()
let k: 0..=2000 = read()
domain P = 0..k
let pts: [P]Pt = read()

fn tri(x: Int) -> Int: x * (x + 1) / 2

// squares with a corner at a point that has `a` room one way along an edge,
// `b` room the other way and `c` room away from the edge
fn corner_squares(a: Int, b: Int, c: Int) -> Int:
    let (a, b, c) = (min(a, c - 1), min(b - 1, c - 1), c - 1)
    var ways = min(a + 1, c - b) * (b + 1)
    if c - a <= b: ways += tri(b + 1) - tri(c - a)
    ways

// every square, axis-parallel or tilted, sits in exactly one i x i box
var total = sum((n - i + 1) * (m - i + 1) % MOD * i for i in 1..=min(n, m)) % MOD

// minus the squares through each deleted point
for p in pts:
    let (xl, xr, yl, yr) = (p.x, n - p.x, p.y, m - p.y)
    total -= corner_squares(xl, xr, yl) + corner_squares(yr, yl, xl) + corner_squares(xr, xl, yr) + corner_squares(yl, yr, xr)

// plus what that double-counted: count each square once per pair of deleted corners
var pairs = HashMap<List<Pt>, Int>()
fn square(a: Pt, side: Pt):
    let turn = Pt(-side.y, side.x)
    let sq = [a, a + side, a + side + turn, a + turn]
    if all(p.x in 0..=n and p.y in 0..=m for p in sq): pairs[sq.sorted()] += 1

for i in P:
    for j in P:
        if i == j: continue
        let d = pts[j] - pts[i]
        square(pts[i], d)
        if i < j and (d.x + d.y) % 2 == 0: square(pts[i], Pt((d.x + d.y) / 2, (d.y - d.x) / 2))

// 1, 3 or 6 pairs of deleted corners: 2, 3 or 4 corners deleted
for (_, c) in pairs.items:
    total += match c:
        1: 1
        3: 2
        _: 3
print(total % MOD)
