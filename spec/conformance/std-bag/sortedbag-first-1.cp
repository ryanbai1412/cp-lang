// expect: ok, output "YES"
use std.bag.{SortedBag}
print(yes(SortedBag.of([3, 1, 1]).first == Some(1)))
