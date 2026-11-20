// cp: 0.1 bea787c+dirty
domain Row = 0..2
domain Col = 0..3
var dp: [Row, Col]Int = 0
let p = (Row(1), Col(2))
dp[p] = 5
print(dp[Row(1), Col(2)])
