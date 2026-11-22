// expect: ok, output "YES"
use std.search.{ternary_min}
print(yes(ternary_min(0..6, i => (i - 4) * (i - 4)) == Some(4)))
