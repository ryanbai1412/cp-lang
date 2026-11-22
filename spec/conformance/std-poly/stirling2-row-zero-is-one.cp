// expect: ok, output "YES"
use std.poly.{stirling2_row}
type Z = Mod<998244353>
let s: List<Z> = stirling2_row(0)
print(yes(s == [Z(1)]))
