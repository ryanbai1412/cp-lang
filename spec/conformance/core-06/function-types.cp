// expect: ok, output "12"
// rule: 06.2.5
fn apply(f: (Int, Int) -> Int, a: Int, b: Int) -> Int = f(a, b)
print(apply((x, y) => x * y, 3, 4))
