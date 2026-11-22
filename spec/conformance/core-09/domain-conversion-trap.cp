// expect: R0102 at 4 (checked)
// rule: 09.3.2
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
domain V = 1..=3
let x = zero(3) + 4
let v = V(x)
print(v)
