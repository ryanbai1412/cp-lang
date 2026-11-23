// CCC 2019 S2, Pretty Average Primes: for each n, print primes a <= b with
// a + b = 2n (one exists for every n in the limits); the original prints
// the pair with the smallest a.
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
