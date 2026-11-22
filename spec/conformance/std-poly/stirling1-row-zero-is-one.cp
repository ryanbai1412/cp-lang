// expect: ok, output "YES"
use std.poly.{stirling1_row}
type Z = Mod<998244353>
let s: List<Z> = stirling1_row(0)
print(yes(s == [Z(1)]))
