// expect: ok, output "YES"
use std.search.{ternary_max}
print(yes(ternary_max(0..0, i => i) == None))
