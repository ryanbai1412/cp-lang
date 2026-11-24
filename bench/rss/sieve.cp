use std.math.{Sieve}
let n: 2..=10_000_000 = read()
let s = Sieve(n)
var c = 0
for x in 0..=n:
    if s.is_prime(x): c += 1
print(c)
