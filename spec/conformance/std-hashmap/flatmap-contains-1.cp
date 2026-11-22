// expect: ok, output "YES"
use std.hashmap.{FlatMap}
var m = FlatMap(0)
m.set(-3, 1)
print(yes(m.contains(-3) and not m.contains(3)))
