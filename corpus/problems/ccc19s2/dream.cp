// CCC 2019 S2 Pretty Average Primes: write 2n as a sum of two primes a + b with a <= b.
let t: 1..=1000 = read()
let ns: [t]4..=1e6 = read()
let primes = Sieve(2 * ns.max()!)
for n in ns:
    let a = primes.primes.find(p => primes.is_prime(2 * n - p))!
    print(a, 2 * n - a)
