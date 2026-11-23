// CSES 1712 Exponentiation II: a^(b^c) mod 1e9+7 for many tests.
// Input: n, then n lines "a b c". Output: one value per line.
const P = 1e9 + 7
type Z = Mod<P>

@!multitest
let (a, b, c) = read<(0..=1e9, 0..=1e9, 0..=1e9)>()
print(Z(a) ** pow_mod(b, c, P - 1))     // Fermat: P is prime and a < P
