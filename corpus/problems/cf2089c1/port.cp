// Source: Codeforces 2089C1 (Key of Like, easy version),
//   https://codeforces.com/contest/2089/problem/C1
// limits: 3000 ms, 500 MB
// Problem: n <= 100 players in turn try one key on one lock, among l <= 5000 locks and their l
//   keys (k = 0 fakes), always a pair most likely to match given all past tries, ties at
//   random. Print each player's expected number of opened locks mod 1e9+7. t <= 100
//   tests, sum l <= 5000.
// Solution: with i locks left the next match comes at try s, uniform over 0..i, and the player
//   after it starts the next search. dp[j], player j's expectation when player 0 moves first,
//   averages shifted[(j - s) mod n] over s, where shifted[j] = dp_(i-1)[j - 1] + [j == 0].
//   Prefix sums with wrap counts give each cyclic window in O(1): O(n l).
// Bounds: n: 1..=100; l: 1..=5_000; _k: 0..=0

const MOD = 1_000_000_007
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let (n, l, _k): (1..=100, 1..=5_000, 0..=0) = read()
var dp: [0..n]Z = 0
for i in 1..=l:
    let pref = prefix([0..n] j => if j == 0: dp[n - 1] + 1 else: dp[j - 1])
    let inv_i = comb.inv(i)
    for k in 0..n:
        let start = k + 1 - i
        let wraps = (n - start) / n
        let window_sum = pref.cut(k + 1) - (
            pref.cut(start + n * wraps) - pref.cut(n) * wraps
        )
        dp[k] = window_sum * inv_i
print(dp)
