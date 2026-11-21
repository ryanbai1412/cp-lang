// Codeforces 2115 A: Gellyfish and Flaming Peony (dream).
// Divide all elements by their overall gcd `g` so the target value becomes 1.
// If `cnt1 > 0` elements are already 1, each remaining element takes 1 op (`n - cnt1`).
// Otherwise, we first form a 1 from the smallest `k`-tuple with gcd 1 (`k - 1` ops),
// then turn the other `n - 1` elements into 1 (`k - 1 + n - 1` total).
// By Mobius inversion, the weighted count of `k`-tuples with gcd 1 is
//   sum_{d=1..vmax} mu[d] * (sum_{d | x} w[x])^k;
// random non-zero weights mod 998244353 (Schwarz-Zippel) prevent accidental zeroing.
use std.math.Sieve
use std.rand.Rng

const MAX_V = 5_000
type Z = Mod<998_244_353>

let mu = Sieve.new(MAX_V).mobius_table()
var rng = Rng.new(48)

@!multitest
let n: 1..=5_000 = read()
let raw: [0..n]1..=5_000 = read()
let g = raw.fold(gcd)
let a = [x / g for x in raw]
let cnt1 = a.count(1)
if cnt1 > 0:
    print(n - cnt1)
    return

let vmax = a.max()!
var freq: [0..=vmax]Z = 0
for x in a: freq[x] += rng.int(1, Z.MOD - 1)

let mult_sum = freq.mult_zeta()
var prod = [0..=vmax] d => Z(mu[d])
let k = (1..=vmax).first(k =>
    var total: Z = 0
    for d in 1..=vmax:
        prod[d] *= mult_sum[d]
        total += prod[d]
    total != 0
)!
print(k - 1 + n - 1)
