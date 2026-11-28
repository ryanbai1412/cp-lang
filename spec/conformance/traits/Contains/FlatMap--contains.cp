// expect: ok, output "YES NO"
// engines: next
use std.hashmap.{FlatMap}
var m = FlatMap(0)
m.set(7, 70)
m.set(-2, 5)
print(yes(m.contains(-2)), yes(m.contains(3)))
