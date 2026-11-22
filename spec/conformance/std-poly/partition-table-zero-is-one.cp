// expect: ok, output "YES"
use std.poly.{partition_table}
type Z = Mod<998244353>
let s: List<Z> = partition_table(0)
print(yes(s == [Z(1)]))
