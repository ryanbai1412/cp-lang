// expect: ok, output "YES"
use std.bag.{SortedBag}
print(SortedBag.of([2, 2, 5]).sum() == 9)
