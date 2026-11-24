// expect: ok, output "7 3 7"
use std.lazy.{Rope}
var r = Rope([1, 2, 3])
r.apply(0..2, 7)
r.reverse(1..3)
print(r)
