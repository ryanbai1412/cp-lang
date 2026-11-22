// expect: ok, output "YES"
use std.hashmap.{FlatMap}
var m = FlatMap(0)
m.set(1, 5)
m.set(1, 6)
print(m.get(1) == Some(6))
