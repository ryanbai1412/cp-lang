// expect: ok, output "2 1 3 2"
use std.bag.{SortedBag}
var b = SortedBag.of([3, 1, 3, 5])
print(b.count(3), b.pop_first()!, b.pop_first()!, b.len)
