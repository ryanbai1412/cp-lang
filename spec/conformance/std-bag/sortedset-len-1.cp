// expect: ok, output "YES"
use std.bag.{SortedSet}
print(yes(SortedSet.of([3, 1, 3]).len == 2))
