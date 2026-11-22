// expect: ok, output "4"
use std.rand.{Rng}
domain V = 0..4
var r = Rng(4)
let p = r.perm<V>()
print(p.distinct().len)
