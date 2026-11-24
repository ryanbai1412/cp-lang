// expect: ok, output "7 3"
use std.persistent.{PArena}
var a = PArena<Int>()
a.alloc(7)
a.alloc(3)
print(a)
