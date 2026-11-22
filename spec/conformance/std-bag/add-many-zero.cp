// expect: ok, output "1 0"
use std.bag.{SortedBag}
var b = SortedBag.of([1])
b.add_many(5, 0)
print(b.len, b.count(5))
