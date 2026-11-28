// expect: ok, output "YES NO"
// engines: next
use std.bitset.{DynBitset}
var b = DynBitset(4)
b.set(1)
b.set(3)
print(yes(b.contains(3)), yes(b.contains(2)))
