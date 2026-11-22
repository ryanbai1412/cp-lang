// expect: ok, output "YES"
use std.hashmap.{FlatMap}
var m: FlatMap<Int> = FlatMap(0)
print(yes(m.len == 0))
