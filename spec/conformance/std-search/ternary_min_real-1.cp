// expect: ok, output "YES"
use std.search.{ternary_min_real}
print(ternary_min_real(1.5, 1.5, x => x) == 1.5)
