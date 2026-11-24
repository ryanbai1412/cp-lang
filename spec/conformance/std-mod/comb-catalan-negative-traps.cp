// expect: E0902
type Z = Mod<1_000_000_007>
let c = Comb<Z>()
print(c.catalan(-1)) // <- here
