// expect: ok, output "YES"
use std.bag.{SortedBag}
var b = SortedBag.of([3, 1, 1])
print(b.pop_first() == Some(1) and b.count(1) == 1)
