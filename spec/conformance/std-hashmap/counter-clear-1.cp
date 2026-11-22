// expect: ok, output "YES"
use std.hashmap.{Counter}
var c = Counter.of([1, 2])
c.clear()
print(c.len == 0 and c[1] == 0)
