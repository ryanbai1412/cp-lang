// expect: ok, output "1 2\n1 2\n0"
use std.deque.{LinkedList}
let none: List<Int> = []
var a = LinkedList.from_list([1, 2])
a.splice_back(LinkedList.from_list(none))
print(a.each().collect())
a.splice_front(LinkedList.from_list(none))
print(a.each().collect())
var e = LinkedList.from_list(none)
e.splice_back(LinkedList.from_list(none))
print(e.each().collect().len)
