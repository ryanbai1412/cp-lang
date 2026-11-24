// Source: APIO 2009 Digging for Oil (采油区域), https://dmoj.ca/problem/apio09p1
// Problem: an n x m grid (n, m <= 1500) of values 0..=500 and a size k; print the largest total
//   of three non-overlapping k x k squares (three always fit).
// Solution: three equal squares always split by a straight cut into one and two, and the two by
//   another cut. From 2-D prefix sums take every square's total, then the best square in each of
//   the four corners left by a row cut i and a column cut j; the six layouts (three bands either
//   way, or one side whole and the other split) each read three corners. O(nm).
// original: corpus/submissions/5536569.cpp
// Bounds: n, m: 1..=1500; k: 1..=min(n, m); oil: 0..=500
let (n, m): (1..=1500, 1..=1500) = read()
let k: 1..=min(n, m) = read()
domain Row = 0..n
domain Col = 0..m
let oil: [Row, Col]0..=500 = read()
type Haul = 0..=500 * 1500 * 1500

let sums = Prefix(oil)
// the total of the square with top-left cell (x, y), and 0 where none fits: no square is negative
fn square(x: Int, y: Int) -> Haul:
    if x < 0 or y < 0 or x > n - k or y > m - k:
        return 0
    sums.fold(Row(x)..=Row(x + k - 1), Col(y)..=Col(y + k - 1))

domain RowCut = 0..=n
domain ColCut = 0..=m
// ul[i, j] is the best square in rows ..i and columns ..j; ur, dl and dr the other corners.
var ul: [RowCut, ColCut]Haul = 0
var ur: [RowCut, ColCut]Haul = 0
var dl: [RowCut, ColCut]Haul = 0
var dr: [RowCut, ColCut]Haul = 0
for i in RowCut:
    for j in ColCut:
        let p = (i, j)
        ul[p] = max(ul[p.up?] ?? 0, ul[p.left?] ?? 0, square(Int(i) - k, Int(j) - k))
    for j in ColCut.rev:
        let p = (i, j)
        ur[p] = max(ur[p.up?] ?? 0, ur[p.right?] ?? 0, square(Int(i) - k, Int(j)))
for i in RowCut.rev:
    for j in ColCut:
        let p = (i, j)
        dl[p] = max(dl[p.down?] ?? 0, dl[p.left?] ?? 0, square(Int(i), Int(j) - k))
    for j in ColCut.rev:
        let p = (i, j)
        dr[p] = max(dr[p.down?] ?? 0, dr[p.right?] ?? 0, square(Int(i), Int(j)))

let (top, bottom) = (RowCut.first!, RowCut.last!)
let (west, east) = (ColCut.first!, ColCut.last!)
var best = 0
for i in RowCut if Int(i) >= k and Int(i) <= n - k:
    for j in ColCut if Int(j) >= k and Int(j) <= m - k:
        best max= ul[i, j] + ur[i, j] + dr[i, west]
        best max= ul[i, east] + dl[i, j] + dr[i, j]
        best max= ul[i, j] + dl[i, j] + dr[top, j]
        best max= ul[bottom, j] + ur[i, j] + dr[i, j]
for x in k..=n - 2 * k:
    best max= ul[RowCut(x), east] + max(square(x, y) for y in 0..=m - k)! + dr[RowCut(x + k), west]
for y in k..=m - 2 * k:
    best max= ul[bottom, ColCut(y)] + max(square(x, y) for x in 0..=n - k)! + dr[top, ColCut(y + k)]
print(best)
