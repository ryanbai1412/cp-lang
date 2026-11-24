// expect: E0902
type Z = Mod<7>
let c = Comb<Z>()
print(c.fact(-1)) // <- here
