@!multitest
let (n0, m0): (Int, Int) = read()
let rows: [0..n0]Str = read()
let tr = n0 > m0
let n = if tr: m0 else: n0
let m = if tr: n0 else: m0

var g: [0..n, 0..m]Int = 0
for i in 0..n0:
    for j in 0..m0:
        let v = Int(rows[i][j] == '1')
        if tr: g[j, i] = v else: g[i, j] = v

let inf = n * m + 1
var ans: [0..n, 0..m]Int = inf

for l in (2..=m).rev:
    if l <= n:
        for i0 in (0..=(n - l)).rev:
            let i1 = i0 + l - 1
            var j0 = 0
            while j0 < m:
                if g[i0, j0] == 1 and g[i1, j0] == 1:
                    var j1 = j0 + 1
                    while j1 < m and not (g[i0, j1] == 1 and g[i1, j1] == 1):
                        j1 += 1
                    if j1 == m: break
                    let area = l * (j1 - j0 + 1)
                    for j in j0..=j1:
                        ans[i0, j] min= area
                    j0 = j1
                else:
                    j0 += 1
            for j in 0..m:
                ans[i0 + 1, j] min= ans[i0, j]

fn clean(v: Int, cap: Int) -> Int = if v > cap: 0 else: v

for i in 0..n0:
    print([clean(if tr: ans[j, i] else: ans[i, j], n * m) for j in 0..m0])
