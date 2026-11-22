// original: 3997206 (cmt21r1xym)
const MOD = 998244353
type Z = Mod<MOD>
let n: Int = read()
domain I = 0..n
let g: [I, I]Char = read()
var ncr: [0..=n, 0..=n]Z = Z.zero
for i in 0..=n:
    ncr[i, 0] = 1
    for j in 1..=i: ncr[i, j] = ncr[i - 1, j - 1] + ncr[i - 1, j]
domain K = 0..=n
fn solve(cnt: [I]Int, mut best: [K]Int, mut ways: [K]Z):
    let a = cnt.sorted(by: x => -x)
    let total = a.sum()
    var top = 0
    for i in 0..=n:
        if i > 0: top += a[i - 1]
        best[K(i)] = top + n * (n - i) - (total - top)
        ways[K(i)] = 1
        if i > 0 and i < n:
            let c = a[i - 1]
            let lo = sum(1 for x in a if x > c)
            let hi = sum(1 for x in a if x >= c)
            ways[K(i)] = ncr[hi - lo, i - lo]
let rows: [I]Int = [I] i => sum(1 for j in I if g[i, j] == '1')
let cols: [I]Int = [I] j => sum(1 for i in I if g[i, j] == '1')
var ma: [K]Int = 0
var wa: [K]Z = 0
var mb: [K]Int = 0
var wb: [K]Z = 0
solve(rows, mut ma, mut wa)
solve(cols, mut mb, mut wb)
var ans = Z.zero
for xa in 0..=n:
    for xb in 0..=n:
        if ma[K(xa)] + mb[K(xb)] - xa * xb - (n - xa) * (n - xb) == n * n: ans += wa[K(xa)] * wb[K(xb)]
print(ans)
