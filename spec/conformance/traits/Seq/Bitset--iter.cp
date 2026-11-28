// expect: ok, output "1\n3\n5"
// engines: next
use std.bitset.{Bitset}
domain D = 0..10
for d in Bitset<D>.of([D(5), D(1), D(3)]): print(d)
