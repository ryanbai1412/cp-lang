// expect: ok, output "YES"
use std.hashmap.{FlatMap}
var m = FlatMap(0)
m.set(1, 2)
let t = m.take()
print(m.is_empty and t.get(1) == Some(2))
