// CF 1925D Good Trip: each excursion picks a friend pair with probability 1/P; by linearity
// the answer is k*sum(f)/P + m * C(k, 2) / P^2 (mod 1e9+7). k*sum(f) overflows Int.
type Z = Mod<1000000007>

@!multitest
let (n, m, k) = read<(2..=1e5, 0..=1e5, 1..=2e5)>()
let pairs = read<[m](a: 1..=n, b: 1..=n, f: 1..=1e9)>()
let p = Z(1) / Z(n * (n - 1) / 2)
let base = pairs.sum(of: e => Z(e.f))
print(Z(k) * base * p + Z(m) * Z(k * (k - 1) / 2) * p * p)
