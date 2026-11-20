// Codeforces 2187 E: DP over the number of carried keys (bounded by O(sqrt(max A + N))),
// updating waiting vs spending a key at each door and pruning unreachable key counts.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let s: Str = read()

let t_inf = n + 1 + max(a)!
var dp: List<Int> = [0]
var ans: [0..n]Int = 0
for i in 0..n:
    if s[i] == '1':
        var ndp: List<Int> = [dp[0]]
        for v in dp: ndp.push(v)
        dp = ndp
    let len = dp.len
    for k in 0..len:
        var best = max(a[i], dp[k]) + 1 + 2 * max(k - 1, 0)
        if k + 1 < len:
            best min= dp[k + 1] + 1 + 2 * k
        dp[k] = best
    while dp.last! > t_inf: dp.pop()
    ans[i] = dp[0]
print(ans)
