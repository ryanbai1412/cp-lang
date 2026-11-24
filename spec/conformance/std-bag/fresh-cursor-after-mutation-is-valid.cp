// expect: ok, output "0 3"
use std.bag.{SortedBag}
var b = SortedBag.of([1, 2, 3])
let _ = b.cursor_first()!
b.add(0)
let c = b.cursor_first()!
print(b.key_of(c), b.key_of(b.cursor_last()!))
