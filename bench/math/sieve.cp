// Matched benchmark: linear Sieve(5_000_000) + phi_table + mobius_table.
let n = 5_000_000
let sv = Sieve(n)
let phi = sv.phi_table()
let mu = sv.mobius_table()
var acc = 0
unchecked:
    for i in 1..=n: acc += phi[i] + mu[i] * i
print(sv.primes.len, acc)
