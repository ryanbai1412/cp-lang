// A pool at steady state: every push takes the node the pop before it freed.
use std.heap.{Meld, MeldHeaps}

let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
var p = MeldHeaps<Int>()
var h = p.of(xs)
var s = 0
for i in 0..8 * n:
    s += p.pop(mut h)! // timed: heap.MeldHeaps.pop
    p.push(mut h, xs[i % n] + i) // timed: heap.MeldHeaps.push
print(s)
