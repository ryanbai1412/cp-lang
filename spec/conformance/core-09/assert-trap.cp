// expect: R0401 (checked)
// rule: 09.3.7
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
let n = zero(3)
assert n > 0 // <- here
print(n)
