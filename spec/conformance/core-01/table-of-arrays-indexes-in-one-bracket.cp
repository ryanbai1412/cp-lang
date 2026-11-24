// expect: ok, output "5 -1\n5\n7 0 7 7"
// rule: 01.8.5
// A table whose cells are arrays takes the cell's index in the same brackets, to read and to write.
domain K = 0..2
domain R = 0..2
type Cell = [K]?Int
var dp: [R, R]Cell = [K] _ => None
dp[R(0), R(1), K(1)] = Some(5)
print(dp[R(0), R(1), K(1)] ?? -1, dp[R(0), R(0), K(1)] ?? -1)
let cell = dp[R(0), R(1)]
print(cell[K(1)] ?? -1)
var cnt: [R, R][K]Int = [K] _ => 0
for r in R:
    for k in K:
        cnt[r, R(1), k] += 7
print(cnt[R(1), R(1), K(0)], cnt[R(1), R(0), K(0)], cnt[R(0), R(1)])
