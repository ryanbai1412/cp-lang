// Melds in a fresh process: a heap.cp run reuses freed nodes at n but not at 4n.
use std.heap.{MeldHeap}

let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
var s = 0
for _ in 0..16:
    var mk = MeldHeap<Int>.of(xs)
    var other = MeldHeap<Int>.of(xs)
    mk.meld(other) // timed: heap.MeldHeap.meld
    var more = MeldHeap<Int>.of(xs)
    mk.absorb(mut more) // timed: heap.MeldHeap.absorb
    s += mk.len
print(s)
