// expect: ok, output "8000000000"
// rule: 03.1.6
let a: [2]0..=4e9 = [2].filled(4e9)
print(a[0] + a[1])
