// expect: ok, output "YES"
use std.poly.{bell_table}
type Z = Mod<998244353>
let s: List<Z> = bell_table(0)
print(yes(s == [Z(1)]))
