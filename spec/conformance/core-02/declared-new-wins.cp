// expect: ok, output "3 6"
// rule: 02.6.3
// `T(args)` calls a declared `new`: `Mod` reduces, it does not range-check `v` (rebuild round 6 H-MOD-CTOR).
type Z = Mod<7>
print(Z(10), Z(-1))
