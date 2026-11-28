// expect: ok, output "1\n3"
// engines: next
use std.bitset.{DynBitset}
var b = DynBitset(4)
b.set(1)
b.set(3)
for i in b: print(i)
