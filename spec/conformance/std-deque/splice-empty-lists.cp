// expect: ok, output "1 2\n1 2\n0"
use std.deque.{LinkedList}
let empty: List<Int> = []
var a = LinkedList.from_list([1, 2])
a.splice_back(LinkedList.from_list(empty))
print(a.each().collect())
a.splice_front(LinkedList.from_list(empty))
print(a.each().collect())
var e = LinkedList.from_list(empty)
e.splice_back(LinkedList.from_list(empty))
print(e.each().collect().len)
