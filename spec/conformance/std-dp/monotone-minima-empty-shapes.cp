// expect: ok, output "0 YES"
use std.dp.{monotone_minima}
let r = monotone_minima(2, 0, (i, j) => Some(i + j))
print(monotone_minima(0, 3, (i, j) => Some(i + j)).len, yes(r == [None, None]))
