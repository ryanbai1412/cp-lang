// Codeforces 2135 E2: Beyond the Palindrome (Hard Version).
//
// Read 1 as a step up and 0 as a step down. Deleting "10" cancels an up
// with the down after it, so f(s) = 0^a 1^b with a = -min and b = end - min
// of the walk (prefix heights, starting at 0). Reversing s turns the walk
// around: f(rev s) has a' = max - end. So s is an almost-palindrome iff
//
//     max + min = end.
//
// Count the complement. The walks with a given (max, min) are the
// reflection-principle count inside that band, an alternating sum of
// binomials over the band's period. Summed over every band, the walks
// ending at ±d (C(n, (n+d)/2) of them each way) come with weight
// 2 * (2g(d) - g(d-1) - g(d+1)), a second difference of the editorial's
// band count g: the odd-divisor sum, less 1 for odd i and less 2 for
// i ≡ 2 (mod 4) when n is odd. Brute force agrees for n ≤ 12.
use std.math.{sieve_multiplicative}

type M = Mod<998_244_353>

let t: 1..=10_000 = read()
let ns: [0..t](1..=20_000_000) = read()
let lim = ns.max() + 2
let comb = Comb<M>(lim)

// odd_div_sum(2^k * m) = 2^k * σ(m) for odd m.
let odd_div_sum = sieve_multiplicative(lim, (p, _, pe) => if p == 2: pe else: (pe * p - 1) / (p - 1))

for n in ns:
    fn g(i: Int) -> M =
        if i <= 0: 0
        elif i % 2 == 1: M(odd_div_sum[i] - 1)
        elif n % 2 == 1 and i % 4 == 2: M(odd_div_sum[i] - 2)
        else: M(odd_div_sum[i])

    let bad = sum(2 * (2 * g(d) - g(d - 1) - g(d + 1)) * comb.choose(n, (n + d) / 2)
                  for d in (n % 2..=n).step(2))
    print(2 ** n - bad)
