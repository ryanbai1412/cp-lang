// expect: R0401
// rule: std.heap.Meld
// A handle copied before a pop names a node the pool may hand to a later push; using it still
// traps instead of reading the element that took the node.
use std.heap.{Meld, MeldHeaps}
var p = MeldHeaps<Int>()
var h = p.of([1])
let old = h
p.pop(mut h)
p.push(mut h, 5)
print(p.top(old)!) // <- here
