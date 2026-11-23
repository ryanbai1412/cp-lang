// expect: ok, output "2 0"
// An `Int` literal fills and assigns `Mod<P>` slots (std/mod.cp, rebuild round 6 H-MOD-FILL).
type Z = Mod<1000000007>
var w: [0..=3]Z = 0
w[0] = 1
w[1] = w[0] + w[0]
print(w[1], w[3])
