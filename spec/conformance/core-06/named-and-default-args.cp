// expect: ok, output "11 3 33 5"
// rule: 06.1.5
fn f(a: Int, b: Int = 10; scale: Int = 1) -> Int = (a + b) * scale
print(f(1), f(1, 2), f(1, scale: 3), f(a: 2, b: 3))
