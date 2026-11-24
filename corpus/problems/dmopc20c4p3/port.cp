// Source: DMOPC '20 Contest 4 P3 (Roving Roombas), https://dmoj.ca/problem/dmopc20c4p3
// limits: 2000 ms, 256 MB
// Problem: n <= 2e5 Roombas clean (0, Y_i) to (X_i, Y_i) and m <= 2e5 cords run (X_j, 0) to
//   (X_j, Y_j), coordinates 1..=1e9 (Roomba Y distinct, cord X distinct). Count the
//   (Roomba, cord) pairs that touch, ends included.
// Solution: a cord touches a Roomba when X_j <= X_i and Y_j >= Y_i. Take the Roombas from the
//   highest down; before each, add every cord at least that tall to a Fenwick tree over the
//   compressed cord x, then count the added cords at or left of X_i. O((n + m) log m).
// original: corpus/submissions/3484973.cpp
// Bounds: n, m: 1..=2e5; roombas.x, roombas.y, cords.x, cords.y: 1..=1e9
type Pt = (x: 1..=1e9, y: 1..=1e9)
let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
var roombas: [n]Pt = read()
var cords: [m]Pt = read()

let (X, xs) = compress([c.x for c in cords])
cords.sort(by: c => desc(c.y))
roombas.sort(by: r => desc(r.y))
var standing = Fenwick<X, Int>()
var added = 0
var touching = 0
for r in roombas:
    while added < m and cords[added].y >= r.y:
        standing.add(xs.index_of(cords[added].x), 1)
        added += 1
    if let reach = xs.last_le(r.x): touching += standing.prefix(reach.r)
print(touching)
