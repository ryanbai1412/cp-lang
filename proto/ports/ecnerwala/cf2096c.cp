@!multitest
let n: Int = read()
let h: [0..n, 0..n]Int = read()
let a: [0..n]Int = read()
let b: [0..n]Int = read()

var row_bad: [0..n - 1, 0..3]Bool = false
var col_bad: [0..n - 1, 0..3]Bool = false
for i in 0..n:
    for j in 0..n:
        if i > 0:
            let d = h[i, j] - h[i - 1, j]
            if -1 <= d and d <= 1: row_bad[i - 1, d + 1] = true
        if j > 0:
            let d = h[i, j] - h[i, j - 1]
            if -1 <= d and d <= 1: col_bad[j - 1, d + 1] = true

fn min_cost(c: [0..n]Int, bad: [0..n - 1, 0..3]Bool) -> ?Int:
    var dp: [0..2]?Int = [0, c[0]]
    for i in 1..n:
        var nxt: [0..2]?Int = none
        for p in 0..2:
            if let cur = dp[p]:
                for q in 0..2 if not bad[i - 1, 1 + p - q]:
                    nxt[q] min= cur + q * c[i]
        dp = nxt
    min(dp[0], dp[1])

if let ra = min_cost(a, row_bad), let rb = min_cost(b, col_bad): print(ra + rb)
else: print(-1)
