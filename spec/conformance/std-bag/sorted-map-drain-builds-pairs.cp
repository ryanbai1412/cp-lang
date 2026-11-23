// expect: ok, output "4 40\n6 60\n1 10\n0"
// `SortedMap.drain` returns the removed (key, value) pairs; it used to stop with an internal error once a pair was removed.
use std.bag.*
var m: SortedMap<Int, Int> = SortedMap()
m.set(1, 10)
m.set(4, 40)
m.set(6, 60)
print(m.drain(after: 1, while: (k, v) => v < 50))
print(m.drain(before: 9, while: (k, v) => k > 5))
print(m.drain(while: (k, v) => true))
print(m.len)
