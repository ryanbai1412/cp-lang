// expect: ok, output "YES NO"
// engines: next
use std.bag.{SortedBag}
let b = SortedBag.of([3, 1, 3])
print(yes(b.contains(3)), yes(b.contains(2)))
