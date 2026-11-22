// expect: ok, output "2000000000 1000000000000000000"
// rule: 03.1.6
let a: [2]0..=1e9 = [2].filled(1e9)
print(a[0] + a[1], a[0] * a[1])
