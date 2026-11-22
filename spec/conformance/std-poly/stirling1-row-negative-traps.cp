// expect: R0401 at 3
use std.poly.{stirling1_row}
type Z = Mod<998244353>
let s: List<Z> = stirling1_row(-1)
print(s.len)
