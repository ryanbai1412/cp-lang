// AtCoder DP F (LCS): reconstruct a longest common subsequence of two ASCII
// strings using opt-in nominal character domains and boundary-indexed DP.
// original: corpus/submissions/4131645.cpp
let a: [domain A]Char = read()
let b: [domain B]Char = read()

var dp: [A.bounds, B.bounds]Int = 0
for i in A.rev:
    for j in B.rev:
        dp[i.l, j.l] = max(dp[i.r, j.l], dp[i.l, j.r])
        if a[i] == b[j]:
            dp[i.l, j.l] max= 1 + dp[i.r, j.r]

var out = ""
var i: ?A = A.first
var j: ?B = B.first
// FRICTION 38.4: `while let ci = i and let cj = j` extends decisions.md's
// `if let ... and let ...`, but neither form is on the one-page cheat sheet.
while let ci = i and let cj = j:
    let cur = dp[ci.l, cj.l]
    if dp[ci.r, cj.l] == cur: i = ci + 1
    elif dp[ci.l, cj.r] == cur: j = cj + 1
    else:
        out.push(a[ci])
        (i, j) = (ci + 1, cj + 1)
print(out)
