// expect: ok, output "YES"
use std.search.{ternary_max}
print(ternary_max(0..0, i => i) == None)
