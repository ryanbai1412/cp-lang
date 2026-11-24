// Blocker repro partition-point-copies-list (run 5, Windows draft): the shipped solution with
// its two `first_true` searches written as `keyed.partition_point`. Over 25 s on max.py
// before 1b76f05c4; must stay AC.
// D. Reverse Partition
// https://codeforces.com/contest/2285/problem/D
type Z = Mod<1_000_000_007>
let comb = Comb<Z>()
@!multitest
let (n, m, q): (1..=2e5, 1..=2e5, 1..=2e5) = read()
let a: [0..n]Int = read()
let pre = prefix(a)
let total = a.sum()
// d[s] for cut s in 1..n-1: prefix(s) + prefix(n-s) - total; 0 means the cut matches its mirror
let keyed = [(pre.cut(s) + pre.cut(n - s) - total, s) for s in 1..n].sorted()
fn count(d: Int, lo: Int, hi: Int) -> Int:
    if lo > hi: return 0
    let len = keyed.len
    let r = keyed.partition_point(p => p > (d, hi))
    let l = keyed.partition_point(p => p >= (d, lo))
    r - l
for _ in 0..q:
    let (i, v): (1..=n, Int) = read()
    var cnt = count(0, 1, n - 1)
    if v != 0:
        let (lo, hi, want) = if i <= n - i: (i, n - i, -v) else: (n - i + 1, i - 1, v)
        cnt += count(want, lo, hi) - count(0, lo, hi)
    print(comb.choose(cnt, m - 1))
