// expect: ok, output "YES"
use std.bag.{SortedBag}
print(yes(SortedBag.of([1]).is_empty == false))
