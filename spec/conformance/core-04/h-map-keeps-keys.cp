// expect: ok, output "60"
// rule: 04a.9.4
let a = [10, 20, 30, 40]
let m = a[1..3].map(x => x * 2)
let i = 2
print(m[i])
