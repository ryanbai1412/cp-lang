// expect: ok, output "0 0"
// rule: std.fenwick.Fenwick.new
use std.fenwick.{Fenwick}
domain E = 0..0
let f = Fenwick<E, Int>()
print(f.all, f.fold(E.all))
