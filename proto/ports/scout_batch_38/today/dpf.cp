// AtCoder DP F (LCS): reconstruct a longest common subsequence of two strings.
// Runs on proto/cpi (`proto/cp.py lib-run`) because proto/cpc cannot index Str.
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
            v = max(v, 1 + dp[R(i + 1), C(j + 1)])
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
