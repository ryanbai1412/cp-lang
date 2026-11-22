// expect: ok, output "3 3 30"
// rule: 01.10.1
fn f(a: Int, b: Int; scale: Int = 1) -> Int = (a - b) * scale
print(f(5, 2), f(b: 1, a: 4), f(5, 2, scale: 10))
