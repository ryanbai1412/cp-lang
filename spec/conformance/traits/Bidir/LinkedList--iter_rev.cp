// expect: ok, output "4\n1\n3"
use std.deque.{LinkedList}
let l = LinkedList.from_list([3, 1, 4])
for x in l.rev: print(x)
