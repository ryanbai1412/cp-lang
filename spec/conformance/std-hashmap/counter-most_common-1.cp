// expect: ok, output "YES"
use std.hashmap.{Counter}
print(yes(Counter.of([3, 1, 3, 2, 1, 3]).most_common(2) == [(3, 3), (1, 2)]))
