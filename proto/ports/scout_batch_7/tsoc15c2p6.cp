// TSOC '15 Contest 2 #6: n soldiers in a row with health values, and q
// volleys. Each volley takes x from every soldier in l..=r; after it, print
// the lowest health in l..=r and the lowest in the whole row, where a
// soldier below 0 counts as 0.
//
// Range add, range min: a lazy tree and two folds per volley.
use std.lazy.{LazySegTree, AddMin}

let (n, q): (Int, Int) = read()
domain Soldier = 1..=n
let health: [Soldier]Int = read()

var row = LazySegTree<Soldier, Int, Int>(AddMin, health)
for _ in 0..q:
    let (l, r, x): (Soldier, Soldier, Int) = read()
    row.apply(l..=r, -x)
    print(max(row.fold(l..=r), 0), max(row.all, 0))
