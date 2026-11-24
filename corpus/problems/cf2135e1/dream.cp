// Codeforces 2135 E1: Beyond the Palindrome (Easy Version).
//
// Read 1 as a step up and 0 as a step down. s is an almost-palindrome iff
// its walk (prefix heights from 0) has max + min = end; see the E2 dream for
// the derivation. Count the complement by the reflection principle summed
// over every band: walks ending at ±d carry weight
// 2 * (2g(d) - g(d-1) - g(d+1)), with g(i) the sum of the periods p that can
// hold a bad walk among p | i with i / p odd. Period 1 is never possible and
// period 2 only for even n. The same closed form solves both versions; E1's
// smaller n only lets the odd-divisor sums come from a plain divisor sieve.
use std.math.{sieve_multiplicative}

type M = Mod<998_244_353>
// g reads odd_div_sum up to n + 1.
let lim = 1e6 + 2
let comb = Comb<M>(lim)
let odd_div_sum = sieve_multiplicative(lim, (p, _, pe) => if p == 2: pe else: (pe * p - 1) / (p - 1))

@!multitest
let n: 1..=1e6 = read()

fn g(i: Int) -> M =
    if i <= 0: 0
    else:
        let narrow = (if i % 2 == 1: 1 else: 0) + (if n % 2 == 1 and i % 4 == 2: 2 else: 0)
        M(odd_div_sum[i] - narrow)

let bad = sum(2 * (2 * g(d) - g(d - 1) - g(d + 1)) * comb.choose(n, (n + d) / 2)
              for d in (n % 2..=n).step(2))
print(M(2) ** n - bad)
