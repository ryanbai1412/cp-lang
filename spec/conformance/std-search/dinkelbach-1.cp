// expect: ok, output "YES"
use std.search.{dinkelbach}
print(dinkelbach(0.0, solve: _ => (num: 3.0, den: 2.0)) == 1.5)
