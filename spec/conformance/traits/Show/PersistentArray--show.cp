// expect: ok, output "7 7 7"
use std.persistent.{PersistentArray}
var p = PersistentArray<0..3, Int>(fill: 7)
print(p)
