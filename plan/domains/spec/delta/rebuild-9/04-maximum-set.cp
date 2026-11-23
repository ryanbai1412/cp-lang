// CF 1796C Maximum Set
// in: t; per test l r. out: max size of a beautiful set in [l, r] and the number of such sets mod 998244353.
type Z = Mod<998_244_353>

@!multitest
let l, r: 1..=1e6 = read()
// longest chain doubles every step: l * 2^(k-1) <= r
let k = (0..=20).last_true(j => l * 2 ** j <= r)! + 1
let doubling = r / 2 ** (k - 1) - l + 1
// one step may triple instead, at any of the k - 1 steps
let one_triple = if k >= 2: (k - 1) * max(0, r / (3 * 2 ** (k - 2)) - l + 1) else: 0
print(k, Z(doubling + one_triple))
