// expect: ok, output "30\n30"
// rule: 04a.8.3
let a = [10, 20, 30, 40]
let i = 2
print(a[1..3][i])
print(a[0..3][i])
