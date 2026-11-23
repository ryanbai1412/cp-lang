type Z = Mod<998244353>

let (n, k): (1..=1000000, 0..=1000000) = read()
let comb = Comb<Z>(n)
print(comb.fact(min(k, n)))
