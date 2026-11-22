// expect: ok, output "3 3 5\n1 7"
// rule: 08.1.1
use std.bag.{SortedBag}
var b = SortedBag.of([1, 3, 3, 5, 7])
print(b.drain_range(3..6).collect())
print(b.to_list())
