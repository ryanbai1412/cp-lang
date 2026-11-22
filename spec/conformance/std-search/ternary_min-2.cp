// expect: ok, output "YES"
use std.search.{ternary_min}
let f = [3, 1, 1, 2]
print(ternary_min(0..4, i => f[i]) == Some(1))
