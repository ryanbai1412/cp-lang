// expect: R0401 at 3 (checked)
type Z = Mod<1_000_000_007>
let c = Comb<Z>()
print(c.catalan(-1))
