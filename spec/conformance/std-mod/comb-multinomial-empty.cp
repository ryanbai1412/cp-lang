// expect: ok, output "1 1"
type Z = Mod<1_000_000_007>
let c = Comb<Z>()
let e: List<Int> = []
print(c.multinomial(e), c.multinomial([3]))
