// Codeforces 2135 E2: Beyond the Palindrome (Hard Version).
//
// Read 1 as a step up and 0 as a step down. Deleting "10" cancels an up
// with the down after it, so f(s) = 0^a 1^b with a = -min and b = end - min
// of the walk (prefix heights, starting at 0). Reversing s turns the walk
// around: f(rev s) has a' = max - end. So s is an almost-palindrome iff
//
//     max + min = end.
//
// Count the complement. Walks inside a band of period p (width p - 1) are
// counted by the reflection principle, an alternating sum of binomials
// repeating every p. Taking exact (max, min) by inclusion-exclusion and
// summing over every band, the walks ending at height ±d (C(n, (n+d)/2)
// each way) carry weight 2 * (2g(d) - g(d-1) - g(d+1)), where
//
//     g(i) = sum of p over p | i with i / p odd, for the periods p that
//            can hold a bad walk.
//
// The sum over every such p is odd_div_sum[i]. Period 1 (a walk that never
// moves) is never possible, and period 2 only when n is even: an odd walk in
// a width-1 band always ends at max + min. Period 1 appears in i iff i is
// odd and period 2 iff i ≡ 2 (mod 4). A brute force agrees for n <= 12.
use std.math.{sieve_multiplicative}

type M = Mod<998_244_353>

let t: 1..=10_000 = read()
let ns: [0..t](1..=20_000_000) = read()
let lim = ns.max() + 2
let comb = Comb<M>(lim)

// odd_div_sum(2^k * m) = 2^k * σ(m) for odd m.
let odd_div_sum = sieve_multiplicative(
    lim,
    (p, _, pe) => if p == 2: pe else: (pe * p - 1) / (p - 1),
)

for n in ns:
    fn g(i: Int) -> M =
        if i <= 0: 0
        else:
            let narrow = (if i % 2 == 1: 1 else: 0) + (if n % 2 == 1 and i % 4 == 2: 2 else: 0)
            M(odd_div_sum[i] - narrow)

    let bad = sum(
        2 * (
            2 * g(d) - g(d - 1) - g(d + 1)
        ) * comb.choose(n, (n + d) / 2) for d in (n % 2..=n).step(2)
    )
    print(M(2) ** n - bad)
