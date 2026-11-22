// expect: ok, output "YES"
use std.search.{ternary_min}
print(ternary_min(5..5, i => i) == None)
