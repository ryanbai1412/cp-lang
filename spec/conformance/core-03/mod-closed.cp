// expect: ok, output "1"
// rule: 03.5.1
type Z = Mod<7>
let x = Z(5) + Z(3)
print(x)
