// expect: ok, output "4 5"
use std.deque.{LinkedList}
var l: LinkedList<Int> = LinkedList()
l.push(4)
l.push(5)
print(l)
