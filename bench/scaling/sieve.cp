use std.math.{Sieve}
let n: Int = read()
var acc = 0
for _ in 0..128:
    if Sieve(n).is_prime(7): acc += 1 // timed: math.Sieve.new
let s = Sieve(n)
for i in 0..8 * n:
    let x = n / 2 + i % (n / 2)
    if s.is_prime(x): acc += 1 // timed: math.Sieve.is_prime
    acc += s.factor(x).len // timed: math.Sieve.factor
    acc += s.phi(x) // timed: math.Sieve.phi
    acc += s.mobius(x) // timed: math.Sieve.mobius
    for e in s.divisors(x): acc += e // timed: math.Sieve.divisors
for _ in 0..128:
    acc += s.phi_table().len // timed: math.Sieve.phi_table
    acc += s.mobius_table().len // timed: math.Sieve.mobius_table
    acc += s.div_count_table().len // timed: math.Sieve.div_count_table
    acc += s.div_sum_table().len // timed: math.Sieve.div_sum_table
print(acc)
