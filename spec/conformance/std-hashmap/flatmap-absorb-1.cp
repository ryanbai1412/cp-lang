// expect: ok, output "YES"
use std.hashmap.{FlatMap}
var a = FlatMap(0)
a.set(1, 10)
var b = FlatMap(0)
b.set(1, 5)
b.set(2, 7)
a.absorb(mut b, join: (x, y) => x + y)
print(yes(a.get(1) == Some(15) and a.get(2) == Some(7) and b.is_empty))
