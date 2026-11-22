// expect: ok, output "YES"
use std.bag.{SortedBag}
print(yes(SortedBag.of([2, 2, 5]).sum() == 9))
