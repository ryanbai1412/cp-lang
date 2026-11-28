// expect: ok, output "YES NO"
// engines: next
use std.bag.{SortedSet}
let s = SortedSet.of([5, 2, 5])
print(yes(s.contains(5)), yes(s.contains(3)))
