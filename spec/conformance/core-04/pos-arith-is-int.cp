// expect: ok, output "3"
// rule: 04.8.2
domain V = 1..=4
let x: Int = V(2) + 1
print(x)
