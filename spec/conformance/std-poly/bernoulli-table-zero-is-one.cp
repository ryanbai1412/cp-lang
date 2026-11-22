// expect: ok, output "YES"
use std.poly.{bernoulli_table}
type Z = Mod<998244353>
let s: List<Z> = bernoulli_table(0)
print(yes(s == [Z(1)]))
