// expect: ok, output "111"
// rule: 06.1.8
fn f(x: Int, y: Int = 10, z: Int = 100) -> Int = x + y + z
fn apply(g: Int -> Int, x: Int) -> Int = g(x)
print(apply(f, 1))
