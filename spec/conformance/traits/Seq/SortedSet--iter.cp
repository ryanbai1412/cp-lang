// expect: ok, output "2\n5"
// engines: next
use std.bag.{SortedSet}
let s = SortedSet.of([5, 2, 5])
for x in s: print(x)
