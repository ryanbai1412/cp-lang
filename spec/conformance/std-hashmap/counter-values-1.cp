// expect: ok, output "YES"
use std.hashmap.{Counter}
print(Counter.of([2, 1, 2]).values.sorted() == [1, 2])
