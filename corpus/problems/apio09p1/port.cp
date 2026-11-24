// Source: APIO 2009 Digging for Oil (采油区域), https://dmoj.ca/problem/apio09p1
// Problem: an n x m grid (n, m <= 1500) of values 0..=500 and a size k; print the largest total
//   of three non-overlapping k x k squares (three always fit).
// Solution: the three squares always split by a straight cut into one and two, and the two by
//   another cut. With 2D prefix sums, for each of four flips/transposes of the grid sweep a
//   horizontal cut i: down[i] is the best square below it, and above it take two squares split
//   either horizontally (best in the row band + up) or vertically (prefix and suffix column
//   maxima). O(nm) per orientation.
// original: corpus/submissions/5536569.cpp
// Bounds: n, m, k: 1..=1500
let n: 1..=1500 = read()
let m: 1..=1500 = read()
let k: 1..=1500 = read()
domain Row = 0..n
domain Col = 0..m
let g: [Row, Col]Int = read()

fn best_oriented(r: Int, c: Int, flat: List<Int>) -> Int:
    var psum: List<Int> = List.filled((r + 1) * (c + 1), 0)
    let w = c + 1
    for i in 0..r:
        for j in 0..c:
            psum[(i + 1) * w + (j + 1)] = flat[i * c + j] + psum[(i + 1) * w + j] + psum[i * w + (j + 1)] - psum[i * w + j]

    fn sq(x: Int, y: Int) -> Int:
        psum[(x + k) * w + (y + k)] - psum[x * w + (y + k)] - psum[(x + k) * w + y] + psum[x * w + y]

    var down: List<Int> = []
    var up: List<Int> = []
    for _ in 0..=r:
        down.push(0)
        up.push(0)
    for i in (0..=r - k).rev:
        var v = down[i + 1]
        for j in 0..=c - k: v max= sq(i, j)
        down[i] = v
    for i in k..=r:
        var v = up[i - 1]
        for j in 0..=c - k: v max= sq(i - k, j)
        up[i] = v

    var pref: List<Int> = []
    var suff: List<Int> = []
    for _ in 0..=c:
        pref.push(0)
        suff.push(0)
    var cup = 0
    var ans = 0
    for i in k..r:
        var row_max = 0
        for j in 0..=c - k:
            let s = sq(i - k, j)
            pref[j + k] max= s
            suff[j] max= s
            row_max max= s
        for j in 0..c: pref[j + 1] max= pref[j]
        for j in (0..c).rev: suff[j] max= suff[j + 1]
        if i - k > 0: cup max= row_max + up[i - k]
        for j in 0..=c: cup max= pref[j] + suff[j]
        ans max= cup + down[i]
    ans

var g0: List<Int> = []
var g1: List<Int> = []
var g2: List<Int> = []
var g3: List<Int> = []
for i in Row:
    for j in Col:
        g0.push(g[i, j])
        g1.push(g[Row(n - 1 - i.idx), j])
for j in Col:
    for i in Row:
        g2.push(g[i, j])
        g3.push(g[i, Col(m - 1 - j.idx)])

var ans = best_oriented(n, m, g0)
ans max= best_oriented(n, m, g1)
ans max= best_oriented(m, n, g2)
ans max= best_oriented(m, n, g3)
print(ans)
