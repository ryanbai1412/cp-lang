// Source: Codeforces 2135E1 (Beyond the Palindrome, easy version),
//   https://codeforces.com/contest/2135/problem/E1
// Problem: f(r) deletes all "10" substrings at once, repeatedly, until none is left. Count binary
//   strings s of length n with f(s) = f(rev(s)), mod 998244353. n <= 1e6, sum of n <= 1e6,
//   multitest.
// Solution: read 1 as a step up and 0 as a step down; s qualifies iff its walk from 0 has
//   max + min = end (see the E2 dream for the derivation). Count the complement by the reflection
//   principle summed over every band: walks ending at +-d weigh 2 (2g(d) - g(d-1) - g(d+1)), with
//   g(i) the sum of the periods p | i, i / p odd, that can hold a bad walk (period 1 never,
//   period 2 only for even n). Odd-divisor sums come from a multiplicative sieve, binomials from
//   factorials: O(lim) once, then O(n) per test.
use std.math.{sieve_multiplicative}

type M = Mod<998_244_353>
// g reads odd_div_sum up to n + 1.
let lim = 2e5 + 2
let comb = Comb<M>(lim)
let odd_div_sum = sieve_multiplicative(
    lim,
    (p, _, pe) => if p == 2: pe else: (pe * p - 1) / (p - 1),
)

@!multitest
let n: 1..=2e5 = read()

fn g(i: Int) -> M =
    if i <= 0: 0
    else:
        let narrow = (if i % 2 == 1: 1 else: 0) + (if n % 2 == 1 and i % 4 == 2: 2 else: 0)
        M(odd_div_sum[i] - narrow)

let bad = sum(
    2 * (2 * g(d) - g(d - 1) - g(d + 1)) * comb.choose(n, (n + d) / 2) for d in (n % 2..=n).step(2)
)
print(M(2) ** n - bad)
