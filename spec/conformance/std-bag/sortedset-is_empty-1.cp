// expect: ok, output "YES"
use std.bag.{SortedSet}
print(yes(SortedSet.of([1]).is_empty == false))
