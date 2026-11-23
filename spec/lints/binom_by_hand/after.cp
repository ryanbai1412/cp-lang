let (a, b): (0..=1000, 0..=1000) = read()
type Z = Mod<998244353>
let c = Comb<Z>(1000)
print(c.choose(a, b))
