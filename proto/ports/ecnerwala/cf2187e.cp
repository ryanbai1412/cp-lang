// Codeforces 2187 E: DP over the number of carried keys (bounded by O(sqrt(max A + N))),
// updating waiting vs spending a key at each door and pruning unreachable key counts.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let s: Str = read()

let t_inf = n + 1 + max(a)!
var dp: [0..=n + 1]Int = 0
var len = 1
var ans: [0..n]Int = 0

for i in 0..n:
    if s[i] == '1':
        for k in (0..len).rev: dp[k + 1] = dp[k]
        len += 1
    for k in 0..len:
        var best = max(a[i], dp[k]) + 1 + 2 * max(k - 1, 0)
        if k + 1 < len:
            best min= dp[k + 1] + 1 + 2 * k
        dp[k] = best
    while dp[len - 1] > t_inf: len -= 1
    ans[i] = dp[0]

print(ans)
