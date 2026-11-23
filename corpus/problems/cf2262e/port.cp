// Source: Codeforces 2262E (Paired Bracket Sequences),
//   https://codeforces.com/contest/2262/problem/E
// Problem: for each k = 0..n, count ordered pairs (s, t) of balanced bracket sequences of length
//   2n with exactly k common pairings (matched pairs (i, j) in both), modulo a prime M in
//   [1e8, 1e9]. n <= 500, multitest, sum n <= 500.
// Solution: ans[k] counts pairs with k marked common pairings: the marks cut the sequence into
//   k + 1 balanced regions, a region of length 2i has C_i^2 choices of (s, t), and Lagrange
//   inversion gives ans[k] = C(2n, k) / (k + 1) [u^(n+1)] S^(k+1), S = sum C_i^2 u^(i+1).
//   Powers of S by convolve over Mod<M> (three NTT primes and CRT), O(n^2 log n); then exact
//   counts by inclusion-exclusion, out[j] = sum_k (-1)^(k-j) C(k, j) ans[k], O(n^2).

@!multitest
let (n, m): (1..=500, 2..=2 * 10 ** 9) = read()
type Z = Mod<m>
let comb = Comb<Z>()

let symbol: List<Z> = [if i == 0: 0 else: comb.catalan(i - 1) ** 2 for i in 0..=n + 1]
var cur: List<Z> = [if i == 0: 1 else: 0 for i in 0..=n + 1]
var ans: [0..=n]Z = 0
for k in 0..=n:
    cur = convolve(cur, symbol)[..n + 2].to_list()
    ans[k] = cur[n + 1] * comb.choose(2 * n, k) * comb.inv(k + 1)

fn shift(j: Int) -> Int:
    var tot: Z = 0
    for k in j..=n:
        let term = ans[k] * comb.choose(k, j)
        tot += if (k - j) % 2 == 0: term else: -term
    tot.v

print([shift(j) for j in 0..=n])
