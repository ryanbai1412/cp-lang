// Codeforces 2165 B — Marble Council
//
// A subset `S` of distinct values can be the chosen multiset of modes iff its
// total multiplicity `sum_{v in S} freq[v] >= max_v freq[v]` (so every color
// class can be distributed across distinct piles without exceeding the chosen
// modes). Each selected value `v` with frequency `f` contributes `f` choices
// for how many copies enter the mode multiset (1..=f), so the knapsack
// `dp[i + f] += dp[i] * f` accumulates the sum of `prod_{v in S} freq[v]` over
// all subsets `S` of total frequency `i`. Summing `dp[i]` for `i >= freq.max()!`
// yields the answer in `O(n^2)`.

const MOD = 998_244_353
type Z = Mod<MOD>

@!multitest
let n: 1..=5_000 = read()
let a: [0..n]1..=n = read()
var freq: [1..=n]Int = 0
for x in a: freq[x] += 1

var dp: [0..=n]Z = 0
dp[0] = 1
var sz = 0
for f in freq if f > 0:
    for i in (0..=sz).rev: dp[i + f] += dp[i] * f
    sz += f

print(sum(dp[i] for i in freq.max()!..=n))
