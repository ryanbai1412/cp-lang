type Z = Mod<1_000_000_007>
const MAX_A = 1e6
let comb = Comb<Z>(MAX_A)
let n = read<1..=1e5>()
for (a, b) in read<[n](a: 1..=MAX_A, b: 0..=MAX_A)>():
    print(comb.choose(a, b))
