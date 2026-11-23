// Source: DMOPC '21 Contest 7 P1 (Chika Grids), https://dmoj.ca/problem/dmopc21c7p1
// Problem: an N x M grid (N, M <= 1500) of values 0..=1e9, 0 meaning blank. Fill the blanks so
//   every row and every column is strictly increasing with all values in 1..=1e18, keeping
//   the given cells, and print the grid; or print -1 if impossible.
// Solution: give each blank the largest value it can hold: going from the bottom-right, one
//   less than the smaller of the cells below and to its right (1e17 past the border). That
//   leaves the most room for the cells above and to the left, so if this grid fails a final
//   check (positive, at most 1e18, rows and columns increasing) every filling fails. O(NM).
// Runnable on proto/cpc today.
// original: corpus/submissions/4452959.cpp
let n: 1..=1000 = read()
let m: 1..=1000 = read()
domain Row = 0..n
domain Col = 0..m
let given: [Row, Col]Int = read()
const TOP = 100000000000000000  // 1e17: room below for n + m blanks

var a: [Row, Col]Int = 0
for i in 0..n:
    let r = Row(n - 1 - i)
    for j in 0..m:
        let c = Col(m - 1 - j)
        if given[r, c] != 0:
            a[r, c] = given[r, c]
            continue
        var v = TOP
        if let below = Row.at(r + 1): v = a[below, c]
        if let right = Col.at(c + 1): v min= a[r, right]
        a[r, c] = v - 1

fn valid() -> Bool:
    for r in Row:
        for c in Col:
            if a[r, c] <= 0 or a[r, c] > 1000000000000000000: return false
            if let right = Col.at(c + 1):
                if a[r, c] >= a[r, right]: return false
            if let below = Row.at(r + 1):
                if a[r, c] >= a[below, c]: return false
    true

if valid():
    for r in Row: print(a[r, ..])
else:
    print(-1)
