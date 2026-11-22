// expect: R0202 at 3
// rule: 03.3.3
fn one(k: Int) -> Int = if k == 0: 1 else: one(k - 1)
let a = -9223372036854775807
print(a - one(3))
