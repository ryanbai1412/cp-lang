// Source: COCI 2021/2022 Contest 6 (Zemljiste), https://dmoj.ca/problem/coci21c6p2
// limits: 2000 ms, 512 MB
// Problem: an r x s grid (r, s <= 500) of prices c <= 1e9, and a, b <= 1e9. A non-empty
//   subrectangle with total m costs |m - a| + |m - b|; print the smallest cost.
// Solution: the cost falls until min(a, b), is flat up to max(a, b), then rises, so with
//   positive prices, for each left end only two sums matter: the smallest >= min(a, b) and
//   the largest below it. Fix the top and bottom rows, collapse to column sums and sweep
//   with two pointers: O(r^2 s).
// Note: written in the forms proto/cpc builds today.
// original: corpus/submissions/4529998.cpp
let n: Int = read()
let m: Int = read()
let a: Int = read()
let b: Int = read()
let grid: [n, m]Int = read()
let lo = min(a, b)

var best: ?Int = None
fn offer(s: Int): best min= abs(s - a) + abs(s - b)

for top in 0..n:
    var col: [m]Int = 0
    for bot in top..n:
        for c in 0..m: col[c] += grid[bot, c]
        var r = 0
        var s = 0
        for l in 0..m:
            while r < m and s < lo:
                s += col[r]
                r += 1
            offer(s)
            if r - l >= 2: offer(s - col[r - 1])
            s -= col[l]
print(best!)
