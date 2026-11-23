// Source: TSOC '15 Contest 2 #6 (All-Out War!), https://dmoj.ca/problem/tsoc15c2p6
// Problem: n <= 1e6 columns of R_i <= 1e9 enemies and q <= 1e5 attacks (a, b, c): subtract c
//   from every column in [a, b], then print the minimum over [a, b] and over [1, n], where a
//   negative column counts as 0.
// Solution: values only decrease, so clamping the minimum at 0 afterwards is exact. A lazy
//   SegTree with range add and min: per attack one `apply`, one range fold and the whole-tree
//   fold, O(log n).

let (n, q): (Int, Int) = read()
domain Soldier = 1..=n
let health: [Soldier]Int = read()

var row = SegTree<Soldier, Int, Int>(min, act: add, a: health)
for _ in 0..q:
    let (l, r, x): (Soldier, Soldier, Int) = read()
    row.apply(l..=r, -x)
    print(max(row.fold(l..=r), 0), max(row.all, 0))
