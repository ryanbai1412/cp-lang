// expect: ok, output "15 8 9"
// rule: 03.1.7
// An `Int` beside a `Mod<P>` in `+`, `-` or `*` converts to `Mod<P>` (rebuild round 29 H16).
type Z = Mod<998244353>
let z = Z(3)
let k = 5
print(z * k, k + z, z ** 2)
