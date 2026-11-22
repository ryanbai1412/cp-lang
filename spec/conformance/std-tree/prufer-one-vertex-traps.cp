// expect: R0401 at 4
use std.tree.{prufer}
domain V = 0..1
let tr: Tree<V> = []
print(prufer(tr).len)
