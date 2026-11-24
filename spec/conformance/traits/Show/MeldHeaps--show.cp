// expect: ok, output "4 nodes"
use std.heap.{MeldHeaps}
var p = MeldHeaps<Int>()
p.of([5, 1, 7])
p.of([3])
print(p)
