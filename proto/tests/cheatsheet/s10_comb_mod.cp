const MOD = 1_000_000_007
type Z = Mod<MOD>
let c = Comb<MOD>(100)
let half: Z = Z(1) / Z(2)
let p: Z = Z(2) ** 3
print(c.choose(5, 2), c.arrange(5, 2), c.fact(5), half, half * Z(2), (p - Z(5)).pow(1))
