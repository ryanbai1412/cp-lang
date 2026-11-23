// Codeforces 2127 F — Hamed and AghaBalaSar
//
// `ways(k, rem, mx)` counts `k`-tuples in `0..=mx` summing to `rem` via
// inclusion–exclusion over `c` entries exceeding `mx`:
// `sum_{c=0}^{min(k, rem / (mx + 1))} (-1)^c * C(k, c) * multichoose(rem - c * (mx + 1), k)`.
//
// Fix `mx = a[n - 1]` (the maximum value). Summing `f(a)` over all arrays
// `a[0..n]` summing to `m` with `a[n - 1] = mx` splits into two terms:
// - Contribution from `a[n - 1]` against the `n - 1` other entries (average
//   `(m - mx) / (n - 1)`): `ways(n - 1, m - mx, mx) * (mx - (m - mx) / (n - 1))`.
// - When `n >= 3` and `2 * mx <= m`, each of the `n - 2` interior positions
//   also achieving `mx` leaves `n - 2` entries summing to `m - 2 * mx` (average
//   `(m - 2 * mx) / (n - 2)`), adding
//   `ways(n - 2, m - 2 * mx, mx) * (mx - (m - 2 * mx) / (n - 2)) * (n - 2)`.
// Total work is `O(m log m)` by harmonic sum over `mx`.

type Z = Mod<1_000_000_007>
let comb = Comb<Z>()

fn ways(k: Int, rem: Int, mx: Int) -> Z:
    var res: Z = 0
    for c in 0..=min(k, rem / (mx + 1)):
        let term = comb.choose(k, c) * comb.multichoose(rem - c * (mx + 1), k)
        if c % 2 == 1: res -= term else: res += term
    res

@!multitest
let (n, m): (2..=200_000, 0..=200_000) = read()
let inv_n1 = Z(n - 1).inv()
let inv_n2: Z = if n >= 3: Z(n - 2).inv() else: 0

var ans: Z = 0
for mx in 0..=m:
    ans += ways(n - 1, m - mx, mx) * (Z(mx) - Z(m - mx) * inv_n1)
    if n >= 3 and 2 * mx <= m:
        ans += ways(n - 2, m - 2 * mx, mx) * (Z(mx) - Z(m - 2 * mx) * inv_n2) * Z(n - 2)
print(ans)
