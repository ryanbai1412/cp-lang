// expect: ok, output "YES"
use std.bag.{SortedBag}
var b = SortedBag.of([1])
print(b.take_first(x => x >= 3) == None and b.len == 1)
