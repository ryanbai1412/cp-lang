// cf1925d Good Trip: a friend pair is picked c ~ Bin(k, p) times, p = 1 / C(n, 2).
// It contributes f * E[c] + E[c (c - 1) / 2]; the binomial sum shares one Comb.
type Z = Mod<1_000_000_007>
let comb = Comb<Z>(2e5)
@!multitest
let (n, m, k): (2..=1e5, 0..=1e5, 1..=2e5) = read()
let friends = read<[m](a: 1..=n, b: 1..=n, f: 1..=1e9)>()
let p = Z(n * (n - 1) / 2).inv()
let q = 1 - p
let repeats = (0..=k).map(c => comb.choose(k, c) * p ** c * q ** (k - c) * (c * (c - 1) / 2)).sum()
print(friends.map(.f).sum() * k * p + m * repeats)
