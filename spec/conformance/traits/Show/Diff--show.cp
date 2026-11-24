// expect: ok, output "3 4 1 1 1"
use std.prefix.{Diff}
domain D = 0..5
var d = Diff<D, Int>()
d.add(D(0)..D(2), 3)
d.add(D(1)..=D(4), 1)
print(d)
