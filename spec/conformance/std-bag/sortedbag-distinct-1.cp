// expect: ok, output "YES"
use std.bag.{SortedBag}
print(SortedBag.of([3, 1, 3]).distinct == 2)
