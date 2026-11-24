// expect: R0401 (checked)
type Z = Mod<1_000_000_007>
let c = Comb<Z>()
print(c.catalan(-1)) // <- here
