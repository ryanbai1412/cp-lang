// expect: R0101 (checked)
// rule: 09.2.1 09.5.2
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
let a = [1, 2, 3]
let i = zero(3) + 5
print(a[i]) // <- here
