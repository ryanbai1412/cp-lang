// expect: ok, output "1 2\n1 2\n0\n0 0"
use std.deque.{LinkedList}
let empty: List<Int> = []
var a = LinkedList.from_list([1, 2])
var b = LinkedList.from_list(empty)
a.splice_back(mut b)
print(a.each().collect())
a.splice_front(mut b)
print(a.each().collect())
var e = LinkedList.from_list(empty)
var f = LinkedList.from_list(empty)
e.splice_back(mut f)
print(e.each().collect().len)
print(f.len, b.len)
