// DMOPC '21 Contest 4 P4 — sum over i of C(n, i) * (i-1)! / (n-1)^i.
// original: corpus/submissions/4131060.cpp
const MOD = 1_000_000_007
type Z = Mod<MOD>

let n: Int = read()
let comb = Comb<MOD>(n)
let step = Z(n - 1).inv
print(sum(comb.choose(n, i) * comb.fact(i - 1) * step.pow(i) for i in 2..=n))
