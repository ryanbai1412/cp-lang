// expect: R0401
use std.tree.{from_prufer}
domain V = 0..4
print(from_prufer<V>([1]).len) // <- here
