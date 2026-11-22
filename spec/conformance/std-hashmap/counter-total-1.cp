// expect: ok, output "YES"
use std.hashmap.{Counter}
print(yes(Counter.of([1, 1, 2]).total() == 3))
