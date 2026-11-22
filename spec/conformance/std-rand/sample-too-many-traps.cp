// expect: R0401 at 3 (checked)
use std.rand.{Rng}
var r = Rng(9)
print(r.sample(4, from: [5, 6, 7]).len)
