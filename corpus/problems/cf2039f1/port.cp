// Source: Codeforces 2039F1 (Shohag Loves Counting, Easy Version),
//   https://codeforces.com/contest/2039/problem/F1
// Problem: f(k) is the gcd of the maxima of all length-k subarrays; an array is good if f(1..n)
//   are pairwise distinct. Given m, count non-empty good arrays of any length over 1..=m,
//   mod 998244353. Multitest, sum of m <= 1e5.
// Solution: a good array has distinct values whose sorted suffix gcds strictly grow, and each such
//   set orders in 2^(len - 1) good ways. Add values y from m down: a set with gcd g takes y iff
//   gcd(y, g) < g, doubling its weight. By Mobius, the weight landing on gcd h | y is
//   sum over d | y/h of mu(d) above[h d] (above[x] = weight of sets whose gcd x divides), minus
//   at[h], the sets already at gcd h. Divisor lists; about O(m log^2 m) per test.
@!multitest
type Z = Mod<998_244_353>
let m: 1..=100_000 = read()
let mu = Sieve.new(m).mobius_table()
var divs: [m + 1]List<Int> = []
for d in 1..=m:
    for x in (d..=m).step(d): divs[x].push(d)

var at: [m + 1]Z = 0
var above: [m + 1]Z = 0
for y in (1..=m).rev:
    let grown = [(h, 2 * (sum(mu[d] * above[h * d] for d in divs[y / h]) - at[h])) for h in divs[y]]
    for (h, w) in grown + [(y, 1)]:
        at[h] += w
        for d in divs[h]: above[d] += w
print(above[1])
