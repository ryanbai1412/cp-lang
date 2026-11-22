// expect: ok, output "3 2"
// rule: 12.4.8
fn total(xs: List<Int>) -> Int = xs.sum()
let a = [1, 2]
print(total(a), a.len)
