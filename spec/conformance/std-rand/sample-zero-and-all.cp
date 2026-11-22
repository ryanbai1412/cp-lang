// expect: ok, output "0 5 6 7"
use std.rand.{Rng}
var r = Rng(9)
let taken = r.sample(0, from: [5, 6, 7])
print(taken.len, r.sample(3, from: [5, 6, 7]).sorted())
