// expect: ok, output "YES"
use std.hashmap.{FlatMap}
var m = FlatMap(0)
m.set(5, 50)
print(yes(m.get(5) == Some(50) and m.get(6) == None))
