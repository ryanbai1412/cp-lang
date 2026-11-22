// expect: ok, output "YES"
use std.hashmap.{Counter}
var c = Counter.of([1])
let d = c.take()
print(c.is_empty and d[1] == 1)
