// A PMultiset over 0..n holding n values, read through its PArena.
use std.persistent.{PArena, PMultiset}
let n: Int = read()
domain V = 0..n
var ar = PArena<PMultiset<V>>()
var ms = PMultiset<V>.empty()
for i in 0..n: ms = ms.with(mut ar, V(i * 7919 % n))
var s = 0
for i in 0..64 * n:
    let w = ms.within(ar) // timed: persistent.PMultiset.within
    s += w.len // timed: persistent.PMultisetWithin.len
    if w.is_empty: s += 1 // timed: persistent.PMultisetWithin.is_empty
for r in 0..16:
    s += ms.within(ar).each().count() // timed: persistent.PMultisetWithin.each
print(s)
