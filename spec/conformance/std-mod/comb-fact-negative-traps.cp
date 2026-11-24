// expect: R0401 (checked)
type Z = Mod<7>
let c = Comb<Z>()
print(c.fact(-1)) // <- here
