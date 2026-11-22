// expect: ok, output "YES YES"
use std.bag.{SortedBag}
let b = SortedBag.of([2, 5])
print(yes(b.cursor_at(5) != None), yes(b.cursor_at(3) == None))
