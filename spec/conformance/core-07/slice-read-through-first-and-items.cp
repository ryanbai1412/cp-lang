// expect: ok, output "2 3"
// rule: 07.4.5
let a = [1, 2, 3, 4]
let w = a[1..3]
print(w.first!, w.nth(1))
