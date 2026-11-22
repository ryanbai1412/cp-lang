// CSES 1079 Binomial Coefficients: n queries of C(a, b) mod 1e9+7.
type Z = Mod<1_000_000_007>
let comb = Comb<Z>.new()
let n = read<1..=1e5>()
for _ in 0..n:
    let (a, b) = read<(1..=1e6, 0..=1e6)>()
    print(comb.choose(a, b))
