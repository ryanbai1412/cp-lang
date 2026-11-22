// expect: ok, output "YES"
use std.search.{ternary_max}
let f = [0, 1, 2, 2, 1]
print(ternary_max(0..5, i => f[i]) == Some(2))
