// sizes stay at n: each insert is paired with an erase
use std.lazy.{LazyTreap}
let n: Int = read()
let reps = 1 << 17
var s = 0

// The sorted treap holds n / 8 values, so its nodes stay in L2 at 4n too and the inserts' cache
// misses don't grow with n.
let on = n / 8
var o = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f, before: (x, y) => x < y)
for i in 0..on: o.insert(i * 7919 % on)
for i in 0..reps:
    let p = i * 7919 % n
    o.insert(p % on) // timed: lazy.LazyTreap.insert
    s += o.erase_at(i * 17 % on)
print(s)
