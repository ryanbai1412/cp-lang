// expect: ok, output "YES"
use std.hashmap.{Counter}
print(Counter.of([2, 1, 2]).items.sorted() == [(1, 1), (2, 2)])
