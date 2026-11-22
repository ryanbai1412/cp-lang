// expect: ok, output "1\n3 3 5"
use std.bag.{SortedBag}
var b = SortedBag.of([1, 3, 3, 5])
let hi = b.split_at(3)
print(b.to_list())
print(hi.to_list())
