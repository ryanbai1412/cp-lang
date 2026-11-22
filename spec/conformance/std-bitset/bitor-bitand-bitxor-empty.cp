// expect: ok, output "1 0 1"
use std.bitset.{Bitset}
domain D = 0..4
let a = Bitset<D>.of([D(1)])
let e = Bitset<D>.of([])
print((a | e).len, (a & e).len, (a ^ e).len)
