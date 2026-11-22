// expect: R0202 at 3
// rule: 03.4.1
fn one(k: Int) -> Int = if k == 0: 1 else: one(k - 1)
let a = 5e18 * one(3)
print(a + a - a)
