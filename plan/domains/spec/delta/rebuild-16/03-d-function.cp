// Codeforces 1985G D-Function: count n in [10^l, 10^r) with D(k n) = k D(n), mod 1e9+7.
// In: t; per test `l r k`. Out: the count mod 1e9+7 per test.
type Z = Mod<1e9 + 7>

@!multitest
let l, r: 0..=1e9 = read()
let k = read<1..=1e9>()
// D(k n) = k D(n) exactly when no digit carries: every digit d has d * k <= 9.
let choices = Z(9 / k + 1)
print(choices ** r - choices ** l)
