// expect: ok, output "0 0"
// rule: 09.3.6 09.2.2
// tool: run --release
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
let d = zero(3)
print(7 / d, 7 % d)
