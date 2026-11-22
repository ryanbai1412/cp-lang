// expect: E0303 at 2
// rule: 01.10.1
fn f(a: Int, b: Int; scale: Int = 1) -> Int = (a - b) * scale
print(f(5, 2, 10))
