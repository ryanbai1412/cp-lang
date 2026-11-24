use std.mod.{Mod}
type Z = Mod<1_000_000_007>
let comb = Comb<Z>()

@!multitest
let (n, m, q): (1..=2e5, 1..=2e5, 1..=2e5) = read()
let a: [0..n]-1e9..=1e9 = read()
let pre = Prefix(a)
let total = pre.prefix(n)
fn d(x: Int) -> Int = pre.prefix(x)
var at: HashMap<Int, List<Int>> = HashMap()
for x in 1..n:
    let h = d(x) + d(n - x) - total
    if h in at: at[h].push(x)
    else: at[h] = [x]
fn count(h: Int, lo: Int, hi: Int) -> Int:
    if lo > hi: return 0
    let ps = at.get(h) ?? []
    return (ps.first_gt(hi) ?? ps.len) - (ps.first_ge(lo) ?? ps.len)
for _ in 0..q:
    let (i, v): (1..=n, -1e9..=1e9) = read()
    let (blo, bhi) = (i, n - i)
    let (nlo, nhi) = (n - i + 1, i - 1)
    let zeros = count(0, 1, n - 1) - count(0, blo, bhi) - count(0, nlo, nhi)
    let g = zeros + count(-v, blo, bhi) + count(v, nlo, nhi)
    print(comb.choose(g, m - 1))
