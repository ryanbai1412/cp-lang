// expect: ok, output "2"
// rule: 09.3.7
fn two(k: Int) -> Int = if k == 0: 2 else: two(k - 1)
let n = two(3)
assert n > 0
print(n)
