// expect: ok, output "5000000"
// rule: 13.2.9
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
let n = zero(3) + 5000000
var a: [n]Int = 0
print(a.len)
