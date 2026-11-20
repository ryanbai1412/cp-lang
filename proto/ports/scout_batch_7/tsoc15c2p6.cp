// TSOC '15 Contest 2 #6: n soldiers in a row with health values, and q
// volleys. Each volley takes x from every soldier in l..=r; after it, print
// the lowest health in l..=r and the lowest in the whole row, where a
// soldier below 0 counts as 0.
//
// Range add, range min: a lazy tree and two folds per volley.
use std.algebra.{Action, Min}
use std.lazy.{LazySegTree}

let (n, q): (Int, Int) = read()
domain Soldier = 1..=n
let health: [Soldier]Int = read()

let add = Action<Int, Int>(id: 0, compose: (f, g) => f + g, apply: (f, x) => if x == Int.MAX: x else: x + f)
var row = LazySegTree<Soldier, Int, Int>.from(Min<Int>, add, health)
for _ in 0..q:
    let (l, r, x): (Soldier, Soldier, Int) = read()
    row.apply(l..=r, -x)
    print(max(row.fold(l..=r), 0), max(row.fold(Soldier.all), 0))
