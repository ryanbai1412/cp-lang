// expect: ok, output "1 1"
// rule: std.bitset.Bitset.sub
use std.bitset.{Bitset}
domain D = 0..10
let a = Bitset<D>.of([D(1), D(2)])
let b = Bitset<D>.of([D(2), D(3)])
var c = Bitset<D>.of([D(1), D(2)])
c -= b
print((a - b).len, c.len)
