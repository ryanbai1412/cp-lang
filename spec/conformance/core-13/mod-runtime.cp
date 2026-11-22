// expect: ok, output "2"
// rule: 13.2.12
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
let p = zero(3) + 7
type Z = Mod(p)
let x = Z(10)
print(x * x)
