// expect: ok, output "1 9 4 1"
use std.bag.{SortedSet}
var s = SortedSet.of([4, 1, 9])
let a = s.pop_first()!
let b = s.pop_last()!
print(a, b, s.first!, s.len)
