// expect: ok, output "3\n1\n4"
use std.deque.{LinkedList}
let l = LinkedList.from_list([3, 1, 4])
for x in l: print(x)
