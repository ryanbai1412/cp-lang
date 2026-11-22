// expect: ok, output "YES"
use std.hashmap.{FlatMap}
var m = FlatMap(0)
m.set(2, 20)
m.set(1, 10)
print(yes(m.items.sorted() == [(1, 10), (2, 20)]))
