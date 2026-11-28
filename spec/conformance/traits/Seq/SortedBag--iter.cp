// expect: ok, output "1\n3\n3"
// engines: next
use std.bag.{SortedBag}
let b = SortedBag.of([3, 1, 3])
for x in b: print(x)
