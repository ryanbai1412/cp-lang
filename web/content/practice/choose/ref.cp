type Z = Mod<1_000_000_007>
let q: 1..=2e5 = read()
let c = Comb<Z>(1e6)
for _ in 0..q:
    let (n, k): (0..=1e6, 0..=1e6) = read()
    print(c.choose(n, k))
