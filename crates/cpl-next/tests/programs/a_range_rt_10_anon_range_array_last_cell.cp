// expect: ok, output "7 0"
// rule: R04.3.4
// input: "3\n"
let n: Int = read()
var dp: [0..=n]Int = 0
dp[n] = 7
print(dp[n], dp[0])
