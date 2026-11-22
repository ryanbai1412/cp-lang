// Codeforces 2084 D — Arcology On Permafrost
//
// Construct `a[0..n]` maximizing `MEX(a)` after an adversary removes `m`
// disjoint subarrays of length `k`. Every value `< MEX` must survive all `m`
// removals, which requires at least `m + 1` copies spaced at stride `>= k` so
// no single length-`k` removal hits two copies.
//
// Visiting residues `r = (n - 1 - i) % k` for `i in 0..k` processes the longest
// residue classes first (each residue class has spacing `k`). Filling all
// indices in this order with `cur / (m + 1)` assigns `m + 1` copies to each
// value `0, 1, ...` at mutual distance `>= k`, achieving the optimal MEX.

@!multitest
let (n, m, k): (2..=200_000, 1..=200_000, 1..=200_000) = read()
var a: [0..n]Int = 0
var cur = 0
for i in 0..k:
    for v in (n - 1 - i) % k..n by k:
        a[v] = cur / (m + 1)
        cur += 1
print(a)
