// expect: ok, output "6000000000"
// rule: 03.1.6
let a: [3]0..=2e9 = [3].filled(2e9)
print(a.sum())
