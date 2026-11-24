// Source: SHOI 2016 / JLOI 2016, Square, https://dmoj.ca/problem/shoi16p2
// limits: 1000 ms, 125 MB
// Problem: an (N+1) x (M+1) lattice of points (N, M <= 1e6) with K <= 2000 points deleted.
//   Count the squares, axis-parallel or tilted, whose four corners are all kept points, mod
//   100000007.
// Solution: inclusion-exclusion. Each square sits in exactly one i x i axis box, which holds i
//   squares: sum (N-i+1)(M-i+1) i. Subtract the squares through each deleted point, a closed
//   form per side of the point. A square with c >= 2 deleted corners was taken away c times:
//   walk each pair of deleted points (as a side, both ways, or as a diagonal) to the squares
//   having both as corners, and add back c - 1 at the pair of its two least deleted corners,
//   looked up in a hash of the deleted points. O(min(N, M) + K^2).
// original: corpus/submissions/5445692.cpp
// Bounds: n, m: 1..=1e6; k: 0..=2000
const MOD = 100000007
let n: 1..=1e6 = read()
let m: 1..=1e6 = read()
let k: 0..=2000 = read()
let pts: [0..k](x: Int, y: Int) = read()

fn tri(x: Int) -> Int: x * (x + 1) / 2

// squares with a corner at a point that has `a` room one way along an edge,
// `b` room the other way and `c` room away from the edge
fn corner_squares(a0: Int, b0: Int, c0: Int) -> Int:
    let c = c0 - 1
    let a = min(a0, c)
    let b = min(b0 - 1, c)
    var ways = min(a + 1, c - b) * (b + 1)
    if c - a + 1 <= b + 1: ways += tri(b + 1) - tri(c - a)
    ways

// every square, axis-parallel or tilted, sits in exactly one i x i box
var total = 0
for i in 1..=min(n, m):
    total = (total + (n - i + 1) * (m - i + 1) % MOD * i) % MOD

// minus the squares through each deleted point
for i in 0..k:
    let (xl, xr, yl, yr) = (pts[i].x, n - pts[i].x, pts[i].y, m - pts[i].y)
    total -= corner_squares(xl, xr, yl) + corner_squares(yr, yl, xl) + corner_squares(
        xr,
        xl,
        yr,
    ) + corner_squares(
        yl,
        yr,
        xr,
    )

// plus what that double-counted, once per square, at its first pair of deleted corners
var deleted = HashMap<Int, Int>()
for i in 0..k: deleted[pts[i].x * (m + 1) + pts[i].y] = i
fn square(i: Int, j: Int, dx: Int, dy: Int):
    let (x, y) = (pts[i].x, pts[i].y)
    var (first, second, count) = (k, k, 0)
    for (cx, cy) in [(x, y), (x + dx, y + dy), (x + dx - dy, y + dx + dy), (x - dy, y + dx)]:
        if cx < 0 or cx > n or cy < 0 or cy > m: return
        if let d = deleted.get(cx * (m + 1) + cy):
            count += 1
            if d < first: (first, second) = (d, first)
            elif d < second: second = d
    if (first, second) == (min(i, j), max(i, j)): total += count - 1

for i in 0..k:
    for j in 0..k:
        if i != j:
            if i < j:
                let ex = pts[j].x + pts[j].y - pts[i].x - pts[i].y
                let ey = pts[j].y - pts[j].x + pts[i].x - pts[i].y
                if ex % 2 == 0: square(i, j, ex / 2, ey / 2)
            square(i, j, pts[j].x - pts[i].x, pts[j].y - pts[i].y)
print((total % MOD + MOD) % MOD)
