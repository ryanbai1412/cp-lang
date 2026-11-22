// expect: ok, output "YES"
use std.math.{sieve_multiplicative}
print(yes(sieve_multiplicative(0, (p, e, pe) => e + 1) == [1]))
