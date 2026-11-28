// expect: ok, output "3 0 1 2\nab cd ef\nYES"
// rule: std.persistent.PArena
// `alloc` issues handles 0, 1, 2 in order and a node never changes after it is stored, so a handle
// read before or after later allocations gives the same value. `get` lends the arena while its
// result lives (R05.5.3), so a value kept across `alloc` is a clone.
use std.persistent.{PArena}
var a = PArena<Str>(cap: 2)
let x = a.alloc("ab")
let y = a.alloc("cd")
let first = a.get(x).clone()
let z = a.alloc("ef")
a.reserve(1000)
print(a.len, x.raw, y.raw, z.raw)
print(a.get(x), a.get(y), a.get(z))
print(yes(first == a.get(x) and not a.is_empty and PArena<Int>().is_empty))
