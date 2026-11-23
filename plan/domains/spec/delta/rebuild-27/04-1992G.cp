// CF 1992G: sum of MEX(b, |b| + 1) over all subsets b of {1..n}.
// For |b| = k and answer m: m is absent, j = m - 1 - k members lie below m, k - j above.
type Z = Mod<1e9 + 7>
let comb = Comb<Z>(5000)

@!multitest
let n: 1..=5000 = read()
var total = Z(0)
for k in 0..=n, m in 1..=2 * k + 1:
    let j = m - 1 - k
    if j < 0 or j > k: continue
    let below = min(m - 1, n)
    let above = max(n - m, 0)
    total += Z(m) * comb.choose(below, j) * comb.choose(above, k - j)
print(total)
