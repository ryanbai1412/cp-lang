// expect: ok, output "YES"
use std.hashmap.{FlatMap}
var m = FlatMap(0)
m.set(1, 2)
m.reserve(1000)
print(yes(m.get(1) == Some(2)))
