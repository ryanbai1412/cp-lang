// expect: ok, output "YES"
use std.hashmap.{FlatMap}
var m: FlatMap<Int> = FlatMap(0)
print(m.len == 0)
