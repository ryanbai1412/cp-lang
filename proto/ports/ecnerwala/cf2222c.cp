// Codeforces 2222 C: find the global median, then partition into the maximum
// number of odd-length segments each having that same median via O(N^2) DP.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let med = sorted([a[i] for i in 0..n])[n / 2]

var pref_lt: [0..=n]Int = 0
var pref_le: [0..=n]Int = 0
for i in 0..n:
    pref_lt[i + 1] = pref_lt[i] + Int(a[i] < med)
    pref_le[i + 1] = pref_le[i] + Int(a[i] <= med)

var dp: [0..=n]Int = -(n + 1)
dp[0] = 0
for i in 0..n:
    for j in i + 1..=n by 2:
        let cnt_lt = pref_lt[j] - pref_lt[i]
        let cnt_le = pref_le[j] - pref_le[i]
        let half = (j - i) / 2
        if cnt_lt <= half and half < cnt_le:
            dp[j] max= dp[i] + 1
print(dp[n])
