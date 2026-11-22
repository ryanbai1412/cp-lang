// expect: ok, output "YES"
use std.hashmap.{Counter}
let c = Counter.of([5])
print(c.contains(5) and not c.contains(6))
