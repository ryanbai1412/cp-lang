// lsp: hover
let n: 1..=2e5 = read()
domain V = 1..=n
let xs: [V]Int = read()
let tot$0al = xs.s$0um()
let k = n % 7 + 1
f$0or v$0 in V:
    let ha$0lf = n / 2
    print(half, total, k$0, xs.l$0en)
let area = n $0* n + n
let wide = area $0* area * area
let ys = xs.map(n => n $0* 2)
let back = (n - 1) $0** 2 - area $0/ 3
let slot = (area $0% 1000) - n $0% 2
var be$0st = 0
for v in V:
    best max= min(n, 50) $0* 2
print(area, wide, ys, back, slot, be$0st)
if n > 50:
    print(n $0- 50)
else:
    print(n $0- 10)
var lo = 0
var hi = n
while lo < hi:
    let m$0id = (lo + hi) / 2
    if xs[V.first] < 0:
        lo = mid + 1
    else:
        hi = mid
print(l$0o, lo $0- hi)
var w = n
while w > 9:
    w = w / 2
let sm$0all = w * 3
print(small)
for v in V:
    f$0or j in 0..k:
        print(v, j)
var a = 0
var b = n
while a < b:
    let c = (a + b) / 2
    print(b $0- c)
    if xs[V.first] < c:
        a = c + 1
    else:
        b = c
fn count(zs: List<Int>, p: Int) -> Int:
    var (left, right) = (0, zs.len)
    while left < right:
        let at = (left + right) / 2
        if zs$0[a$0t] < p: left = at + 1
        else: right = at
    zs.len - left
print(count([1, 2], 2))
fn sum_rows(g: List<List<Int>>) -> Int:
    var s = 0
    for u in 0..g.len:
        for j in 0..g[u].len:
            s += g[u]$0[j]
    s
print(sum_rows([[1], [2, 3]]))
domain Row = 0..3
domain Col = 0..4
var grid: [Row, Col]Int = 0
for v in V:
    for r in Row:
        grid[r, Col.first] += xs$0[v]
print(grid[Row.first, C$0ol.last])
