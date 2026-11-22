// expect: ok, output "YES"
use std.hashmap.{Counter}
let c = Counter.of([5])
print(yes(c.contains(5) and not c.contains(6)))
