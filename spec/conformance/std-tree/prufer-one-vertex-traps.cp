// expect: R0401
use std.tree.{prufer}
domain V = 0..1
let tr: Tree<V> = []
print(prufer(tr).len) // <- here
