// Runnable port (proto/cp.py lib-run) of statement-first aac5p3.cp
// original: corpus/submissions/4239862.cpp
let (n, m): (Int, Int) = read()
var adj: List<List<(w: Int, v: Int)>> = [[] for _ in 0..n]
for _ in 0..n - 1:
    let (u, v, w): (Int, Int, Int) = read()
    adj[u - 1].push((w: w, v: v - 1))
    adj[v - 1].push((w: w, v: u - 1))
var xs: List<Int> = []
for _ in 0..m:
    let x: Int = read()
    xs.push(x)
sort(mut xs)

// Number of values strictly below x.
fn below(x: Int) -> Int:
    var (lo, hi) = (0, m)
    while lo < hi:
        let mid = (lo + hi) / 2
        if xs[mid] >= x: hi = mid
        else: lo = mid + 1
    return lo

var ans = List.filled(n, 0)

// Values in [l, r) reach u; each child edge keeps the values closest to its weight.
fn dfs(u: Int, p: Int, l: Int, r: Int):
    if l >= r: return
    ans[u] = below(r) - below(l)
    var kids: List<(w: Int, v: Int)> = []
    for e in adj[u]:
        if e.v != p: kids.push(e)
    sort(mut kids, by: e => e.w)
    for i in 0..kids.len:
        let w = kids[i].w
        let cl = if i > 0: (kids[i - 1].w + w) / 2 + 1 else: l
        let cr = if i + 1 < kids.len: (w + kids[i + 1].w) / 2 + 1 else: r
        dfs(kids[i].v, u, max(cl, l), min(cr, r))

dfs(0, -1, 0, 1000000005)
for x in ans: print(x)
