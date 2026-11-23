// expect: ok, output "1 1 5 1 0 1 2"
type Z = Mod<1_000_000_007>
let c = Comb<Z>()
print(c.catalan(0), c.catalan(1), c.catalan(3), c.derange(0), c.derange(1), c.derange(2), c.derange(3))
