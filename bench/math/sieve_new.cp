// Matched benchmark: the linear sieve alone, Sieve.new(1_000_000), built 20 times.
let n = 1_000_000
var acc = 0
for r in 0..20:
    let sv = Sieve.new(n)
    acc += sv.primes.len + sv.spf[n - r]
print(acc)
