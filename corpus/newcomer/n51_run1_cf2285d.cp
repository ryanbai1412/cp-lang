// First draft from acceptance run 1 (cold agent, docs only), Codeforces 2285D.
type Z = Mod<1_000_000_007>
let comb = Comb<Z>()

@!multitest
let (n, m, q): (1..=2e5, 1..=2e5, 1..=2e5) = read()
let a: [0..n]-1e9..=1e9 = read()
var ps: List<Int> = [0]  // expect: hand_rolled_prefix_sum
for x in a:
    ps.push(ps[ps.len - 1] + x)
fn diff(x: Int) -> Int = ps[x] - (ps[n] - ps[n - x])
var pos: Map<Int, List<Int>> = Map()
for x in 1..n:
    (pos[diff(x)] ??= []).push(x)  // expect: missing group-by-key-loop
fn cnt(val: Int, lo: Int, hi: Int) -> Int:
    if let l = pos.get(val):
        return (l.first_ge(hi) ?? l.len) - (l.first_ge(lo) ?? l.len)
    return 0
let base = cnt(0, 1, n)
for _ in 0..q:
    let (i, v): (1..=n, -1e9..=1e9) = read()
    let lo = min(i, n + 1 - i)
    let hi = max(i, n + 1 - i)
    var k = base
    if v != 0 and lo < hi:
        let s = if i == lo: 1 else: -1
        k += cnt(-s * v, lo, hi) - cnt(0, lo, hi)
    print(comb.choose(k, m - 1))
