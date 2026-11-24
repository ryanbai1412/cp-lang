// expect: ok, output "998244352 3"
// rule: 02.6.2
// On its right side an alias's own name means what it means outside it:
// `Mod` there is std's `Mod`, not the alias being defined.
type Mod = Mod<998244353>
let x: Mod = Mod(10) * Mod(5) - Mod(51)
print(x, Mod(3))
