// expect: R0401 (checked)
use std.bag.{SortedBag}
var b = SortedBag.of([1, 2, 3])
let c = b.cursor_last()!
let _ = b.remove(1)
print(yes(b.next_cursor(c) == None))  // <- here
