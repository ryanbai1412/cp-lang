// Source: CCC 2004 Senior #5, Super Plumber, https://dmoj.ca/problem/ccc04s5
// limits: 1000 ms, 256 MB
// Problem: multitest, ending with "0 0": an m x n grid (2 <= m, n <= 100) of '*' obstacles,
//   coins '1'..'9' and '.'. Moving only right, up or down, never revisiting a cell, go from the
//   bottom-left to the bottom-right corner; print the largest total of coins collected.
// Solution: A path enters each column once and runs straight up or down in it before moving
//   right. Go column by column from the left: here[r] is the most collected on entering the
//   column at row r (None if no path gets there); from each reachable row walk up and down
//   until '*' or the edge, and every row passed is a place to leave the column. Start with
//   only the bottom row reachable; the answer is here[bottom] past the last column.
//   O(m^2 n) per test.
// original: corpus/submissions/4806811.cpp
// Bounds: rows, cols: 0..=100
fn value(ch: Char) -> Int = if '0' <= ch <= '9': ch - '0' else: 0

while true:
    let rows: 0..=100 = read()
    let cols: 0..=100 = read()
    if rows == 0 and cols == 0:
        break
    domain R = 0..rows
    domain Col = 0..cols
    let g: [R, Col]Char = read()
    var here: [R]?Int = None
    here[R.last!] = Some(0)
    for c in Col:
        var leave: [R]?Int = None
        for from in R:
            let base = here[from] else: continue
            for step in [-1, 1]:
                var (q, got) = (from.ord, base)
                while q in 0..rows and g[R(q), c] != '*':
                    got += value(g[R(q), c])
                    leave[R(q)] max= got
                    q += step
        here = leave
    print(here[R.last!]!)
