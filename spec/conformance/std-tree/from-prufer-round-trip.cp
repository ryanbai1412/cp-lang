// expect: ok, output "YES YES"
use std.tree.{prufer, from_prufer}
domain V = 0..4
let tr = from_prufer<V>([1, 1])
print(yes(tr == [(u: 0, v: 1), (u: 2, v: 1), (u: 1, v: 3)]), yes(prufer(tr) == [1, 1]))
