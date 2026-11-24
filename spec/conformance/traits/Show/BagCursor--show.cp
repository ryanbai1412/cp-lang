// expect: ok, output "cursor at slot 0"
use std.bag.{SortedBag}
let b = SortedBag.of([4])
print(b.cursor_first()!)
