// expect: ok, output "0"
use std.bitset.{Bitset}
domain D = 0..10
print((Bitset<D>.of([D(1)]) & Bitset<D>.of([D(2)])).len)
