// expect: E0310 at 3
// rule: 02.6.3
// With a declared `new`, the generated field constructor is not callable outside the module.
type Z = Mod<7>
print(Z(v: 3))
