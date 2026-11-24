// Matched benchmark: 25_000_000 Comb.inv(k) lookups with k in 1..=5000, as in cf2138d's inner loop.
type Z = Mod<1_000_000_007>

let c = Comb<Z>()
var s = 246813579
var acc = Z(0)
for _ in 0..25_000_000:
    s = (s * 1103515245 + 12345) & 2_147_483_647
    acc += Z(s) * c.inv(s % 5000 + 1)
print(acc.v)
