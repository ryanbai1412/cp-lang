// Source: AtCoder Educational DP Contest F (LCS), https://atcoder.jp/contests/dp/tasks/dp_f
// Problem: two lowercase strings s, t of length <= 3000. Print one longest common subsequence.
// Solution: dp[i, j] is the LCS length of the suffixes from i and j: max of skipping either
//   head, or 1 + dp[i+1, j+1] on equal heads. Walk from (0, 0), skipping a head while the value
//   stays the same and taking the character otherwise. O(nm).
// original: corpus/submissions/4131645.cpp
let a: Str = read()
let b: Str = read()
let n = a.len
let m = b.len
domain R = 0..=n
domain C = 0..=m

var dp: [R, C]Int = 0
for ii in 0..n:
    let i = n - 1 - ii
    for jj in 0..m:
        let j = m - 1 - jj
        var v = max(dp[R(i + 1), C(j)], dp[R(i), C(j + 1)])
        if a[i] == b[j]:
            v max= 1 + dp[R(i + 1), C(j + 1)]
        dp[R(i), C(j)] = v

var out: List<Char> = []
var i = 0
var j = 0
while i < n and j < m:
    let cur = dp[R(i), C(j)]
    if dp[R(i + 1), C(j)] == cur:
        i += 1
    elif dp[R(i), C(j + 1)] == cur:
        j += 1
    else:
        out.push(a[i])
        i += 1
        j += 1
print(Str(out))
