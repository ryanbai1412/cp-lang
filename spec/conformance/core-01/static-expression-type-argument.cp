// expect: ok, output "1 2"
// rule: 01.3.4
const P = 1e9 + 7
type Z = Mod<1e9 + 7>
type Y = Mod<P>
print(Z(1e9 + 8), Y(P + 2))
