use std.mod.{Mod}
const P = 1_000_000_007
type Z = Mod<P>
let comb = Comb<Z>()

@!multitest
let (n, m, q): (1..=2e5, 1..=2e5, 1..=2e5) = read()
let a: [0..n]-1e9..=1e9 = read()
let pre = a.scan(+)
let total = pre.cut(n)
var pos: Map<Int, List<Int>> = Map()
for s in 1..n:
    (pos[pre.cut(s) + pre.cut(n - s) - total] ??= []).push(s)
fn count_in(w: Int, l: Int, r: Int) -> Int:
    let xs = pos.get(w) ?? []
    (xs.first_gt(r) ?? xs.len) - (xs.first_ge(l) ?? xs.len)
let zeros = count_in(0, 1, n - 1)
for _ in 0..q:
    let (i, v): (1..=n, -1e9..=1e9) = read()
    var c = zeros
    if v != 0:
        let (l, r, delta) = if i <= n - i: (i, n - i, v) else: (n - i + 1, i - 1, -v)
        if l <= r:
            c += count_in(-delta, l, r) - count_in(0, l, r)
    print(comb.choose(c, m - 1))
