// expect: ok, output "-2=5, 7=70"
use std.hashmap.{FlatMap}
var m = FlatMap(0)
m.set(7, 70)
m.set(-2, 5)
print(m)
