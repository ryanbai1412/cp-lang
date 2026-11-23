// expect: ok, output "2\n0"
// rule: 03.5.1 10.1.1
// input: "2\n7 10\n1 5\n"
// A runtime modulus read below the divider is a fresh type per case; `Mod(1)` holds 0.
@!multitest
let m = read<1..=1e9>()
let x = read<Int>()
type Z = Mod(m)
print(Z(x) * Z(x))
