// Source: Codeforces 2127F (Hamed and AghaBalaSar), https://codeforces.com/contest/2127/problem/F
// Problem: a snake array has n entries in 0..=m summing to m with a[n] a maximum; f(a) climbs
//   from position 1 by next-greater jumps adding each rise, stepping past earlier copies of
//   the maximum. Sum f over all snake arrays mod 1e9+7. n, m <= 2e5; multitest, sum m <= 2e5.
// Solution: the rises telescope: with mx = a[n], f(a) sums mx - a[s] over s = 1 and every s < n
//   right after an earlier copy of mx. Fix mx; by symmetry the first term totals
//   ways(n-1, m-mx) * (mx - average) and the n-2 interior copies add (n-2) * ways(n-2, m-2mx) *
//   (mx - average), where ways counts tuples in 0..=mx with a given sum by inclusion-exclusion
//   over entries above mx. All mx together cost O(m log m) (harmonic sum).

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
