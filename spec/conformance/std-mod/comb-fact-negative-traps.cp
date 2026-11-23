// expect: R0401 at 4 (checked)
type Z = Mod<7>
let c = Comb<Z>()
print(c.fact(-1))
