// expect: ok, output "70"
// rule: 04a.4.2 04b.1.1
let a = [10, 20, 30, 40, 50]
let w = a[1..5]
print(w[2..4].sum())
