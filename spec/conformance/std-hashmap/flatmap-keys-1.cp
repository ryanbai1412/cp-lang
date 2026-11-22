// expect: ok, output "YES"
use std.hashmap.{FlatMap}
var m = FlatMap(0)
m.set(2, 20)
m.set(1, 10)
print(yes(m.keys.sorted() == [1, 2]))
