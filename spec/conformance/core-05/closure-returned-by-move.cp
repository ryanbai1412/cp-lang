// expect: ok, output "5"
// rule: 05.10.2
fn adder(k: Int) -> (Int -> Int) = x => x + k
let f = adder(2)
print(f(3))
