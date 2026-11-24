// expect: R0203 (checked)
// rule: 09.3.6
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
let d = zero(3)
print(10 / d) // <- here
