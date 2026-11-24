// n: 2048
// fixed: P, k
use std.poly.{stirling1_row, stirling2_row, stirling1_col, stirling2_col, bell_table, bernoulli_table, partition_table}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
var z = [Z(i * 3 + 2) for i in 0..n]
z[0] = Z(0)
for _ in 0..12:
    acc ^= stirling1_row<998_244_353>(n).len // timed: poly.stirling1_row
    acc ^= stirling2_row<998_244_353>(n).len // timed: poly.stirling2_row
    acc ^= stirling1_col<998_244_353>(3, n).len // timed: poly.stirling1_col
    acc ^= stirling2_col<998_244_353>(3, n).len // timed: poly.stirling2_col
    acc ^= bell_table<998_244_353>(n).len // timed: poly.bell_table
    acc ^= bernoulli_table<998_244_353>(n).len // timed: poly.bernoulli_table
    acc ^= partition_table<998_244_353>(n).len // timed: poly.partition_table
print(acc)
