// expect: ok, output "1 1 0 -1"
use std.math.{Sieve}
let s = Sieve(30)
print(s.phi(1), s.mobius(1), s.mobius(12), s.mobius(30))
