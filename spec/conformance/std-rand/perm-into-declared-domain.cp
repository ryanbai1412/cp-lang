// expect: ok, output "4 4"
use std.rand.{Rng}
domain V = 0..4
var r = Rng(seed: 7)
let p: [V]V = r.perm()
print(p.len, p.distinct().len)
