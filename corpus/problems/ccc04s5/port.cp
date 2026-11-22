// CCC 2004 S5, Super Plumber (sentinel-terminated 0 0 multi-test + mixed cell alphabet, buildable today-form)
// original: corpus/submissions/4806811.cpp
fn solve_case(rows: Int, cols: Int):
    domain R = 0..rows
    domain C = 0..cols
    let grid: [R, C]Char = read()
    let neg_inf = -1000000000

    var dp: [R]Int = neg_inf
    dp[R(rows - 1)] = 0

    var c = cols - 1
    while c >= 0:
        var nxt: [R]Int = neg_inf
        for start in 0..rows:
            var gain = 0
            for r in start..rows:
                let ch = grid[R(r), C(c)]
                if ch == '*':
                    break
                if ch >= '0' and ch <= '9':
                    gain += ch - '0'
                if dp[R(r)] > neg_inf:
                    nxt[R(start)] max= gain + dp[R(r)]
            gain = 0
            var r = start
            while r >= 0:
                let ch = grid[R(r), C(c)]
                if ch == '*':
                    break
                if ch >= '0' and ch <= '9':
                    gain += ch - '0'
                if dp[R(r)] > neg_inf:
                    nxt[R(start)] max= gain + dp[R(r)]
                r -= 1
        dp = nxt
        c -= 1
    print(dp[R(rows - 1)])

while true:
    let m: 0..=100 = read()
    let n: 0..=100 = read()
    if m == 0 and n == 0:
        break
    solve_case(m, n)
