// Matched benchmark: Comb<Mod<998244353>>(2_000_000) + 5_000_000 choose(n, k) queries.
type Z = Mod<998_244_353>

let lim = 2_000_000
let c = Comb<Z>(lim)
var s = 246813579
var acc = Z(0)
for _ in 0..5_000_000:
    s = (s * 1103515245 + 12345) & 2_147_483_647
    let n = s % (lim + 1)
    s = (s * 1103515245 + 12345) & 2_147_483_647
    let k = s % (n + 1)
    acc = acc + c.choose(n, k)
print(acc.v)
