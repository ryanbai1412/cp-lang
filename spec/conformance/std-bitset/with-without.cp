// expect: ok, output "1 3\n3"
use std.bitset.{Bitset}
domain D = 0..10
let b = Bitset<D>.single(D(1)).with(D(3))
print(b.each().collect())
print(b.without(D(1)).each().collect())
