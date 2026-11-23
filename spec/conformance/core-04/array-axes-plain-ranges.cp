// expect: ok, output "7 0"
// rule: 04.2.25
// An array's axes may be plain ranges, indexed like a product of domains (rebuild round 21 H1).
let n = 3
let k = 2
var dp: [0..=n, 0..=k]Int = 0
dp[3, 2] = 7
print(dp[3, 2], dp[0, 0])
