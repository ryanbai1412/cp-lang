// expect: ok, output "YES"
use std.search.{ternary_max}
print(yes(ternary_max(0..6, i => -(i - 3) * (i - 3)) == Some(3)))
