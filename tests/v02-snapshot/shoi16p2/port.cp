// Source: SHOI 2016 / JLOI 2016, Square, https://dmoj.ca/problem/shoi16p2
// Problem: an (N+1) x (M+1) lattice of points (N, M <= 1e6) with K <= 2000 points deleted.
//   Count the squares, axis-parallel or tilted, whose four corners are all kept points, mod
//   100000007.
// Solution: inclusion-exclusion. Each square sits in exactly one i x i axis box, which holds i
//   squares: sum (N-i+1)(M-i+1) i. Subtract the squares through each deleted point, a closed
//   form per side of the point. For each pair of deleted points list the squares having both
//   as corners (as a side, both ways, or as a diagonal), sort them, and add back 1, 2 or 3 for
//   squares with 2, 3 or 4 deleted corners (1, 3 or 6 pairs). O(min(N, M) + K^2 log K).
// original: corpus/submissions/5445692.cpp
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

// plus what that double-counted: collect each square once per pair of deleted corners
var hit: List<(Int, Int, Int, Int)> = []
fn square(x: Int, y: Int, dx: Int, dy: Int):
    var xs: List<Int> = []
    var ys: List<Int> = []
    xs.push(x)
    ys.push(y)
    xs.push(x + dx)
    ys.push(y + dy)
    xs.push(x + dx - dy)
    ys.push(y + dx + dy)
    xs.push(x - dy)
    ys.push(y + dx)
    var codes: List<Int> = []
    for c in 0..4:
        if xs[c] < 0 or xs[c] > n or ys[c] < 0 or ys[c] > m: return
        codes.push(xs[c] * (m + 1) + ys[c])
    codes.sort()
    hit.push((codes[0], codes[1], codes[2], codes[3]))

for i in 0..k:
    for j in 0..k:
        if i != j:
            if i < j:
                let ex = pts[j].x + pts[j].y - pts[i].x - pts[i].y
                let ey = pts[j].y - pts[j].x + pts[i].x - pts[i].y
                if ex % 2 == 0: square(pts[i].x, pts[i].y, ex / 2, ey / 2)
            square(pts[i].x, pts[i].y, pts[j].x - pts[i].x, pts[j].y - pts[i].y)

hit.sort()
var i = 0
while i < hit.len:
    var j = i
    while j < hit.len and hit[j] == hit[i]: j += 1
    // 1, 3 or 6 pairs of deleted corners: 2, 3 or 4 corners deleted
    let pairs = j - i
    total += if pairs == 1: 1 elif pairs == 3: 2 else: 3
    i = j
print(total % MOD)
