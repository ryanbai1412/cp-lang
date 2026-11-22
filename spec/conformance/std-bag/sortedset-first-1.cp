// expect: ok, output "YES"
use std.bag.{SortedSet}
print(SortedSet.of([4, 1, 9]).first == Some(1))
