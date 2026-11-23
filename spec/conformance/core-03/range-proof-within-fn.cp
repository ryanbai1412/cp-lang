// expect: ok, output "12"
// rule: 03.4.2
// A declared parameter range proves the result range inside the fn.
fn triple(x: 0..=10) -> 0..=30 = x * 3
let y: 0..=30 = triple(4)
print(y)
