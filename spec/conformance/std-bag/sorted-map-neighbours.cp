// expect: ok, output "3 3 1 -1"
use std.bag.{SortedMap}
var m: SortedMap<Int, Int> = SortedMap()
m.set(1, 10)
m.set(3, 30)
let (a, _) = m.first_ge(2)!
let (b, _) = m.first_gt(1)!
let (c, _) = m.last_le(2)!
print(a, b, c, m.last_lt(1).map(((k, _)) => k) ?? -1)
