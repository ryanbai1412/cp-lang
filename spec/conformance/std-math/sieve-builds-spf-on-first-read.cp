// expect: ok, output "25 YES NO YES\n3 2 2 97\nYES"
// rule: 08.1.2
// `Sieve(n)` keeps the primes, a primality bitset and `spf`, built with the sieve (R05.10.9: a `@cache` field cannot be lent), and reads of `spf` agree with `is_prime`.
use std.math.{Sieve}
let sv = Sieve(100)
print(sv.primes.len, yes(sv.is_prime(97)), yes(sv.is_prime(91)), yes(sv.is_prime(2)))
print(sv.factor(60).len, sv.factor(60)[0].1, sv.spf[64], sv.spf[97])
var agree = true
for x in 0..=100: if sv.is_prime(x) != (x >= 2 and sv.spf[x] == x): agree = false
print(yes(agree))
