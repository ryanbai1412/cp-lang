// expect: ok, output "2"
// `Sieve(n)` is `Sieve.new(n)`, as R10.1 writes it (round 26, H07).
use std.math.{Sieve}
let sv = Sieve(30)
print(sv.spf[12])
