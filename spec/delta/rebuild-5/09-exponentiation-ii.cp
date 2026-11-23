// CSES 1712 Exponentiation II: a^(b^c) mod 1e9+7 for n queries.
// Input: n, then n lines "a b c". Output: one answer per line.
type Z = Mod<1e9 + 7>

@!multitest
let (a, b, c) = read<(0..=1e9, 0..=1e9, 0..=1e9)>()
if a == 0:
    print(if b == 0 and c > 0: 1 else: 0)       // 0^(b^c): b^c is 0 only when b = 0 < c
else:
    print(Z(a) ** pow_mod(b, c, Z.MOD - 1))      // Fermat: a^(P-1) = 1 for a != 0
