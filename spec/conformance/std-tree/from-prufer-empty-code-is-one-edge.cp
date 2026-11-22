// expect: ok, output "YES"
use std.tree.{from_prufer}
domain V = 0..2
print(yes(from_prufer<V>([]) == [(u: 0, v: 1)]))
