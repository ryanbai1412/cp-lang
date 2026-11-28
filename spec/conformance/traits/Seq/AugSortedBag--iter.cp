// expect: ok, output "1\n3\n3"
// engines: next
use std.bag.{AugSortedBag}
var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k)
for x in [3, 1, 3]: b.add(x)
for x in b: print(x)
