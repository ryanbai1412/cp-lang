// expect: ok, output "YES"
use std.seq.{perm_cycles}
print(yes(perm_cycles([0, 1, 2]) == [[0], [1], [2]]))
