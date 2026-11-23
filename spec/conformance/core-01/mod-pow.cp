// expect: ok, output "81 1"
// rule: 01.5.8
// `**` on `Mod<P>` is the modular power (rebuild round 16 H-MOD-POW).
type Z = Mod<1000000007>
let x: Z = 3
print(x ** 4, x ** 0)
