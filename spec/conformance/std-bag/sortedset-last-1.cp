// expect: ok, output "YES"
use std.bag.{SortedSet}
print(yes(SortedSet.of([4, 1, 9]).last == Some(9)))
