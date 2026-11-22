// expect: ok, output "[(b, 2), (d, 1)]"
// rule: 07.2.1
use std.seq.{runs}
let ss = "abbd"
print(ss[1..4].runs())
