// expect: ok, output "0"
use std.tree.{prufer}
domain V = 0..2
let tr: Tree<V> = [(u: 0, v: 1)]
print(prufer(tr).len)
