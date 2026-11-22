// Codeforces 2187 E — Keys and Doors
//
// At door i (opening time a[i]), s[i] == '1' grants a key before crossing.
// Let `dp[k]` be the minimum time to pass doors 0..i while carrying k keys
// (stored as a `List<Int>` whose length is the active key count):
// - If s[i] == '1', we may pick up the key, shifting `k -> k + 1` while
//   keeping `dp[0]` unchanged (`dp.insert(0, dp[0])`).
// - Crossing door i while ending with k carried keys either:
//   - waits until a[i] and walks through without spending a key, paying
//     `1 + 2 * max(k - 1, 0)` for the extra key-carrying steps; or
//   - spends one of `k + 1` keys to skip waiting, paying `dp[k + 1] + 1 + 2 * k`.
//
// Waiting at every door with 0 keys finishes by `t_inf = n + 1 + a.max()!`,
// so states with `dp[k] > t_inf` can never improve `dp[0]` and are popped;
// the quadratic key-carrying penalty bounds `dp.len` to O(sqrt(max A + n)).

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]0..=10 ** 9 = read()
let s: Str = read()

let t_inf = n + 1 + a.max()!
var dp: List<Int> = [0]
var ans: [0..n]Int = 0

for i in 0..n:
    if s[i] == '1': dp.insert(0, dp[0])
    for k in 0..dp.len:
        var best = max(a[i], dp[k]) + 1 + 2 * max(k - 1, 0)
        if k + 1 < dp.len:
            best min= dp[k + 1] + 1 + 2 * k
        dp[k] = best
    while dp.last! > t_inf: dp.pop()
    ans[i] = dp[0]

print(ans)
