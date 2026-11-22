// expect: ok, output "YES"
use std.hashmap.{FlatMap}
var m = FlatMap(0)
m.set(2, 9)
print(yes(m.remove(2) == Some(9) and m.remove(2) == None))
