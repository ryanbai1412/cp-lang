// expect: R0401 at 4
// A non-permutation traps instead of giving a silent answer (its doc used to promise a debug trap that never ran).
use std.seq.*
let p = [0, 0, 1]
print(perm_cycles(p))
