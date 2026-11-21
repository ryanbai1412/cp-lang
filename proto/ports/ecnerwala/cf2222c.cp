// Codeforces 2222 C: find the global median, then partition into the maximum
// number of odd-length segments each having that same median via O(N^2) DP.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let med = sorted(a)[n / 2]
let pref_lt = prefix_n([Int(x < med) for x in a])
let pref_le = prefix_n([Int(x <= med) for x in a])

var dp: [0..=n]?Int = none
dp[0] = 0
for i in 0..n if let cur = dp[i]:
    for j in i + 1..=n by 2:
        if pref_lt.sum(i..j) <= (j - i) / 2 < pref_le.sum(i..j):
            dp[j] max= cur + 1
print(dp[n]!)
