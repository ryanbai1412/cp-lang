// expect: ok, output "1"
// rule: R02.10.4
use std.bitset.Bitset
domain Q = 0..4
type M = Bitset<Q>
let m = M.of([Q(1)])
print(m.len)
