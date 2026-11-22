// expect: ok, output "3 12"
// rule: 06.1.3
fn f((a, b): (Int, Int)) -> Int = a + b
let g = ((x, y): (Int, Int)) => x * y
print(f((1, 2)), g((3, 4)))
