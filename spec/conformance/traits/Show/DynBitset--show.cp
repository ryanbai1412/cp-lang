// expect: ok, output "1 3"
use std.bitset.{DynBitset}
var b = DynBitset(4)
b.set(1)
b.set(3)
print(b)
