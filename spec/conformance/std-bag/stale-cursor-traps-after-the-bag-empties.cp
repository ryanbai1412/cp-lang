// expect: R0401 (checked)
use std.bag.{SortedBag}
var b = SortedBag<Int>()
for i in 0..100: b.add(i)
let c = b.cursor_first()!
for i in 0..100: b.remove(i)
b.add(7)
print(b.key_of(c))  // <- here
