// expect: R0401
// rule: std.heap.Meld
// A `Meld` handle copied before its heap changed no longer names that heap, so using it traps.
use std.heap.{Meld, MeldHeaps}
var p = MeldHeaps<Int>()
var h = p.of([3, 1])
let old = h
p.push(mut h, 2)
print(p.top(old)!) // <- here
