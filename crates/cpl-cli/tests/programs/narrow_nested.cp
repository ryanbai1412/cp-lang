fn sum_grid_narrow<R, C>(g: [R, C]0..=1_000_000) -> Int:
    var s = 0
    for r in R:
        for c in C:
            s += g[r, c] * g[r, c]
    return s

fn sum_grid_wide<R, C>(g: [R, C]Int) -> Int:
    var s = 0
    for x in g:
        s += x
    return s

fn sum_row_wide<E>(row: [E]Int) -> Int:
    var s = 0
    for x in row:
        s += x
    return s

fn sum_pts_wide<V>(pts: [V](Int, Int)) -> Int:
    var s = 0
    for (x, y) in pts:
        s += x + y
    return s

let n: 1..=10 = read()
let m: 1..=10 = read()
domain R = 0..n
domain C = 0..m

// 2D ranged grid (narrowed)
var g: [R, C]0..=1_000_000 = read()
for r in R:
    for c in C:
        g[r, c] += 1
g[R(0), C(0)] max= 500_000
print(sum_grid_narrow<R, C>(g))

// 2D ranged grid widened via plain [R, C]Int param
var gw: [R, C]0..=1_000_000 = 0
for r in R:
    for c in C:
        gw[r, c] = g[r, c]
print(sum_grid_wide<R, C>(gw))

// Nested [R][C] and [R]List<0..=1_000_000> (narrowed)
var mat: [R][C]0..=1_000_000 = 0
var adj: [R]List<0..=1_000_000> = []
for r in R:
    for c in C:
        mat[r][c] = g[r, c]
        adj[r].push(g[r, c])
    adj[r].sort()
var nest_sum = 0
for row in mat:
    for x in row:
        nest_sum += x * x
for r in R:
    for x in adj[r]:
        nest_sum += x
print(nest_sum)

// Nested array widened when a row flows to plain [C]Int
var mat_w: [R][C]0..=1_000_000 = 0
for r in R:
    for c in C:
        mat_w[r][c] = g[r, c]
print(sum_row_wide<C>(mat_w[R(0)]))

// Tuple elements whose fields are all ranged (narrowed)
var pts: [R](0..=1_000_000, -100..=100) = (0, 0)
var plist: List<(0..=1_000_000, -100..=100)> = []
for r in R:
    let a = g[r, C(0)]
    let b = r.ord * 10 - 15
    pts[r] = (a, b)
    plist.push((a, b))
plist.sort()
var pt_sum = 0
for r in R:
    let (x, y) = pts[r]
    pt_sum += x * x + y
for (x, y) in plist:
    pt_sum += x - y
print(pt_sum)

// Tuple array widened via plain [R](Int, Int) param
var pts_w: [R](0..=1_000_000, -100..=100) = (0, 0)
for r in R:
    pts_w[r] = pts[r]
print(sum_pts_wide<R>(pts_w))
