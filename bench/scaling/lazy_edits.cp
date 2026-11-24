// sizes stay at n: each insert is paired with an erase; at n = 4096 the treap's nodes stay in L2 at 4n
// too, so the walks' cache misses don't grow with n
// n: 4096
use std.lazy.{LazyTreap}
let n: Int = read()
let reps = 1 << 17
var s = 0

fn treap() -> LazyTreap<Int, Int> = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f)
var t = treap()
for i in 0..n: t.push(i * 7919 % n)
for i in 0..reps:
    let p = i * 7919 % n
    t.insert_at(p, i) // timed: lazy.LazyTreap.insert_at
    s += t.erase_at(i * 17 % n) // timed: lazy.LazyTreap.erase_at
    t.push(i) // timed: lazy.LazyTreap.push
    t.erase_range(p..p + 1) // timed: lazy.LazyTreap.erase_range
    var u = t.split_at(t.len - 1 - i % 8) // timed: lazy.LazyTreap.split_at [let: k = n - 4]
    t.splice_back(mut u) // timed: lazy.LazyTreap.splice_back [fixed: m]
    s += t.take(4).len // timed: lazy.LazyTreap.take [fixed: k]
// At n / 8 elements the list stays under 128 KB at 4n too, so malloc serves both sizes from the
// heap and neither pays fresh-page faults that the other doesn't.
var tl = treap()
for i in 0..n / 8: tl.push(i * 7919 % n)
for _ in 0..6400:
    s += tl.to_list().len // timed: lazy.LazyTreap.to_list
// Storage grows by moving the nodes already there. At n / 16 nodes the doubled block stays
// under 128 KB at 4n too, so malloc serves both sizes from the heap and neither pays fresh-page
// faults that the other doesn't.
for _ in 0..4000:
    var r = treap()
    for i in 0..n / 16: r.push(i)
    r.reserve(n / 8) // timed: lazy.LazyTreap.reserve
    s += r.len
print(s)
