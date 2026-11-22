// expect: ok, output "YES"
use std.search.{ternary_max_real}
print(ternary_max_real(2.0, 2.0, x => x) == 2.0)
