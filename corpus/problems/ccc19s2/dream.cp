// ccc19s2 Pretty Average Primes: two primes averaging n.
let sieve = Sieve(2_000_000)
@!multitest
let n = read<4..=1_000_000>()
let p = (2..=n).find(p => sieve.is_prime(p) and sieve.is_prime(2 * n - p))!
print(p, 2 * n - p)
