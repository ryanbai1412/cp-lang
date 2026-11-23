// expect: ok, output "120 10 60 44"
// rule: 05.10.11

type Z = Mod<1_000_000_007>
let c = Comb<Z>()
print(c.fact(5), c.choose(5, 2), c.arrange(5, 3), c.derange(5))
