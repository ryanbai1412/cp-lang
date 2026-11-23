// Codeforces 2159 B — Rectangles
//
// For each cell (i, j), find the minimum area of an axis-aligned rectangle
// containing (i, j) whose four corners are all '1' (or 0 if none exists).
//
// Transpose if n0 > m0 so n <= m (hence n <= sqrt(n * m) and O(n^2 * m) is
// O((n * m)^(3/2))). Iterate rectangle height l from min(n, m) down to 2 and
// top row i0 from n - l down to 0: between consecutive columns j0 < j1 where
// both g[i0, j] and g[i0 + l - 1, j] are '1', the rectangle [i0..=i1] x
// [j0..=j1] has area `l * (j1 - j0 + 1)`, updating `ans[i0, j0..=j1]`.
// Propagating `ans[i0 + 1, ..] min= ans[i0, ..]` after each height-l row
// covers every interior row of all rectangles of height >= l in O(n * m).

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
