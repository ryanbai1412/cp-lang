type Z = Mod<1_000_000_007>
let cb = Comb<Z>()

@!multitest
let (n, m, q): (1..=2e5, 1..=2e5, 1..=2e5) = read()
let a: [0..n]-1e9..=1e9 = read()
let sc = a.scan(+)
let total = sc[n - 1]
fn pre(s: Int) -> Int = if s == 0: 0 else: sc[s - 1]
var pos = HashMap<Int, List<Int>>()
for s in 1..n:
    (pos[pre(s) + pre(n - s) - total] ??= []).push(s)

fn cnt(x: Int, lo: Int, hi: Int) -> Int:
    if lo > hi:
        return 0
    if let ps = pos.get(x):
        return ps.partition_point(p => p > hi) - ps.partition_point(p => p >= lo)
    0

let base = cnt(0, 1, n - 1)
for _ in 0..q:
    let (i, v): (1..=n, -1e9..=1e9) = read()
    let (l1, h1) = (max(i, 1), min(n - i, n - 1))
    let (l2, h2) = (max(n - i + 1, 1), min(i - 1, n - 1))
    let k = base - cnt(0, l1, h1) - cnt(0, l2, h2) + cnt(-v, l1, h1) + cnt(v, l2, h2)
    print(cb.choose(k, m - 1))
