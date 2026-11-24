// expect: R0401 (checked)
use std.dp.{knuth}
print(knuth(-1, (i, j) => j - i).len) // <- here
