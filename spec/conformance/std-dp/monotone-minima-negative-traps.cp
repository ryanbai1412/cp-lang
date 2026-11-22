// expect: R0401 at 2 (checked)
use std.dp.{monotone_minima}
print(monotone_minima(-1, 2, (i, j) => Some(i + j)).len)
