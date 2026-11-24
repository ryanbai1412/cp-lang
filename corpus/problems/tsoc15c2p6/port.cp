// Source: TSOC '15 Contest 2 #6 (All-Out War!), https://dmoj.ca/problem/tsoc15c2p6
// Problem: n <= 1e6 columns of R_i <= 1e9 enemies and q <= 1e5 attacks (a, b, c): subtract c
//   from every column in [a, b], then print the minimum over [a, b] and over [1, n], where a
//   negative column counts as 0.
// Solution: values only decrease, so clamping the minimum at 0 afterwards is exact. A lazy
//   SegTree with range add and min: per attack one `apply`, one range fold and the whole-tree
//   fold, O(log n).
// original: corpus/submissions/3657393.cpp
// Bounds: n: 1..=1e6; q: 1..=1e5; enemies: 0..=1e9

let n: 1..=1e6 = read()
let q: 1..=1e5 = read()
domain Column = 1..=n
let enemies: [Column]0..=1e9 = read()

var row = SegTree<Column, Int, Int>(min, act: add, a: enemies)
for _ in 0..q:
    let (l, r): (Column, Column) = read()
    let kills: 0..=1e9 = read()
    row.apply(l..=r, -kills)
    print(max(row.fold(l..=r), 0), max(row.all, 0))
