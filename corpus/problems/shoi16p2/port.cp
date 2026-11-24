// Source: SHOI 2016 / JLOI 2016, Square, https://dmoj.ca/problem/shoi16p2
// Problem: an (N+1) x (M+1) lattice of points (N, M <= 1e6) with K <= 2000 points deleted.
//   Count the squares, axis-parallel or tilted, whose four corners are all kept points, mod
//   100000007.
// Solution: inclusion-exclusion. Each square sits in exactly one i x i axis box, which holds i
//   squares: sum (N-i+1)(M-i+1) i. Subtract the squares through each deleted point, a closed
//   form per side of the point. A square with c >= 2 deleted corners was taken away c times and
//   is met once per pair of them (as a side, both ways, or as a diagonal), so each meeting adds
//   back 2/c mod the prime, (c - 1) in all. O(min(N, M) + K^2).
// original: corpus/submissions/5445692.cpp
// Bounds: n, m: 1..=1e6; k: 0..=2000
type Z = Mod<100_000_007>

let (n, m, k): (1..=1e6, 1..=1e6, 0..=2000) = read()
domain X = 0..=n
domain Y = 0..=m
domain Gone = 0..k
type Point = (X, Y)
let gone: [Gone](x: 0..=n, y: 0..=m) = read()

fn tri(x: Int) -> Int = x * (x + 1) / 2

// squares with a corner at a point that has `along` room one way along an edge, `back` room
// the other way and `out` room away from the edge
fn corner_squares(along: Int, back: Int, out: Int) -> Int:
    let c = out - 1
    let (a, b) = (min(along, c), min(back - 1, c))
    min(a + 1, c - b) * (b + 1) + (if c - a <= b: tri(b + 1) - tri(c - a) else: 0)

fn point(x: Int, y: Int) -> ?Point:
    if let px = X.at(x), let py = Y.at(y): Some((px, py)) else: None

// every square, axis-parallel or tilted, sits in exactly one i x i box, which holds i of them
var total = sum(Z((n - i + 1) * (m - i + 1) * i) for i in 1..=min(n, m))

// minus the squares through each deleted point
for (x, y) in gone:
    total -= corner_squares(x, n - x, y) + corner_squares(m - y, y, x)
        + corner_squares(n - x, x, m - y) + corner_squares(y, m - y, n - x)

// plus a share of each square met at a pair of deleted corners
let deleted = Set([point(x, y)! for (x, y) in gone])
let share = [if c < 2: Z(0) else: Z(2) / Z(c) for c in 0..=4]

// the square with corner (x, y) and side (dx, dy), turning left
fn meet(x: Int, y: Int, dx: Int, dy: Int) -> Z:
    let corners = [point(x, y), point(x + dx, y + dy), point(x + dx - dy, y + dx + dy), point(x - dy, y + dx)]
    if corners.contains(None): Z(0) else: share[corners.count(c => c! in deleted)]

for i in Gone:
    for j in Gone:
        if i == j: continue
        let (p, q) = (gone[i], gone[j])
        total += meet(p.x, p.y, q.x - p.x, q.y - p.y)
        let (ex, ey) = (q.x + q.y - p.x - p.y, q.y - q.x + p.x - p.y)
        if i < j and ex % 2 == 0: total += meet(p.x, p.y, ex / 2, ey / 2)
print(total)
