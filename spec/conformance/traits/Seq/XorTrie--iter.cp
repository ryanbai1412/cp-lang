// expect: ok, output "0\n4"
// engines: next
use std.bitset.{XorTrie}
var t = XorTrie(3)
t.add(5)
t.add(1)
t.xor_all(1)
for x in t: print(x)
