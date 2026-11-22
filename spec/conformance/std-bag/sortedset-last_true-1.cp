// expect: ok, output "YES"
use std.bag.{SortedSet}
print(yes(SortedSet.of([1, 4, 6]).last_true(x => x <= 5) == Some(4)))
