// Source: Codeforces 2159 B (Rectangles), https://codeforces.com/contest/2159/problem/B
// limits: 4000 ms, 1024 MB
// Problem: an n x m binary grid (n, m >= 2). A rectangle uses two distinct rows and two distinct
//   columns with 1s at all four corners. For each cell print the least area of a rectangle
//   containing it, or 0. Multitest, sum of n m <= 2.5e5.
// Solution: transpose so n <= m. For each pair of rows i0 < i1, only consecutive columns where
//   both rows have a 1 matter: each such span updates row i0's cells with its area. Taking
//   heights from tall to short and pushing row i0's minima down to row i0 + 1 covers every
//   inner row. O(n^2 m) = O((n m)^1.5).

@!multitest
let (n0, m0): (2..=250_000, 2..=250_000) = read()
let rows: [0..n0]Str = read()
let tr = n0 > m0
let (n, m) = if tr: (m0, n0) else: (n0, m0)
let g = [0..n, 0..m] (i, j) => (if tr: rows[j][i] else: rows[i][j]) == '1'

var ans: [0..n, 0..m]?Int = None
for l in (2..=min(n, m)).rev:
    for i0 in (0..=n - l).rev:
        let i1 = i0 + l - 1
        var j0 = 0
        while j0 < m:
            if g[i0, j0] and g[i1, j0]:
                var j1 = j0 + 1
                while j1 < m and not (g[i0, j1] and g[i1, j1]): j1 += 1
                if j1 == m: break
                let area = l * (j1 - j0 + 1)
                for j in j0..=j1: ans[i0, j] min= area
                j0 = j1
            else:
                j0 += 1
        ans[i0 + 1, ..] min= ans[i0, ..]

for i in 0..n0:
    print([(if tr: ans[j, i] else: ans[i, j]) ?? 0 for j in 0..m0])
