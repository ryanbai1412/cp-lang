// expect: ok, output "1 3"
// rule: decisions 460
use std.bitset.Bitset
domain Q = 0..4
print(Bitset<Q>.of([Q(3), Q(1)]))
