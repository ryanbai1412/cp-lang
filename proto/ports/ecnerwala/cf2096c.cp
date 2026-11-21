const INF = 2_000_000_000_000_000_000

@! multitest
let n: Int = read()
var row_bad: [0..n - 1][0..3]Bool = false
var col_bad: [0..n - 1][0..3]Bool = false
var prev: [0..n]Int = -2
for i in 0..n:
    for j in 0..n:
        let h: Int = read()
        if i > 0:
            let d = h - prev[j]
            if -1 <= d and d <= 1:
                row_bad[i - 1][d + 1] = true
        if j > 0:
            let d = h - prev[j - 1]
            if -1 <= d and d <= 1:
                col_bad[j - 1][d + 1] = true
        prev[j] = h
let a: [0..n]Int = read()
let b: [0..n]Int = read()

fn min_cost(c: [0..n]Int, bad: [0..n - 1][0..3]Bool) -> ?Int:
    var dp = (0, c[0])
    for i in 1..n:
        var n0 = INF
        var n1 = INF
        if dp.0 < INF:
            if not bad[i - 1][1]:
                n0 min= dp.0
            if not bad[i - 1][0]:
                n1 min= dp.0 + c[i]
        if dp.1 < INF:
            if not bad[i - 1][2]:
                n0 min= dp.1
            if not bad[i - 1][1]:
                n1 min= dp.1 + c[i]
        dp = (n0, n1)
    let res = min(dp.0, dp.1)
    if res >= INF:
        return none
    res

if let ra = min_cost(a, row_bad), let rb = min_cost(b, col_bad):
    print(ra + rb)
else:
    print(-1)
