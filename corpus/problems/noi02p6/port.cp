// Source: NOI 2002 P6 (Robot No. M), https://dmoj.ca/problem/noi02p6
// limits: 1000 ms, 16 MB
// Problem: m = prod p_i^e_i is given factored (k <= 1000 primes < 1e4, e_i <= 1e6). Robot m's
//   teachers are its divisors other than 1 and m; robot d's independence number counts the
//   smaller robots sharing no teacher with it. Over m and its teachers, sum that number for d
//   a product of an even count of distinct odd primes, of an odd count, and the rest, mod 1e4.
// Solution: the independence number of d is phi(d). The first two classes are squarefree odd
//   d, where phi(d) = prod (p - 1): track the even- and odd-count sums over the odd primes
//   (d = 1 counts as even, so take it off). phi summed over all divisors is m, so the rest is
//   m - even - odd. O(k log e).
// original: corpus/submissions/3754486.cpp
// Bounds: k: 1..=1000; factors.p: 2..=9999; factors.e: 1..=1e6
type Z = Mod<10_000>
let k: 1..=1000 = read()
let factors: [k](p: 2..=9999, e: 1..=1e6) = read()

// sums of phi(d) = prod (p - 1) over squarefree odd d with an even (d = 1 included) and an odd
// count of prime factors
var (even, odd) = (Z(1), Z(0))
for f in factors if f.p > 2:
    (even, odd) = (even + odd * (f.p - 1), odd + even * (f.p - 1))
let m = prod(Z(f.p) ** f.e for f in factors)
print(even - 1)
print(odd)
print(m - even - odd)
