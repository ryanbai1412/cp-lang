// expect: ok, output "1 0 0"
type Z = Mod<1_000_000_007>
let c = Comb<Z>()
print(c.multichoose(0, 0), c.multichoose(3, 0), c.multichoose(-1, 2))
