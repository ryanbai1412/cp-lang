const MAXA = 1_000_000
type Z = Mod<1_000_000_007>
let n = read<1..=1e5>()
let c = Comb<Z>(MAXA)
for _ in 0..n:
    let (a, b) = read<(0..=MAXA, 0..=MAXA)>()
    print(c.choose(a, b))
