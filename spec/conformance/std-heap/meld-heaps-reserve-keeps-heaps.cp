// expect: ok, output "1\n3\n1 2 3 5 8\n0"
// rule: std.heap.MeldHeaps.reserve
// `reserve` only preallocates pool nodes: heaps built before it keep their items and handles.
use std.heap.{Meld, MeldHeaps}
var p = MeldHeaps<Int>()
var a = p.of([5, 1])
p.reserve(0)
print(p.top(a)!)
p.reserve(64)
var b = p.of([3, 8])
print(p.top(b)!)
p.push(mut a, 2)
var all = p.meld(a, b)
print(p.drain(mut all))
print(all.len)
