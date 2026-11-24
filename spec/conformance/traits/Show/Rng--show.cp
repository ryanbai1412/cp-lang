// expect: ok, output "rng state 1"
use std.rand.{Rng}
var r = Rng(1)
print(r)
