// expect: ok, output "YES NO"
// engines: next
use std.bitset.{XorTrie}
var t = XorTrie(3)
t.add(5)
t.add(1)
t.xor_all(1)
print(yes(t.contains(4)), yes(t.contains(5)))
