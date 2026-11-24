// Source: CCC 2019 S2 (Pretty Average Primes), https://dmoj.ca/problem/ccc19s2
// limits: 1000 ms, 1024 MB
// Problem: t <= 1000 values 4 <= n <= 1e6: for each, print primes a and b with a + b = 2n (any
//   such pair; one always exists).
// Solution: sieve up to 2 max n once, then per n the first prime a with 2n - a prime (the pair
//   with the smallest a, as the original prints); a small a always works in practice.
// original: corpus/submissions/4325077.cpp
// Bounds: t: 1..=1000; ns: 4..=1e6
use std.math.{primes_in}

let t: 1..=1000 = read()
let ns: [t]4..=1e6 = read()
let primes = primes_in(2, 2 * ns.max()! + 1)
var is_prime = List.filled(2 * ns.max()! + 1, false)
for p in primes: is_prime[p] = true
for n in ns:
    let a = primes.find(p => is_prime[2 * n - p])!
    print(a, 2 * n - a)
