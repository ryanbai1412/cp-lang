// expect: ok, output "YES YES"
use std.bag.{SortedBag}
let e: List<Int> = []
let b = SortedBag.of(e)
print(yes(b.cursor_first() == None), yes(b.cursor_last() == None))
