// expect: ok, output "0 1 3"
use std.math.{Sieve}
let s = Sieve(100)
print(s.factor(1).len, s.factor(97).len, s.factor(60).len)
