// Source: Codeforces 2135E2 (Beyond the Palindrome, Hard Version),
//   https://codeforces.com/contest/2135/problem/E2
// limits: 3000 ms, 1024 MB
// Problem: f(r) deletes every "10" from the binary string r, repeatedly, until none is left. For
//   each n (<= 2e7, sum <= 1e8, t <= 1e4) count the binary strings s of length n with
//   f(s) = f(rev(s)), mod 998244353. Multitest.
// Solution: read 1 as +1 and 0 as -1: f(s) = 0^(-min) 1^(end - min) of the prefix walk and
//   f(rev s) starts with max - end zeros, so s counts iff max + min = end. Count the rest: the
//   reflection principle counts walks in a band of period p, and inclusion-exclusion over
//   (max, min) weights the C(n, (n+d)/2) walks ending at +-d by 2(2g(d) - g(d-1) - g(d+1)),
//   g(i) = sum of p | i with i/p odd, less periods 1 (always) and 2 (n odd) that hold no bad
//   walk. One multiplicative sieve of odd-divisor sums to max n, then O(n) per test.
// A brute force agrees for n <= 12.
use std.math.{sieve_multiplicative}

type M = Mod<998_244_353>

let t: 1..=10_000 = read()
let ns: [0..t](1..=20_000_000) = read()
let lim = ns.max()! + 2
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
