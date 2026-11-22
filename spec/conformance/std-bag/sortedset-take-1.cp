// expect: ok, output "YES"
use std.bag.{SortedSet}
var s = SortedSet.of([1, 2])
let t = s.take()
print(s.is_empty and t.to_list() == [1, 2])
