// expect: ok, output "125"
// rule: 02.6.2
// A type alias constructs as the type it names.
type Z = Mod<1e9 + 7>
let a = 5
print(Z(a) ** 3)
