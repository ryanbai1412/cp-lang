// expect: ok, output "5"
// rule: 05.10.2
// std_functions: 1
fn adder(k: Int) -> (Int -> Int) = x => x + k
let f = adder(2)
print(f(3))
