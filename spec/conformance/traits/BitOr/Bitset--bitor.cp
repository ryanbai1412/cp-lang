// expect: ok, output "3"
use std.bitset.{Bitset}
domain D = 0..10
let a = Bitset<D>.of([D(1), D(2)]) | Bitset<D>.of([D(2), D(3)])
print(a.len)
