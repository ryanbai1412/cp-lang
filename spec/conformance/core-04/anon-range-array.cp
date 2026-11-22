// expect: ok, output "7"
// rule: 04.8.2
var dp: [0..=5]Int = 0
let i = 3
dp[i + 1] = 7
print(dp[4])
