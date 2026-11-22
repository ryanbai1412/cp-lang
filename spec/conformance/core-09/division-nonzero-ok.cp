// expect: ok, output "5"
// rule: 09.3.6
fn two(k: Int) -> Int = if k == 0: 2 else: two(k - 1)
let d = two(3)
print(10 / d)
