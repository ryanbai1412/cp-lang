// expect: ok, output "0 10"
// `choose(n, k)` is 0 when k > n (round 23, H11).
use std.mod.{Comb}
type Z = Mod<998_244_353>
let c = Comb<Z>.new(10)
print(c.choose(3, 5), c.choose(5, 2))
