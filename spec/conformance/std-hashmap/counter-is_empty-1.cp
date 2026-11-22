// expect: ok, output "YES"
use std.hashmap.{Counter}
print(Counter.of([1]).is_empty == false)
