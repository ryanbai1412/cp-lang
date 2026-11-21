use std.dp.{knuth}
let n: 1..=3000 = read()
let a: [0..n]1..=1e9 = read()
var pre = [0]
for x in a: pre.push(pre.last! + x)
let dp = knuth(n, (l, r) => pre[r] - pre[l])
print(dp[0][n])
