// expect: ok, output "1000000000000000000"
// rule: 03.1.2
let x: 0..=1e9 = 1e9
let y = x * x
print(y)
