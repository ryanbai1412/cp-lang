// expect: ok, output "9"
// rule: 01.3.5
fn k(_: Int) -> Int = 7
let (_, b) = (1, 2)
print(k(0) + b)
