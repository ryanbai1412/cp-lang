// expect: ok, output "1 2 4 -1"
use std.bag.{SortedSet}
let s = SortedSet.of([9, 1, 4])
print(s.rank(4), s.rank(5), s.nth(1)!, s.nth(3) ?? -1)
