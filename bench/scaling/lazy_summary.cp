use std.lazy.{LazyTreap}
use std.algebra.{Action}
let n: Int = read()
let reps = 1 << 17
var s = 0

// A sorted treap that keeps each subtree's sum, at n / 8 elements so its nodes and sums stay in L2
// at 4n too and the walk's cache misses don't grow with it.
let few = n / 8
var w = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f, before: (x, y) => x > y, summary: Action<Int, Int>(apply: (d, x, m) => x + d * m))
for i in 0..few: w.insert(i * 7919 % few)
for i in 0..reps:
    s += w.prefix(i * 13 % few) // timed: lazy.LazyTreap.prefix
    let v = i * 7919 % few
    s += w.count_while(x => x >= v) // timed: lazy.LazyTreap.count_while
print(s)
