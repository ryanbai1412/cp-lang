# CCC 2019 S2 Pretty Average Primes: write 2n as a sum of two primes a + b with a <= b.
let ns = read [t] Int
let primes = Sieve(2 * ns.max())
for n in ns:
    let a = primes.primes.find(p => primes.is_prime(2 * n - p))!
    print(a, 2 * n - a)
