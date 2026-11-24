// Source: CCC 2019 S2 (Pretty Average Primes), https://dmoj.ca/problem/ccc19s2
// Problem: t <= 1000 values 4 <= n <= 1e6: for each, print primes a and b with a + b = 2n (any
//   such pair; one always exists).
// Solution: sieve up to 2 max n, then for each n try the primes a in increasing order until
//   2n - a is prime, printing the pair with the smallest a (as the original does); a small a
//   always works in practice.
// original: corpus/submissions/4325077.cpp

let t: Int = read()
domain Case = 0..t
let ns: [Case]Int = read()
let sieve = Sieve(2 * [ns[c] for c in Case].max()!)
for c in Case:
    for a in sieve.primes:
        if sieve.is_prime(2 * ns[c] - a):
            print(a, 2 * ns[c] - a)
            break
