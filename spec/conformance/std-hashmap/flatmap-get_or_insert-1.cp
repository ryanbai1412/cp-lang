// expect: ok, output "YES"
use std.hashmap.{FlatMap}
var m = FlatMap(0)
print(m.get_or_insert(4, 7) == 7 and m.get_or_insert(4, 9) == 7)
