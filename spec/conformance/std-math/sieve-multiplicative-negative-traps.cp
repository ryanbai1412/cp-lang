// expect: R0401 at 2 (checked)
use std.math.{sieve_multiplicative}
print(sieve_multiplicative(-1, (p, e, pe) => e + 1).len)
