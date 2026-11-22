// Codeforces 2089 C1 — Key of Like (Easy Version, `_k = 0` fake keys)
//
// `dp[j]` is the expected number of keys found by player `j` (0-based) when
// `i` keys remain and player `0` moves next. When a key is found by player `0`:
// - Player `0` earns `+1`, and the turn advances by `1` mod `n`, so after the
//   find player `j` inherits `shifted[j] = dp[(j - 1) mod n] + (1 if j == 0)`.
// - Because any of the `i` remaining keys is equally likely to be the first
//   matched (at step `s in 0..i` around the circle), `dp[k]` is the average of
//   `shifted[(k - s) mod n]` over the cyclic window of length `i` ending at `k`.
//   `cyclic_window_sum` computes this sum in O(1) using `pref` and `wraps`.

const MOD = 1_000_000_007
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let (n, l, _k): (1..=5_000, 1..=5_000, 0..=0) = read()
var dp: [0..n]Z = 0
for i in 1..=l:
    let pref = prefix([0..n] j => if j == 0: dp[n - 1] + 1 else: dp[j - 1])
    let inv_i = comb.inv(i)
    for k in 0..n:
        let start = k + 1 - i
        let wraps = (n - start) / n
        let window_sum = pref.prefix(k + 1) - (
            pref.prefix(start + n * wraps) - pref.prefix(n) * wraps
        )
        dp[k] = window_sum * inv_i
print(dp)
