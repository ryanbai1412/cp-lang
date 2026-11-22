// expect: ok, output "2 10\n1"
use std.math.{factor}
let f = factor(1024)
print(f[0].0, f[0].1)
print(factor(97).len)
