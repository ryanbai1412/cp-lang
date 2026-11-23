// expect: ok, output "1 3 5"
use std.bitset.{Bitset}
domain D = 0..10
print(Bitset<D>.of([D(5), D(1), D(3)]))
