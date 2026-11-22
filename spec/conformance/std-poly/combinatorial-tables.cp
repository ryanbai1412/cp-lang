// expect: ok, output "YES YES YES YES YES"
// rule: 07.7
use std.poly.{bell_table, bernoulli_table, partition_table, stirling1_row, stirling2_row}
type Z = Mod<998244353>
let s1: List<Z> = stirling1_row(3)
let s2: List<Z> = stirling2_row(3)
let b: List<Z> = bell_table(4)
let bn: List<Z> = bernoulli_table(2)
let p: List<Z> = partition_table(5)
print(
    yes(s1 == [Z(0), Z(2), Z(3), Z(1)]),
    yes(s2 == [Z(0), Z(1), Z(3), Z(1)]),
    yes(b == [Z(1), Z(1), Z(2), Z(5), Z(15)]),
    yes(bn == [Z(1), Z(-1) / Z(2), Z(1) / Z(6)]),
    yes(p == [Z(1), Z(1), Z(2), Z(3), Z(5), Z(7)]),
)
