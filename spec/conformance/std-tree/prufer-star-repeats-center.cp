// expect: ok, output "YES"
use std.tree.{prufer}
domain V = 0..4
let tr: Tree<V> = [(u: 0, v: 3), (u: 1, v: 3), (u: 2, v: 3)]
print(yes(prufer(tr) == [3, 3]))
