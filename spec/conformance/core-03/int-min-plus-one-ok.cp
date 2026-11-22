// expect: ok, output "-9223372036854775807"
// rule: 03.3.3
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
let a = -9223372036854775807
print(a - zero(3))
