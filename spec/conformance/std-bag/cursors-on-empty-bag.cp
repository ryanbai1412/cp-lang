// expect: ok, output "YES YES YES YES"
use std.bag.{SortedBag}
let e: List<Int> = []
let b = SortedBag.of(e)
print(yes(b.cursor_ge(0) == None), yes(b.cursor_gt(0) == None), yes(b.cursor_le(0) == None), yes(b.cursor_lt(0) == None))
