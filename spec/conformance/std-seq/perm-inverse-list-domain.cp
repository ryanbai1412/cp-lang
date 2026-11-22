// expect: ok, output "YES YES YES"
// rule: 04.2
use std.seq.{inverse, perm_cycles, perm_pow}
let p = [1, 2, 0, 4, 3]
print(yes(perm_cycles(p) == [[0, 1, 2], [3, 4]]), yes(perm_pow(p, 2) == [2, 0, 1, 3, 4]), yes(p.inverse() == [2, 0, 1, 4, 3]))
