// expect: ok, output "-1 -1"
use std.bag.{SortedSet}
var s: SortedSet<Int> = SortedSet()
print(s.pop_first() ?? -1, s.pop_last() ?? -1)
