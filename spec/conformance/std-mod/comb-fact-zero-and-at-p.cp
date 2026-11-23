// expect: ok, output "1 0"
type Z = Mod<7>
let c = Comb<Z>()
print(c.fact(0), c.fact(7))
