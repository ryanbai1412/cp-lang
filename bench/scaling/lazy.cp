// sizes stay at n: each insert is paired with an erase
use std.lazy.{LazyTreap}
let n: Int = read()
let reps = 1 << 17
var s = 0

fn treap() -> LazyTreap<Int, Int> = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f)
for i in 0..reps:
    var e = treap() // timed: lazy.LazyTreap.new
    if e.is_empty: s += i % 2
var t = treap()
for i in 0..n: t.push(i * 7919 % n)
for i in 0..reps:
    let p = i * 7919 % n
    s += t.len // timed: lazy.LazyTreap.len
    if t.is_empty: s += 1 // timed: lazy.LazyTreap.is_empty
    t.apply_all(1) // timed: lazy.LazyTreap.apply_all
    t.apply_range(p / 2, p, 1) // timed: lazy.LazyTreap.apply_range
    t.apply(p / 2..p, -1) // timed: lazy.LazyTreap.apply
    t.reverse(p / 3..p) // timed: lazy.LazyTreap.reverse
    s += t.get(i * 13 % n) // timed: lazy.LazyTreap.get
print(s)
