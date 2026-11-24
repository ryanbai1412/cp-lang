// expect: R0401 (checked)
type Z = Mod<7>
let c = Comb<Z>()
print(c.inv_fact(7)) // <- here
