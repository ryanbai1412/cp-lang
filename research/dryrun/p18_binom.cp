// CSES 1079 Binomial Coefficients: n queries of C(a, b) mod 1e9+7.
use std.mod.{Mod}
type Z = Mod<1_000_000_007>
let comb = Comb<Z>()
let n: 1..=1e5 = read()
repeat n:
    let (a, b): (1..=1e6, 0..=1e6) = read()
    print(comb.choose(a, b))
