// expect: R0101
// rule: 04.8.2
var dp: [0..=5]Int = 0
let i = 3
dp[i + 3] = 7 // <- here
