// Source: CCC 2004 Senior #5, Super Plumber, https://dmoj.ca/problem/ccc04s5
// Problem: multitest, ending with "0 0": an m x n grid (2 <= m, n <= 100) of '*' obstacles,
//   coins '1'..'9' and '.'. Moving only right, up or down, never revisiting a cell, go from the
//   bottom-left to the bottom-right corner; print the largest total of coins collected.
// Solution: A path enters each column once and runs straight up or down in it before moving
//   right. Go column by column from the right: dp[r] is the best total from entering column c
//   at row r, the max over rows reachable straight up or down without crossing '*' of the coins
//   on the way plus the next column's dp there (a virtual column past the end allows only the
//   bottom row). O(m^2 n) per test.
// Buildable today-form.
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
                if '0' <= ch <= '9':
                    gain += ch - '0'
                if dp[R(r)] > neg_inf:
                    nxt[R(start)] max= gain + dp[R(r)]
            gain = 0
            var r = start
            while r >= 0:
                let ch = grid[R(r), C(c)]
                if ch == '*':
                    break
                if '0' <= ch <= '9':
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
