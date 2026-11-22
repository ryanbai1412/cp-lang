// expect: ok, output "YES"
// rule: std.fenwick.Fenwick.kth
use std.fenwick.{Fenwick}
domain D = 0..3
var f = Fenwick<D, Int>()
f.add(D(0), 1)
f.add(D(2), 2)
print(yes(f.kth(3) == Some(D(2)) and f.kth(4) == None))
