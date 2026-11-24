// expect: ok, output "3 elements\n4 elements"
// rule: std.lazy SeqFold prints its element count
use std.lazy.{SeqFold}
var s = SeqFold<Int>(+, [1, 2, 3])
print(s)
s.push(4)
print(s)
