// expect: ok, output "YES"
use std.bag.{SortedBag}
print(yes(SortedBag.of([3, 1, 3]).last == Some(3)))
