// expect: R0201 at 3 (checked)
// rule: 09.3.3
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
var x: 0..7 = 0
x = zero(3) + 9
print(x)
