// expect: ok, output "YES NO"
use std.bitset.{Bitset}
domain D = 0..10
let b = Bitset<D>.of([D(1), D(3)])
print(yes(D(3) in b), yes(D(2) in b))
