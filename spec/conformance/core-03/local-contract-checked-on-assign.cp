// expect: R0201 at 3 (checked)
// rule: 03.1.2
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
var x: 0..=10 = 5
x = x + 6 + zero(3)
print(x)
