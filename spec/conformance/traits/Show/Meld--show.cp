// expect: ok, output "1 element 1 element"
use std.heap.{MeldHeaps}
var p = MeldHeaps<Int>()
var h = p.of([5, 1])
let g = p.of([3])
p.pop(mut h)
print(h, g)
