// lsp: hover
use std.math.{splitmix64}
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
    if xs[V.first!] < 0:
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
    if xs[V.first!] < c:
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
        grid[r, Col.first!] += xs$0[v]
print(grid[Row.first!, C$0ol.last!])
fn sc$0an():
    for v in V:
        for j in 0..k:
            print(v, j)
sca$0n()
fn re$0scan():
    for i in 0..3:
        scan()
fn ra$0nked(ys: List<Int>) -> Int:
    for v in V:
        print(v, ys.sorted().len)
    let f = (x: Int) => ys.sorted().len + x
    ys.so$0rted().len + f(1)
let pts: [0..k]Int = read()
fn ord$0er(cells: [V]Int):
    for j in 0..k:
        print(pts.sorted().len, cells.so$0rted().len)
print(pts.so$0rted().len)
fn reor$0der(cells: [V]Int):
    for j in 0..k:
        order(cells)
print(splitmi$0x64(U64(n)))
fn coll$0ect() -> Int:
    var seen: List<Int> = []
    for v in V:
        for j in 0..k:
            seen.push(j)
    seen.so$0rted().len
fn tal$0ly(cells: [V]Int) -> Int:
    var best = 0
    for v in V:
        best max= cells[v]
    cells.sorted().len + best

fn sp$0read(cells: [V]Int) -> Int:
    var least = 0
    for v in V:
        least = min(least, abs(cells[v]))
    max(least, 1)

struct Span:
    lo: Int
    hi: Int

fn wi$0dest(cells: [V]Int) -> Span:
    var cur = Span(0, 0)
    for v in V:
        cur = Span(min(cur.lo, cells[v]), max(cur.hi, cells[v]))
    cur

fn ec$0ho():
    for _ in 0..n:
        let (x, y): (Int, Int) = read()
        print(x + y, "and {x * y}")
fn pre$0fixes(bit: Fenwick<V, Int>):
    for v in V:
        print(bit.f$0old(..v))
fn sh$0ow(cells: [V]Int):
    for _ in 0..k:
        p$0rint(cells)
fn ga$0ther():
    var got: List<Int> = []
    for i in 0..k:
        got.push(i * i)
        print(i)
    p$0rint("!", got)
struct Ed$0ge:
    to: V
    w: Int
    used: Bool
let e$0s: [V]Edge = [V] v => Edge(v, 0, false)
let fl$0ag: [V]?Bool = None
let pa$0irs: [V](V, Int) = [V] v => (v, 0)
let ad$0j: [V]List<V> = [V] v => []
struct Ru$0ns:
    first: [V]Int
    seen: List<V>
let na$0mes: [V](name: Str, t: 1..=1000) = [V] v => (name: "", t: 1)
let di$0sts: [V][V]?Int = [V] u => [V] v => None
print(es[V.first!].to, es[V.first!].w, yes(es[V.first!].used), yes(flag[V.first!] ?? false), pairs, adj, names, dists[V.first!][V.first!] ?? 0, Runs([V] v => 0, []).seen)
let word = "aab"
for r in word.ru$0ns(): print(r.len, word.mi$0n()!)
