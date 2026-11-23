type Z = Mod<1_000_000_007>
let comb = Comb<Z>()
@!multitest
let (n, m, q): (1..=2e5, 1..=n, 1..=2e5) = read()
let a: [0..n]-1e9..=1e9 = read()
var pre: [0..=n]Int = 0
for i in 0..n: pre[i + 1] = pre[i] + a[i]
let total = pre[n]
var zeros: [0..=n]Int = 0
var keyed: List<(Int, Int)> = []
for s in 1..n:
    let dv = pre[s] + pre[n - s] - total
    zeros[s] = zeros[s - 1] + (if dv == 0: 1 else: 0)
    keyed.push((dv, s))
let byval = keyed.sorted()
fn zeros_in(lo: Int, hi: Int) -> Int:
    let (l, h) = (max(lo, 1), min(hi, n - 1))
    if l > h: return 0
    return zeros[h] - zeros[l - 1]
fn count_in(x: Int, lo: Int, hi: Int) -> Int:
    let (l, h) = (max(lo, 1), min(hi, n - 1))
    if l > h: return 0
    let from = byval.first_ge((x, l)) ?? byval.len
    let to = byval.first_gt((x, h)) ?? byval.len
    return to - from
for _ in 0..q:
    let (i, v): (1..=n, -1e9..=1e9) = read()
    var good = zeros_in(1, n - 1)
    if v != 0:
        good -= zeros_in(i, n - i) + zeros_in(n - i + 1, i - 1)
        good += count_in(-v, i, n - i) + count_in(v, n - i + 1, i - 1)
    print(comb.choose(good, m - 1))
