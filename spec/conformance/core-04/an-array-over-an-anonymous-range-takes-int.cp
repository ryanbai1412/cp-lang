// expect: ok, output "5 0"
// rule: 04.3.4
// R04.3.4: an array over an anonymous range counted from a number takes any `Int`, checked.
let n = 3
var dp: [1..=n]Int = 0
dp[2] = 5
print(dp[2], dp[3])
