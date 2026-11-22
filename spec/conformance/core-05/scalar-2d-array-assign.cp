// expect: ok, output "0"
// rule: R05.1
domain I = 1..=3
var dp: [I.cuts, 0..=4]Int = 0
for i in I:
    for c in 0..=4:
        dp[i.after, c] = dp[i.before, c]
print(dp[I.end, 2])
