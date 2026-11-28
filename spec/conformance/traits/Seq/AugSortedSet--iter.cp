// expect: ok, output "1\n3"
// engines: next
use std.bag.{AugSortedSet}
var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k)
for x in [3, 1, 3]: s.add(x)
for x in s: print(x)
