// expect: R0101
// A handle past `len` (here one minted by another, larger arena) traps at `get`, in both engines.
use std.persistent.{PArena}
var big = PArena<Int>()
var small = PArena<Int>()
big.alloc(1)
let far = big.alloc(2)
small.alloc(7)
print(small.get(far)) // <- here
