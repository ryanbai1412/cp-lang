// expect: ok, output "0"
// rule: std.mod.Mod.inv_all
type Z = Mod<7>
let xs: List<Z> = []
print(Z.inv_all(xs).len)
