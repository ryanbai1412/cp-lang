// expect: E0303
// rule: 06.1.5
fn f(a: Int, b: Int = 10; scale: Int = 1) -> Int = (a + b) * scale
print(f(1, 2, 3)) // <- here
