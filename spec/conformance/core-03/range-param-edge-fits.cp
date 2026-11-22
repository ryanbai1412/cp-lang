// expect: ok, output "18"
// rule: 03.1.4
fn f(x: 0..=9) -> Int = x * 2
let y = 9
print(f(y))
