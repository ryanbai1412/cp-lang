// Source: Codeforces 2222C (Median Partition), https://codeforces.com/contest/2222/problem/C
// Problem: an array of odd length n < 5000 (a_i <= 1e9). Cut it into the most subarrays of
//   odd length that all have the same median; print that number. Multitest, sum of n^2 <=
//   5000^2.
// Solution: in each odd segment more than half the elements are <= its median and more than
//   half >= it, so a common median m does the same for the whole array: m is the global
//   median med. dp[j] = most segments for the prefix of length j, over odd segments (i, j]
//   with median med (count < med and <= med by prefix sums). The whole array always works.
//   O(n^2).

@!multitest
let n: 1..=5_000 = read()
let a: [0..n]1..=10 ** 9 = read()
let med = a.sorted()[n / 2]
let pref_lt = prefix([Int(x < med) for x in a])
let pref_le = prefix([Int(x <= med) for x in a])

var dp: [0..=n]?Int = None
dp[0] = 0
for i in 0..n if let cur = dp[i]:
    for j in i + 1..=n by 2:
        if pref_lt.fold(i..j) <= (j - i) / 2 < pref_le.fold(i..j):
            dp[j] max= cur + 1
print(dp[n]!)
