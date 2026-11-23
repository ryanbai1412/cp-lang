// Codeforces 1987D World is Mine: Alice eats strictly increasing cakes, Bob any; how many for Alice?
// In: t; per test n, then a_1..a_n (1..n). Out: Alice's count per test.
@!multitest
let n = read<1..=5000>()
let a = read<[n]1..=n>()
var have: [1..=n]Int = 0
for x in a: have[x] += 1
let cnt = [c for c in have if c > 0]
let m = cnt.len

// Bob erases some values entirely before Alice reaches them. dp[s]: fewest cakes Bob spends to
// erase s values; value i is reachable only if he finishes it within the i - s turns Alice spends
// on the earlier values she eats.
var dp: [0..=m]?Int = None
dp[0] = 0
for i in 0..m:
    for s in (0..=i).rev:
        if let d = dp[s], d + cnt[i] <= i - s:
            dp[s + 1] min= d + cnt[i]
print(m - (0..=m).filter(s => dp[s] != None).max()!)
