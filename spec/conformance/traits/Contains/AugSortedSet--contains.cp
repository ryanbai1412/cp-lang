// expect: ok, output "YES NO"
// engines: next
use std.bag.{AugSortedSet}
var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k)
for x in [3, 1, 3]: s.add(x)
print(yes(s.contains(3)), yes(s.contains(2)))
