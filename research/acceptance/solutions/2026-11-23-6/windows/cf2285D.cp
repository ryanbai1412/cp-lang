type Z = Mod<1_000_000_007>
let comb = Comb<Z>()

@!multitest
let n: 1..=2e5 = read()
let m: 1..=n = read()
let q: 1..=2e5 = read()
let a: [0..n]Int = read()
let qs: [0..q](i: 1..=n, v: Int) = read()
var pre: [0..=n]Int = 0
for i in 0..n:
    pre[i + 1] = pre[i] + a[i]
let total = pre[n]
var pos: Map<Int, List<Int>> = Map()
var zeros = 0
for s in 1..n:
    let d = pre[s] + pre[n - s] - total
    (pos[d] ??= []).push(s)
    if d == 0: zeros += 1
fn count_in(x: Int, l: Int, r: Int) -> Int:
    if l > r: return 0
    let ps = pos.get(x) ?? []
    ps.bsearch(p => p > r) - ps.bsearch(p => p >= l)
for (i, v) in qs:
    var (l, r, target) = (i, n - i, -v)
    if i > n - i:
        (l, r, target) = (n - i + 1, i - 1, v)
    let cnt = zeros - count_in(0, l, r) + count_in(target, l, r)
    print(comb.choose(cnt, m - 1))

@alt(program)
fn slow():
    let cb = Comb<Z>()
    for (i, v) in qs:
        var b = a
        b[i - 1] += v
        var p: [0..=n]Int = 0
        for j in 0..n:
            p[j + 1] = p[j] + b[j]
        var good = 0
        for s in 1..n:
            if p[s] + p[n - s] == p[n]: good += 1
        print(cb.choose(good, m - 1))
