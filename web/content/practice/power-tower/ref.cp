const P = 1_000_000_007
type Z = Mod<P>
@!multitest
let (a, b, c): (1..=1e9, 0..=1e9, 0..=1e9) = read()
print(Z(a) ** pow_mod(b, c, P - 1))
