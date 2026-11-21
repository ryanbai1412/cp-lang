@!multitest
let (n0, m0): (Int, Int) = read()
let rows: [0..n0]Str = read()
let tr = n0 > m0
let (n, m) = if tr: (m0, n0) else: (n0, m0)
let g = [0..n, 0..m] (i, j) => (if tr: rows[j][i] else: rows[i][j]) == '1'

let cap = n * m
var ans: [0..n, 0..m]Int = cap + 1
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

fn clean(v: Int) -> Int = if v > cap: 0 else: v
for i in 0..n0:
    print([clean(if tr: ans[j, i] else: ans[i, j]) for j in 0..m0])
