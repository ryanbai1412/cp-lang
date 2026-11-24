// Source: DMOPC '20 Contest 5 P2 (On The Clock), https://dmoj.ca/problem/dmopc20c5p2
// limits: 2000 ms, 256 MB
// Problem: an N x M pixel grid (N, M <= 1e6), and the segment from its bottom-left to its
//   top-right corner. Print the number of pixels whose interior it crosses (touching a
//   corner does not count), then those pixels sorted by (row, column).
// Solution: within row r the line runs from x = m r / n to x = m (r + 1) / n, so it crosses
//   the interiors of columns floor(m r / n) up to ceil(m (r + 1) / n) - 1; a corner it only
//   touches falls outside that range. Rows come out in order, so no sort: O(n + m).
// original: corpus/submissions/3591332.cpp
// Bounds: n, m: 1..=1e6
let n: 1..=1e6 = read()
let m: 1..=1e6 = read()
domain Row = 0..n
let crossed = [Row] r => (m * Int(r) / n)..(m * (Int(r) + 1)).ceil_div(n)
print(sum(cols.len for cols in crossed))
for r in Row:
    for c in crossed[r]: print(Int(r) + 1, c + 1)
