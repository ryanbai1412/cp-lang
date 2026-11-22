// expect: ok, output "5"
use std.dp.{knuth}
print(knuth(3, (i, j) => j - i)[0][3])
