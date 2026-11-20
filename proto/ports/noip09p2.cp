// NOIP 2009 Hankson: count x with gcd(x, a0) = a1 and lcm(x, b0) = b1.
// Every such x divides b1.
use std.math.{divisors}

@!multitest
let (a0, a1, b0, b1): (1..=2e9, 1..=2e9, 1..=2e9, 1..=2e9) = read()
print(count(gcd(x, a0) == a1 and lcm(x, b0) == b1 for x in divisors(b1)))
