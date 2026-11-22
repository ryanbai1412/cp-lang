// expect: ok, output "YES"
use std.bag.{SortedBag}
print(SortedBag.of([3, 1, 1]).first == Some(1))
