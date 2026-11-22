// expect: ok, output "7 3"
// rule: 06.1
// `min` and `max` lower as first-class function values even without a Fn type hint.
fn apply2(f: (Int, Int) -> Int, a: Int, b: Int) -> Int = f(a, b)
let f = max
let g = min
print(apply2(f, 3, 7), apply2(g, 3, 7))
