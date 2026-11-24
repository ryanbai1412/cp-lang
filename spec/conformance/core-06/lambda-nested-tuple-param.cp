// expect: ok, output "7"
// rule: 06.1.4
// A lambda parameter may be a nested tuple pattern (decision 153).
fn app(f: (Int, (Int, Int)) -> Int) -> Int = f(1, (2, 3))
print(app((d, (u, v)) => d + u * v))
