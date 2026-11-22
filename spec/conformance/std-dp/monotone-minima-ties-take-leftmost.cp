// expect: ok, output "YES"
use std.dp.{monotone_minima}
print(yes(monotone_minima(1, 3, (i, j) => Some(0)) == [Some((val: 0, at: 0))]))
