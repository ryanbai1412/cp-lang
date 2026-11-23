// cf1462e2 Close Tuples (hard): sort; for each maximum a[i], choose the other m - 1 from the
// earlier elements within k of it.
type Z = Mod<1_000_000_007>
let comb = Comb<Z>(2e5)

@!multitest
let n: 1..=2e5 = read()
let m: 1..=100 = read()
let k: 1..=2e5 = read()
let a = read<[n]1..=2e5>().sorted()
var lo = 0
var total = Z(0)
for i in 0..n:
    while a[i] - a[lo] > k: lo += 1
    total += comb.choose(i - lo, m - 1)
print(total)
