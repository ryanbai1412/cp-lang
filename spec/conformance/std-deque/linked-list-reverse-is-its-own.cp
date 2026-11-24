// expect: ok, output "3 2 1"
// A type's own `reverse` is what `a.reverse()` calls, not the in-place
// `reverse` of an `IndexedMut` sequence.
use std.deque.{LinkedList}
var a = LinkedList<Int>.new()
a.push(1)
a.push(2)
a.push(3)
a.reverse()
print(a.each().collect())
