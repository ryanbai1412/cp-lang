// expect: ok, output "YES"
use std.hashmap.{FlatMap}
var a = FlatMap(0)
a.set(1, 10)
var b = FlatMap(0)
b.set(1, 5)
a.absorb(mut b)
print(yes(a.get(1) == Some(5)))
