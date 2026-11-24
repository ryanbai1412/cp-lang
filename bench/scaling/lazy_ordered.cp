// sizes stay at n: each insert is paired with an erase
use std.lazy.{LazyTreap}
let n: Int = read()
let reps = 1 << 17
var s = 0

var o = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f, before: (x, y) => x < y)
for i in 0..n: o.insert(i * 7919 % n)
for i in 0..reps:
    let p = i * 7919 % n
    o.insert(p) // timed: lazy.LazyTreap.insert
    s += o.erase_at(i * 17 % n)
print(s)
