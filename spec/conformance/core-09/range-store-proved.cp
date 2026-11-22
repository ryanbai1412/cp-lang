// expect: ok, output "3"
// rule: 09.3.3
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
let y: 0..7 = (zero(3) + 10) % 7
print(y)
