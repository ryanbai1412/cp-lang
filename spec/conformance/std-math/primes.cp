// expect: ok, output "YES"
use std.math.{primes_in, prime_count}
print(yes(primes_in(10, 30) == [11, 13, 17, 19, 23, 29] and primes_in(0, 3) == [2] and primes_in(20, 10) == [] and prime_count(10) == 4 and prime_count(100) == 25 and prime_count(1) == 0 and prime_count(1_000_000) == 78498 and primes_in(999_900, 1_000_000).len == prime_count(999_999) - prime_count(999_899)))
