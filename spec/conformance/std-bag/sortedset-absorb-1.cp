// expect: ok, output "YES"
use std.bag.{SortedSet}
var a = SortedSet.of([1, 3])
var b = SortedSet.of([3, 2])
a.absorb(mut b)
print(a.to_list() == [1, 2, 3] and b.is_empty)
